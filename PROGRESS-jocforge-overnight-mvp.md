# 进度追踪：JOCForge overnight MVP

> 创建时间：2026-09-02 | 状态：进行中

## 目标
构建独立的 clean-room/public-spec-based JOCForge MVP，实现受限 ADM/BW64 输入到新生成 raw E-AC-3/JOC `.ec3` 的端到端编码、OpenJOC 外部严格验证、解码和渲染。

## 成功标准
- 优先调查并记录 EBU `libadm`、`libbw64`、`libear` 的上游、版本/提交、许可证、API、构建成本和适用性。
- 生成 idx0、真正的 idx1 Flat-7.X、真正的 idx2 5.X+2 多 AU raw `.ec3`。
- 通过 OpenJOC v0.15.0 strict validation，含 OAMD、full matrix、有限对象 PCM 和可听 round-trip。
- 支持实际 BW64/ADM 输入和仓库自生成的可复现实验 fixture。
- 测试、文档、来源记录、指标和质量门禁可复现；不使用专有 Dolby encoder。

## 已读文件
- `C:/Users/chyinan/.codex/RTK.md` — shell 命令必须优先通过 `rtk`。
- `C:/Users/chyinan/.agents/skills/using-plan-and-execute/SKILL.md` — 先发现并遵循适用技能。
- `C:/Users/chyinan/.agents/skills/focused-problem-solver/SKILL.md` — 四检查点、进度和摘要要求。
- `C:/Users/chyinan/.agents/skills/brainstorming/SKILL.md` — 编码前设计探索与验证。
- `C:/Users/chyinan/.agents/skills/coding-effectively/SKILL.md` — Rust 编码工作流。
- `C:/Users/chyinan/.agents/skills/functional-core-imperative-shell/SKILL.md` — 纯核心与副作用壳分离。
- `C:/Users/chyinan/.agents/skills/investigating-a-codebase/SKILL.md` — 先调查代码库事实。
- `C:/Users/chyinan/.agents/skills/test-driven-development/SKILL.md` — 先写失败测试再实现。
- `C:/Users/chyinan/.agents/skills/writing-good-tests/SKILL.md` — 测试隔离与条件验证。
- `C:/Users/chyinan/.agents/skills/verification-before-completion/SKILL.md` — 完成声明必须有新鲜验证证据。
- `C:/Users/chyinan/.agents/skills/maintaining-project-context/SKILL.md` — 阶段完成时维护项目上下文。
- `C:/Users/chyinan/.agents/skills/anti-overengineering/SKILL.md` — 控制 MVP 复杂度。
- `D:/Programs/OpenJOC/summary/legacy-ac3-core-joc-frontend.md` — OpenJOC external carriage contract summary.
- `D:/Programs/OpenJOC/summary/joc-idx234-implementation.md` — idx0/idx1/idx2 topology and external-validation boundary.
- `D:/Programs/OpenJOC/crates/openjoc-emdf/src/lib.rs` — bounded EMDF parser and ETSI strict payload configuration.
- `D:/Programs/OpenJOC/crates/openjoc-joc/src/parser.rs` — JOC header, full-matrix syntax, 5/7 input counts.
- `D:/Programs/OpenJOC/crates/openjoc-joc/src/reconstruction.rs` — full-matrix differential and coefficient quantization behavior.
- `D:/Programs/OpenJOC/crates/openjoc-ffmpeg/src/lib.rs` — public test-only synthetic I0/D0 carriage reference.
- `D:/Programs/OpenJOC/crates/openjoc-oamd/src/payload.rs` — OAMD element window and content-prefix parsing boundary.
- `D:/Programs/OpenJOC/crates/openjoc-oamd/src/object_element.rs` — object active/basic/render field order.
- `D:/Programs/JOCForge/summary/ebus-and-openjoc.md` — EBU library audit, integration decision, supplied file observation.

## 当前进度
writer-first proof、窄 BW64/ADM fixture、rank-controlled analysis、FFmpeg I0 + JOCForge D0 packaging、三种 4 秒 profile 输出、OpenJOC strict inspect/decode/render、确定性和文档已完成；正在执行最终质量门禁与 adversarial review。

## 下一步
刷新运行 cargo quality gates、复核 Git diff、提交本地 commit；最终报告真实输入文件观察、三个 profile 证据、MVP 限制和下一步。

## 发现的关键信息
- 用户要求 writer-first，OpenJOC 只能作为外部验证器/解码器/渲染器。
- 用户要求：若 C++ FFI 会吞掉 overnight MVP，则以公共语义为参考，暂用最窄的自有 BW64/axml/chna 子集，并在 `PROVENANCE.md` 明确记录。
- `libadm` upstream `master` at `aed1ac5aea2844da374712a2b51339a09485df6f`; Apache-2.0; C++11; Boost 1.57+ headers; CMake; upstream CMake version 0.14.0.
- `libbw64` upstream `master` at `d3dae5c74deada286994c8a4c2a03edd82d65589`; Apache-2.0; header-only C++; CMake; upstream CMake version 0.10.0; BW64/RF64, ds64, axml, chna, 16/24/32-bit PCM.
- `libear` upstream `master` at `2db69f8fcea0bc5db8a78e14a9c2ae6ed4283c15`; Apache-2.0; C++14; Boost 1.57+ and recursive submodules; independent BS.2127 render reference only.
- Initial integration choice: keep all three as documented public references; defer full C++ FFI unless local toolchain proves it is low-cost; do not copy their encoding/render internals.
- Writer proof: JOCForge-generated one-AU idx0/idx1/idx2 were externally inspected by OpenJOC as ETSI_STRICT/normative_compliant; idx1 and idx2 rendered through the external OpenJOC CLI.
- Synthetic fixture is OpenJOC ADM-BWF-valid: BW64 + ds64 + 24-bit PCM + axml + chna + dbmd; 15 objects, 192000 samples, 125 complete JOC units.
- Final raw outputs are each 832000 bytes: 125 access units, each 4096-byte I0 plus 2560-byte D0, actual 1664000 bit/s.
- OpenJOC external evidence: all three outputs strict inspect exit 0, 125/125 EMDF parsed, 0 malformed; all three render exit 0; FFmpeg independently decodes each stream exit 0; OpenJOC `decode --internal-base` produced 15 reconstruction rows for each profile.
- Repeat encode SHA-256 is identical: `928fe595e967889c674cfea92adb4c3b5f5b68e6f7cf68c39548658ea5d8c6be`.
- The supplied `D:\ADM BWF\打上花火.wav` remains ordinary 38-channel RIFF PCM at the inspected header boundary and is rejected by the ADM parser for missing `axml`; no silent ADM reinterpretation is performed.
