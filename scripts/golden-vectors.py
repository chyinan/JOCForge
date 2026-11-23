"""编码产物的基准向量：改造流式输入前后必须逐字节一致。

用法：
    python scripts/golden-vectors.py --update          # 记录当前产物
    python scripts/golden-vectors.py                   # 校验（不一致则退出码非 0）
    python scripts/golden-vectors.py --input other.wav # 换输入（会连带换基准文件）

设计要点：
- 只用**命令行参数**构造用例，不用 job XML。XML 里含绝对路径，换目录就会改变
  身份信息，做基准不可靠；XML 相关的档位由 AtmosLabs 的 check-drc / check-trims 覆盖。
- 每个用例都完整读一遍 ADM 的 PCM，因此块读取层任何偏移/长度偏差都会改变产物 sha。
- flat 7.1 要求输出名以 .candidate.ec3 结尾且父目录为空，所以每个用例单独建目录。
"""

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "docs/memory/agents/golden-encode.json"
DEFAULT_INPUT = ROOT / ".test-tmp/wasm-parity/excerpt.wav"
DEFAULT_BIN = ROOT / "target/release/jocforge.exe"

# (名字, 额外参数, 是否需要 flat7x 的空目录 + .candidate.ec3 输出名)
CASES = [
    *( (f"5.1-{rate}", ["--data-rate", str(rate)], False)
       for rate in (384, 448, 576, 640, 768, 1024) ),
    *( (f"flat7x-total-{total}",
        ["--joc-layout", "flat7x", "--total-rate", str(total)],
        True)
       for total in (1024, 1088, 1216, 1280, 1408, 1664) ),
    ("flat7x-ex-unspecified",
     ["--joc-layout", "flat7x", "--total-rate", "1024", "--surround-ex", "0"], True),
    ("flat7x-ex-encoded",
     ["--joc-layout", "flat7x", "--total-rate", "1024", "--surround-ex", "2"], True),
    ("flat7x-ex-not-encoded",
     ["--joc-layout", "flat7x", "--total-rate", "1024", "--surround-ex", "1"], True),
    ("topology-dependent-1", ["--dependent-count", "1"], False),
]


def run_case(binary: Path, source: Path, case) -> dict:
    name, extra, wants_candidate = case
    with tempfile.TemporaryDirectory(prefix="golden-") as directory:
        work = Path(directory)
        output = work / ("out.candidate.ec3" if wants_candidate else "out.ec3")
        command = [
            str(binary),
            "encode",
            *extra,
            "--emdf-profile",
            "deployed",
            str(source),
            str(output),
        ]
        completed = subprocess.run(
            command, capture_output=True, text=True, errors="replace"
        )
        if completed.returncode != 0 or not output.exists():
            detail = (completed.stderr or completed.stdout).strip().splitlines()
            raise SystemExit(
                f"{name}: 编码失败（退出码 {completed.returncode}）\n"
                + "\n".join(detail[-4:])
            )
        payload = output.read_bytes()
        return {"bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update", action="store_true", help="重写基准文件")
    parser.add_argument("--input", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--bin", type=Path, default=DEFAULT_BIN)
    parser.add_argument("--baseline", type=Path, default=BASELINE)
    args = parser.parse_args()

    source = args.input.resolve()
    binary = args.bin.resolve()
    if not source.is_file():
        raise SystemExit(f"输入不存在: {source}")
    if not binary.is_file():
        raise SystemExit(f"找不到可执行文件: {binary}（先 cargo build --release -p jocforge-cli）")

    measured = {}
    for case in CASES:
        measured[case[0]] = run_case(binary, source, case)

    if args.update:
        args.baseline.parent.mkdir(parents=True, exist_ok=True)
        document = {
            "input": source.name,
            "input_bytes": source.stat().st_size,
            "cases": measured,
        }
        args.baseline.write_text(
            json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        print(f"已写入 {len(measured)} 个基准用例 → {args.baseline.relative_to(ROOT)}")
        for name, value in measured.items():
            print(f"  {name:28} {value['bytes']:>8} B  {value['sha256'][:16]}")
        return 0

    if not args.baseline.is_file():
        raise SystemExit(f"缺少基准文件: {args.baseline}（先跑 --update）")
    expected = json.loads(args.baseline.read_text(encoding="utf-8"))["cases"]

    failures = []
    for name, want in expected.items():
        got = measured.get(name)
        if got is None:
            failures.append(f"{name}: 本次没有产出")
        elif got != want:
            failures.append(
                f"{name}: {want['bytes']} B {want['sha256'][:16]} → "
                f"{got['bytes']} B {got['sha256'][:16]}"
            )

    for name in measured:
        if name not in expected:
            failures.append(f"{name}: 新增用例但基准里没有")

    if failures:
        print(f"基准不一致（{len(failures)}/{len(expected)}）:")
        for line in failures:
            print(f"  {line}")
        return 1

    print(f"基准一致：{len(expected)} 个用例逐字节相同")
    return 0


if __name__ == "__main__":
    sys.exit(main())
