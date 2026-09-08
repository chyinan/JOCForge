# Interoperability census QA note

This note preserves the historical test-run sequence for the publication
package.

| Run | Result |
|---|---|
| Initial complete workspace test run | One transient failure was observed. |
| Targeted rerun | PASS. |
| Subsequent full workspace suite rerun | PASS. |
| Clean census reruns | Reproducible normalized results and hashes. |

The transient failure is historical QA context, not a current unresolved
failure and not part of the interoperability result counts. The final census
baseline remains 20 positive probes, OpenJOC 20/20, FFmpeg 14 FULL plus 6
`PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED`, and MediaInfo 1 JOC plus 19
`EAC3_ONLY_FALSE_NEGATIVE`.

The public reproduction path is
[`scripts/run-interop-census.ps1`](../scripts/run-interop-census.ps1). It uses
project-owned synthetic fixtures and does not require publishing copyrighted
source media.
