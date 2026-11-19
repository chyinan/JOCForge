"""Package a Windows release build and its runtime assets."""

import argparse
import subprocess
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    supplements = Path("outputs/IMPLEMENTER_SUPPLEMENTS")
    bundles = (
        "ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1",
        "F01B-F03W-COLD-STATE-ASSETS-V1",
        "METER-APPROVED-PARAMETER-ASSETS-V1",
        "METER-PROVIDER-SET-IDENTITY-APPLICATION-V1",
        "F04-RECEIVER-APPROVED-PARAMETERS-V1",
    )
    assets = []
    for name in bundles:
        files = subprocess.check_output(
            ["git", "ls-files", "-z", "--", (supplements / name).as_posix()], cwd=root
        ).decode("utf-8").split("\0")
        paths = [path for path in files if path]
        if not paths:
            raise SystemExit(f"Missing provider bundle: {name}")
        assets.extend(paths)

    with ZipFile(args.archive, "x", ZIP_DEFLATED) as archive:
        archive.write(root / "target/release/jocforge.exe", "jocforge.exe")
        for name in ["README.md", "LICENSE", "THIRD_PARTY_NOTICES.md", *assets]:
            archive.write(root / name, name)


if __name__ == "__main__":
    main()
