# pattern: Imperative Shell
"""Check the current scaffold's dependency and source isolation boundaries."""
import pathlib
import re
import tomllib

root = pathlib.Path(__file__).resolve().parents[1]
expected = {
    "domain": set(),
    "application": {"jocforge-domain"},
    "io": {"jocforge-domain"},
    "cli": {"jocforge-application"},
    "validation": {"jocforge-domain", "jocforge-application", "jocforge-io"},
}
workspace = tomllib.loads((root / "Cargo.toml").read_text())
assert set(workspace["workspace"]["members"]) == {
    f"crates/jocforge-{name}" for name in expected
}
assert {p.name for p in (root / "crates").iterdir()} == {
    f"jocforge-{name}" for name in expected
}
for name, dependencies in expected.items():
    crate = root / "crates" / f"jocforge-{name}"
    config = tomllib.loads((crate / "Cargo.toml").read_text())
    assert set(config.get("dependencies", {})) == dependencies, name
    for dependency, values in config.get("dependencies", {}).items():
        assert values == {"path": f"../{dependency}"}, dependency
    assert not config.get("features"), name
    assert not config.get("build-dependencies"), name
    assert not config.get("dev-dependencies"), name
    assert not config.get("target"), name
    assert not (crate / "build.rs").exists(), name
    for source in (crate / "src").rglob("*.rs"):
        text = source.read_text()
        assert not re.search(r'include!|#\[path|static\s+mut|legacy|ffmpeg|compat', text, re.I), source
        if name in {"domain", "application"}:
            assert not re.search(r'\b(fs|process|env|net|thread)\b', text), source
print("dependency and source boundary checks passed (5 crates; lexical guard only)")
