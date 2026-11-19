"""Package a Windows release build."""

import argparse
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    with ZipFile(args.archive, "x", ZIP_DEFLATED) as archive:
        archive.write(root / "target/release/jocforge.exe", "jocforge.exe")
        for name in ["README.md", "LICENSE", "THIRD_PARTY_NOTICES.md"]:
            archive.write(root / name, name)


if __name__ == "__main__":
    main()
