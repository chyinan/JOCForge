"""在 V1 和 V2 的实际 Rust 边界分别采样，再生成供独立 Wash 检查器比较的 JSON。"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[1]
ROUTES = ROOT / "crates/jocforge-domain/tests/fixtures/default_auto_position/routes.json"


def collect(baseline):
    rows = {}
    for crate, test in [
        ("jocforge-domain", "washed_rate_routes_exercise_the_actual_point_consumer"),
        ("jocforge", "washed_rate_route_rewrap_copies_the_elementary_stream"),
    ]:
        run = subprocess.run(
            (["rtk", "proxy"] if shutil.which("rtk") else [])
            + ["cargo", "test", "--locked", "-p", crate, "--lib", test, "--", "--nocapture"],
            cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, encoding="utf-8", errors="replace",
        )
        for line in run.stdout.splitlines():
            if line.startswith("JOC51_ROUTE "):
                key, value = json.loads(line.removeprefix("JOC51_ROUTE "))
                if key in rows:
                    raise ValueError(f"重复用例：{key}")
                rows[key] = value
        # V1 的域测试会在输出所有实际值后报告新增五档未命中。
        if run.returncode and not (baseline and crate == "jocforge-domain"
                                   and "assertion failed" not in run.stdout
                                   and "r05-768-v1-exception" in run.stdout):
            raise RuntimeError(run.stdout[-5000:])
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True, help="V1 冻结的实现观测 JSON")
    parser.add_argument("--actual", type=Path, help="不指定时，仅在尚未改动的 V1 上采样")
    args = parser.parse_args()
    cases = json.loads(ROUTES.read_bytes())["cases"]
    keys = {key for key, _, _ in cases}
    rows = collect(args.actual is None)
    if set(rows) != keys:
        raise ValueError(f"路由 ID 不完整，缺少 {keys - set(rows)}")
    if args.actual is None:
        for key, delta, decision in cases:
            # 先确认采到的是 V1：仅实际原生 1024 正例会进入增益分支。
            rate = delta.get("kbps", 1024)
            if ("numeric" in rows[key]) != (decision == "V1_ONCE" and rate == 1024):
                raise ValueError(f"未得到 V1 基线：{key}")
        with args.baseline.open("x", encoding="utf-8") as f:
            json.dump(rows, f, indent=2)
        print(f"已保存 {len(rows)} 项 V1 实际观测：{args.baseline}")
        return
    before = json.loads(args.baseline.read_bytes())
    if set(before) != keys:
        raise ValueError("V1 基线不完整")
    actual = {}
    for key, _, _ in cases:
        row = rows[key]
        if "numeric" in row:
            actual[key] = dict(decision="V1_ONCE", calls=row["calls"], numeric=row["numeric"])
            if key == "r06-1024-already-active":
                actual[key]["v1_regression"] = dict(before=before[key]["result"]["sha256"],
                                                    after=row["result"]["sha256"])
                if before[key] != row:
                    raise ValueError("原 1024 路径的数值或调用次数变化")
        elif "after_es" in row:
            if before[key] != row or row["after_es"] != row["before_es"]:
                raise ValueError("再封装后的基本流变化")
            actual[key] = row
        else:
            actual[key] = dict(decision="BASELINE", calls=row["calls"],
                               before=before[key]["result"], after=row["result"])
            if actual[key]["before"] != actual[key]["after"]:
                raise ValueError(f"原有行为变化：{key}")
    args.actual.write_text(json.dumps(actual, indent=2), encoding="utf-8")
    print(f"已保存 {len(rows)} 项候选结果：{args.actual}")


if __name__ == "__main__":
    main()
