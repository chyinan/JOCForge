# pattern: Imperative Shell
"""Mutation tests for repository boundary gates."""
from __future__ import annotations

import importlib.util
import pathlib
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable


ROOT = pathlib.Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts" / "check-boundaries.py"
SPEC = importlib.util.spec_from_file_location("jocforge_boundary_checker", CHECKER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("failed to load boundary checker")
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def replace(path: pathlib.Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"mutation source not found in {path}: {old}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def append(path: pathlib.Path, text: str) -> None:
    with path.open("a", encoding="utf-8") as stream:
        stream.write(text)


def expect_failure(name: str, mutate: Callable[[pathlib.Path], None]) -> None:
    with tempfile.TemporaryDirectory(prefix="jocforge-boundary-") as temp:
        clone = pathlib.Path(temp) / "repo"
        shutil.copytree(ROOT, clone, ignore=shutil.ignore_patterns(".git", "target", "__pycache__"))
        mutate(clone)
        try:
            CHECKER.run_checks(clone, CHECKER.PACKAGE)
        except CHECKER.BoundaryError:
            print(f"PASS mutation rejected: {name}")
            return
        raise RuntimeError(f"mutation unexpectedly passed: {name}")


def main() -> int:
    CHECKER.run_checks(ROOT, CHECKER.PACKAGE)
    cases: list[tuple[str, Callable[[pathlib.Path], None]]] = [
        ("missing architecture document", lambda root: (root / "docs/architecture/PUBLIC-API.md").unlink()),
        ("missing mapping table", lambda root: (root / "docs/implementation/VECTOR-MAP.md").unlink()),
        ("missing functional ID", lambda root: replace(root / "docs/implementation/SPEC-TRACEABILITY.md", "| STRC |", "| REMOVED_STRC |")),
        ("duplicate functional ID", lambda root: replace(root / "docs/implementation/SPEC-TRACEABILITY.md", "| GTC |", "| STRC |")),
        ("extra functional ID", lambda root: replace(root / "docs/implementation/SPEC-TRACEABILITY.md", "| OWNJ |", "| EXTRA | `START-HERE.md` | `x` | `t` | `programme_root` | `ProgrammeAttempt` | NO_RUNTIME_PARAMETER | — | NO_DIRECT_VECTOR | — | — | — | Phase 1 |\n| OWNJ |")),
        ("bad spec path", lambda root: replace(root / "docs/implementation/SPEC-TRACEABILITY.md", "`test-vectors/STATUS-RECONCILIATION-LOCAL-REVIEW-STATUS.json`", "`specs/does-not-exist.md`")),
        ("bad provider path", lambda root: replace(root / "docs/implementation/PARAMETER-MAP.md", "`parameters/adm-installed48-state-v1.json`", "`parameters/does-not-exist.json`")),
        ("bad vector path", lambda root: replace(root / "docs/implementation/VECTOR-MAP.md", "`test-vectors/unit/vectors.json`", "`test-vectors/does-not-exist.json`")),
        ("registry used as FILE_PROVIDER", lambda root: replace(root / "docs/implementation/PARAMETER-MAP.md", "| STRC | NO_RUNTIME_PARAMETER | — |", "| STRC | FILE_PROVIDER | `PARAMETER-INTERFACES.json` |")),
        ("coverage ledger used as DIRECT_VECTOR", lambda root: replace(root / "docs/implementation/VECTOR-MAP.md", "| STRC | NO_DIRECT_VECTOR | — |", "| STRC | DIRECT_VECTOR | `test-vectors/PRE-HANDOFF-EVIDENCE-COVERAGE.json` |")),
        ("owner map mismatch", lambda root: replace(root / "docs/implementation/OWNER-MAP.md", "| F01 | `spatial_control` |", "| F01 | `joc` |")),
        ("programme owner root missing", lambda root: replace(root / "docs/architecture/STATE-OWNERSHIP.md", "| `reader_eof` |", "| `reader_eof_removed` |")),
        ("clock domain mismatch", lambda root: replace(root / "docs/implementation/CLOCK-MAP.md", "| F02 | `AdmSampleIndex`<br>`FrontendCallbackIndex` |", "| F02 | `PublishedAuIndex` |")),
        ("multiline public usize", lambda root: append(root / "crates/jocforge-domain/src/state/mod.rs", "\npub fn exposed_clock(\n    value: usize,\n) { let _ = value; }\n")),
        ("async implementation in empty leaf", lambda root: append(root / "crates/jocforge-domain/src/codec/bap.rs", "\npub async fn forbidden() {}\n")),
        ("production depends on validation", lambda root: replace(root / "crates/jocforge-application/Cargo.toml", "[dependencies]\n", "[dependencies]\njocforge-validation = { path = \"../jocforge-validation\" }\n")),
        ("production process and FFmpeg", lambda root: append(root / "crates/jocforge-application/src/lib.rs", "\npub fn forbidden_process() { let _ = std::process::Command::new(\"FFmpeg\"); }\n")),
        ("legacy compatibility fallback", lambda root: append(root / "crates/jocforge-application/src/lib.rs", "\n// legacy compat fallback\n")),
    ]
    for name, mutate in cases:
        expect_failure(name, mutate)

    optimized = subprocess.run(
        [sys.executable, "-O", str(CHECKER_PATH)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if optimized.returncode != 0:
        raise RuntimeError(f"optimized baseline failed: {optimized.stderr or optimized.stdout}")
    print("PASS optimized mode executes real gates")
    print(f"boundary mutation tests passed ({len(cases)} rejected mutations plus optimized baseline)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
