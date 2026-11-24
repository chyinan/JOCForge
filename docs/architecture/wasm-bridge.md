# 浏览器桥接（jocforge-wasm）

本文记录浏览器侧编码的边界契约。实现全部在 `crates/jocforge-wasm`（ABI 与错误分类）和 `crates/jocforge-pipeline`（编码管线）里，CLI 的输入输出不受影响。

## 1. 分层

```
ffi.rs        原始 C ABI：句柄、分配、状态码
lib.rs        薄封装：只做错误分类，不再自带一套输入准备
pipeline      真正的编码：解析、准入、meter、spatial、发布
io            内存 provider 树与内存发布适配器
```

浏览器和 CLI 走**同一条** `run_inner`。桥接里不复制解析或准备逻辑——一旦复制，两条路径就会漂移。

## 2. ABI

| 导出 | 语义 |
| --- | --- |
| `jocforge_wasm_encoder_create()` | 返回句柄，`0` 表示失败 |
| `jocforge_wasm_alloc(len)` / `jocforge_wasm_dealloc(ptr, len)` | 有界分配，上限 64 MiB |
| `jocforge_wasm_encoder_push_bytes(handle, ptr, len)` | 追加输入；返回状态码，`< 0` 为错误 |
| `jocforge_wasm_encoder_finish(handle)` | 执行编码；`1` = `BitstreamAvailable` |
| `jocforge_wasm_encoder_bitstream_ptr/len(handle)` | 取产物字节 |
| `jocforge_wasm_encoder_error_ptr/len/category(handle)` | 取错误详情与分类（0 输入、1 编码、2 生命周期、3 内部） |

`Encoder::finish` 在浏览器目标把已积累的输入 `mem::take` 后直接交给
`encode_adm_resource`，因此输入只驻留一份；native 测试构建仍借用同一缓冲区。

模块实例化时必须提供 `env.jocforge_host_len` 和 `env.jocforge_host_read`，即使只用
`push_bytes` 也不能传空的 imports 对象。仓库内的 Node 脚本共用
[`residentImports`](../../scripts/wasm-resident-imports.mjs)：没有宿主文件时，长度返回
`0n`（WASM `i64`），读取返回 `-1`；这不会影响已推入内存的输入。使用
`jocforge_wasm_encoder_finish_streamed` 进行实际编码时，需由宿主提供真实文件的长度，
并按偏移将所请求的字节完整写入模块内存后返回 `0`。无文件的占位实现不能用于流式编码。

## 3. 没有命名空间时的发布

`wasm32-unknown-unknown` 上 `Path::is_absolute()` 恒为假，F15 的"目的地必须是绝对路径"无法满足。
唯一接受的例外是保留 scheme：

```
memory://output.ec3
```

命中该标记时，准入跳过 `is_absolute` 检查，其余 owner provider、档位与身份校验照常执行；
发布改由 `MemoryOutputStaging` 在隔离区 buffer 上完成，receipt 语义见该类型注释。
文件目标（CLI）不认该标记，仍然强制绝对路径。

## 4. 栈

DSP 阶段在 native 有 16–32 MiB 的专用线程栈。浏览器里没有线程，这些阶段跑在模块栈上，
默认栈会直接 `memory access out of bounds`。`.cargo/config.toml` 给该目标加了：

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-arg=-zstack-size=16777216"]
```

## 5. 执行模型

- **meter prepass**：wasm 内联执行（native 走线程）。
- **spatial frame producer**：wasm 由消费者按序拉取（`&mut dyn FnMut(u64) -> SpatialFrameResult`），
  不预物化整段节目；native 仍是单槽 `sync_channel(1)` + 生产者线程。

## 6. 验证

```powershell
# 产物
cargo build -p jocforge-wasm --release --locked --target wasm32-unknown-unknown

# ABI 冒烟（分配上限、句柄生命周期、截断输入拒绝）
node scripts/test-wasm.mjs target/wasm32-unknown-unknown/release/jocforge_wasm.wasm

# 与 native 的逐字节对照（需要 JOCFORGE_ADM_PARITY 指向真实 ADM）
cargo test -p jocforge-wasm dumps_excerpt_and_native_bitstream -- --nocapture
node scripts/compare-wasm-encode.mjs `
  target/wasm32-unknown-unknown/release/jocforge_wasm.wasm `
  .test-tmp/wasm-parity/excerpt.wav `
  .test-tmp/wasm-parity/native.ec3 `
  .test-tmp/wasm-parity/wasm.ec3
```

已记录结果：2,468,488 B / 12288 帧 excerpt → 两端均 32,768 B（8 AU），`bit_exact=true`。

## 7. 已知限制

- **输入预算 512 MiB**（`MAX_RESIDENT_INPUT_BYTES`）：超过即 `push_bytes` 直接返回 `InvalidInput`，不会在编码中途 OOM 把模块打废。整份 PCM 必须常驻，所以更长的节目属于 native 编码；常规几十 MB 的素材不受影响。
- 内存发布的终端 receipt 沿用 domain 唯一的 finalize 构造函数（`atomic_replace` / `parent_fsync` 恒为真），
  其语义是"buffer 即产品、交接即完成"；是否给 domain 加 memory 变体待 owner 决定。
- `admission_rejection_detail` 是 wasm-only 诊断，浏览器报错时附带 F15 具体拒绝因子，稳定后可删。
