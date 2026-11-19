"""Time two JOCForge encoders on the same ADM and compare the complete EC3 streams."""

import argparse
import filecmp
import subprocess
import tempfile
from pathlib import Path
from time import perf_counter


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path, help="encoder built before the change")
    parser.add_argument("candidate", type=Path, help="encoder built after the change")
    parser.add_argument("adm", type=Path)
    parser.add_argument("--joc-layout", choices=["flat7x"])
    parser.add_argument("--data-rate", type=int)
    args = parser.parse_args()
    options = []
    if args.joc_layout:
        options.extend(["--joc-layout", args.joc_layout])
    if args.data_rate is not None:
        options.extend(["--data-rate", str(args.data_rate)])

    with tempfile.TemporaryDirectory(prefix="jocforge-compare-") as temp:
        outputs = []
        for label, binary in [("baseline", args.baseline), ("candidate", args.candidate)]:
            directory = Path(temp) / label
            directory.mkdir()
            output = directory / "output.candidate.ec3"
            start = perf_counter()
            subprocess.run([binary.resolve(), "encode", args.adm, output, *options], check=True)
            print(f"{label}: {perf_counter() - start:.3f} s", flush=True)
            outputs.append(output)

        if not filecmp.cmp(*outputs, shallow=False):
            raise SystemExit("EC3 files differ")
        print(f"Bit-exact: {outputs[0].stat().st_size:,} bytes")


if __name__ == "__main__":
    main()
