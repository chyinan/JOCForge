# OpenJOC validator provenance

JOCForge uses one external validator identity for strict inspection and
rendering. The machine-readable pin is
`validation/openjoc-validator.json`; `scripts/openjoc-validator-pin.ps1`
hard-fails before external validation when either the executable version or
SHA256 differs.

The original mutable artifact had SHA256
`b4fb4593be8dfe3bbeceb4ba4e3eb9c97f89ebe043555e8af615cb4eea289bee` and was
unavailable after the active development target was overwritten. The migrated
frozen artifact is OpenJOC semantic version `0.16.0`, built from source
revision `ad6556babf42566f1a09820b01dc333703c8b1da`, with SHA256
`7e761d0c8a23a064c96c1f70f6415e1d95f18b8b88d72eca0bfa6b52d8ee2daf` on
Windows x86-64. Its immutable manifest and executable are provided through
the `JOCFORGE_OPENJOC_MANIFEST` and `JOCFORGE_OPENJOC_EXECUTABLE` environment
variables declared in `validation/openjoc-validator.json`. OpenJOC remains an
external validator/decoder/renderer, not a JOCForge implementation dependency
or the sole evidence authority.

The repository does not redistribute the validator binary. Before running a
script that needs it, provision the read-only pinned artifact and set:

```powershell
$env:JOCFORGE_OPENJOC_MANIFEST = "C:\\path\\to\\validator-manifest.json"
$env:JOCFORGE_OPENJOC_EXECUTABLE = "C:\\path\\to\\openjoc.exe"
```

## Release CI provisioning

The Windows release workflow builds the bundle on GitHub's hosted Windows
runner from an exact source tag. When the frozen validator is unavailable on
that runner, dispatch with `publish=false` and download the successful run's
complete artifact. On a maintainer's Windows host with the two environment
variables above, PowerShell 7, and Python with NumPy, run:

```powershell
pwsh -NoProfile -File scripts/release/smoke-windows-bundle.ps1 `
  -ZipPath "C:\path\to\release\JOCForge-vVERSION-win-x64.zip" `
  -ReportPath "C:\path\to\release\release-smoke.json" `
  -ReleaseArtifactPath "C:\path\to\release\release-artifact.json" `
  -OpenJocPath $env:JOCFORGE_OPENJOC_EXECUTABLE -RequireOpenJoc
```

Keep the downloaded ZIP unchanged. This verifies its hashes and validator pin,
runs the full extracted-bundle semantic gate, and updates the external evidence
files. Require passing smoke, semantic, and license evidence before uploading
the ZIP, the outer `SHA256SUMS.txt` (including the ZIP digest), the updated
reports, and the FFmpeg source archive to a draft release. Verify those uploaded
assets before publishing with `gh`. Record that semantic QA ran on the
maintainer host, separately from the hosted build.

If the frozen validator is provisioned on the CI runner, `publish=true` retains
the all-in-CI semantic gate and upload path. Neither path redistributes the
validator binary.
