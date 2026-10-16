# pattern: Imperative Shell
import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
MAP_RELATIVE = pathlib.Path("docs/implementation/IMPLEMENTATION-MAP.json")


def edit_map(change):
    def mutate(repository):
        path = repository / MAP_RELATIVE
        data = json.loads(path.read_text(encoding="utf-8"))
        change(data)
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")

    return mutate


def append_text(relative_path, text):
    def mutate(repository):
        path = repository / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8", newline="\n") as stream:
            stream.write(text)

    return mutate


def write_text(relative_path, text):
    def mutate(repository):
        path = repository / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")

    return mutate


def entry(data, functional_id):
    return next(item for item in data["functional_entries"] if item["functional_id"] == functional_id)


def root_entry(data, root_id):
    return next(item for item in data["programme_roots"] if item["root_id"] == root_id)


def tree_digest(directory):
    digest = hashlib.sha256()
    for path in sorted(item for item in directory.rglob("*") if item.is_file()):
        relative_path = path.relative_to(directory)
        if any(part in {".git", "target", "__pycache__"} for part in relative_path.parts):
            continue
        digest.update(relative_path.as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.digest()


def mutations():
    cases = [
        ("missing-architecture-doc", lambda repo: (repo / "docs/architecture/ARCHITECTURE.md").unlink()),
        ("missing-generated-map", lambda repo: (repo / "docs/implementation/CLOCK-MAP.md").unlink()),
        ("functional-id-removed", edit_map(lambda data: data["functional_entries"].pop())),
        (
            "functional-id-duplicated",
            edit_map(
                lambda data: data["functional_entries"][-1].update(
                    functional_id=data["functional_entries"][0]["functional_id"]
                )
            ),
        ),
        ("functional-id-replaced", edit_map(lambda data: data["functional_entries"][-1].update(functional_id="UNKNOWN"))),
        (
            "clock-task-not-go",
            edit_map(lambda data: data["phase_one"]["P1.1"]["tasks"][0].update(status="HOLD")),
        ),
        (
            "clock-spec-drift",
            edit_map(lambda data: entry(data, "F02").update(spec_files=["specs/missing.md"])),
        ),
        (
            "clock-file-drift",
            edit_map(lambda data: data["phase_one"]["P1.1"]["tasks"][0]["planned_files"].pop()),
        ),
        (
            "active-slice-status-drift",
            edit_map(lambda data: data["status"].update(CLOCK_FOUNDATION="LOCALLY_VERIFIED")),
        ),
        ("p12-not-held", edit_map(lambda data: data["phase_one"].update({"P1.2": "GO"}))),
        ("p13-not-held", edit_map(lambda data: data["phase_one"].update({"P1.3": "GO"}))),
        ("p14-not-held", edit_map(lambda data: data["phase_one"].update({"P1.4": "GO"}))),
        (
            "root-materialized",
            edit_map(lambda data: root_entry(data, "reader_eof").update(materialization_status="MATERIALIZED")),
        ),
        (
            "validation-reverse-dependency",
            append_text(
                pathlib.Path("crates/jocforge-domain/Cargo.toml"),
                '\njocforge-validation = { path = "../jocforge-validation" }\n',
            ),
        ),
        ("domain-file-io", append_text(pathlib.Path("crates/jocforge-domain/src/lib.rs"), "\nuse std::fs;\n")),
        ("domain-process-io", append_text(pathlib.Path("crates/jocforge-domain/src/lib.rs"), "\nuse std::process;\n")),
        ("domain-environment-io", append_text(pathlib.Path("crates/jocforge-domain/src/lib.rs"), "\nuse std::env;\n")),
        ("domain-network-io", append_text(pathlib.Path("crates/jocforge-domain/src/lib.rs"), "\nuse std::net;\n")),
        ("production-ffmpeg-reference", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), "\n// FFmpeg\n")),
        ("production-legacy-reference", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), "\n// legacy\n")),
        ("production-compat-reference", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), "\n// compat\n")),
        ("production-fallback-reference", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), "\n// fallback\n")),
        ("production-concat-macro", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), '\nconst VALUE: &str = concat!("a", "b");\n')),
        ("production-include-macro", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), '\ninclude!("missing.rs");\n')),
        ("production-path-module", append_text(pathlib.Path("crates/jocforge-application/src/lib.rs"), '\n#[path = "missing.rs"] mod missing;\n')),
        (
            "public-usize",
            append_text(pathlib.Path("crates/jocforge-domain/src/lib.rs"), "\npub fn raw_index(value: usize) -> usize { value }\n"),
        ),
        (
            "trait-usize",
            append_text(
                pathlib.Path("crates/jocforge-domain/src/lib.rs"),
                "\npub trait RawIndex { fn value(&self) -> usize; }\n",
            ),
        ),
        (
            "unapproved-build-script",
            write_text(pathlib.Path("crates/jocforge-application/build.rs"), "fn main() {}\n"),
        ),
        (
            "future-provider-detail",
            edit_map(lambda data: entry(data, "F03").update(cold_provider={"kind": "FILE_PROVIDER"})),
        ),
        (
            "profile-supported-row-added",
            append_text(
                pathlib.Path("docs/architecture/PROFILE-DIMENSIONS.md"),
                "\n| selected ADM BWF / extra | x | x | x | x | x |\n",
            ),
        ),
        (
            "active-clause-removed",
            edit_map(lambda data: entry(data, "F02")["clock"]["covered_clauses"].pop()),
        ),
        (
            "active-test-empty",
            edit_map(lambda data: entry(data, "F02").update(test_modules=[])),
        ),
        (
            "writer-types-merged",
            edit_map(lambda data: entry(data, "F02")["clock"].update(domains=["JocWriterIndex"])),
        ),
        (
            "future-entry-materialized",
            edit_map(lambda data: entry(data, "F03").update(status="PARTIALLY_MATERIALIZED", active_slice_materialization="MATERIALIZED")),
        ),
        (
            "render-target-in-production",
            append_text(
                pathlib.Path("crates/jocforge-domain/src/profile/production.rs"),
                "\npub type InvalidProductionProfile = RenderTarget;\n",
            ),
        ),
        (
            "f02-marked-complete",
            edit_map(lambda data: data["phase_one"]["P1.1"].update(completed_functional_ids=["F02"])),
        ),
        (
            "xstj-marked-complete",
            edit_map(lambda data: data["phase_one"]["P1.1"].update(completed_functional_ids=["XSTJ"])),
        ),
        (
            "clock-outcome-removed",
            edit_map(lambda data: entry(data, "F02")["error_outcomes"].pop()),
        ),
        (
            "clock-outcome-claimed-normative",
            edit_map(lambda data: entry(data, "F02")["error_outcomes"][0].update(code_authority="PACKAGE_NORMATIVE")),
        ),
        (
            "unrelated-root-provider",
            edit_map(lambda data: root_entry(data, "spatial_audio").update(cold_provider={"file": "unknown.json"})),
        ),
        (
            "unrelated-direct-vector",
            edit_map(
                lambda data: entry(data, "F03").update(
                    vectors={"kind": "DIRECT_VECTOR", "case_pointers": []}
                )
            ),
        ),
    ]
    return cases


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--scratch-root", type=pathlib.Path, required=True)
    args = parser.parse_args()
    scratch_root = args.scratch_root.resolve()
    repository = ROOT.resolve()
    if scratch_root == repository or repository.is_relative_to(scratch_root):
        print("scratch root must not be the repository or one of its ancestors", file=sys.stderr)
        return 2

    created_scratch_root = not scratch_root.exists()
    scratch_root.mkdir(parents=True, exist_ok=True)
    cases = mutations()
    failures = []
    try:
        for name, mutate in cases:
            case_dir = scratch_root / name
            try:
                case_dir.mkdir()
            except FileExistsError:
                failures.append(f"{name}: case directory already exists; left untouched")
                continue

            try:
                probe = case_dir / ".copy-probe"
                probe.write_text("write permission verified\n", encoding="utf-8")
                if probe.read_text(encoding="utf-8") != "write permission verified\n":
                    raise RuntimeError("scratch permission probe did not round-trip")
                probe.unlink()

                copied_repository = case_dir / "repository"
                ignore = shutil.ignore_patterns(".git", "target", "__pycache__")
                shutil.copytree(repository, copied_repository, ignore=ignore)
                before = tree_digest(copied_repository)
                mutate(copied_repository)
                after = tree_digest(copied_repository)
                if before == after:
                    raise RuntimeError("mutation did not change the copied repository")

                result = subprocess.run(
                    [
                        sys.executable,
                        "-X",
                        "utf8",
                        "-B",
                        "-O",
                        str(copied_repository / "scripts" / "check-boundaries.py"),
                    ],
                    cwd=copied_repository,
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                    timeout=120,
                    check=False,
                )
                if result.returncode == 0:
                    raise RuntimeError("boundary checker accepted the mutation")
                print("PASS " + name)
            except Exception as error:
                failures.append(f"{name}: {error}")
            finally:
                shutil.rmtree(case_dir)
    finally:
        if created_scratch_root and scratch_root.is_dir() and not any(scratch_root.iterdir()):
            scratch_root.rmdir()

    print(f"boundary mutation cases: {len(cases)}")
    if failures:
        for failure in failures:
            print("FAIL " + failure, file=sys.stderr)
        print(f"boundary mutation suite failed ({len(failures)} failures)", file=sys.stderr)
        return 1
    print(f"boundary mutation suite passed ({len(cases)} rejected mutations)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
