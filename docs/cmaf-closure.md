# CMAF closure

JOCForge now has an advanced pure CMAF writer for the constrained public
profile: one complete 1,536-sample I0+D0 sample, an `ec-3` sample entry, a
normative `dec3` descriptor, `cmfc`/`ceao` compatibility brands, and fragmented
`moof`/`mdat` samples. The CLI entry point is:

```powershell
cargo run -p jocforge -- conformance-cmaf idx1 INPUT.ec3 OUTPUT.mp4
```

The closure script validates idx0..idx4 with `ffprobe` and pinned OpenJOC
container decode. Legacy AC-3 cores, D1+, grouped short blocks, and LFE-present
CMAF cells remain fail-closed because Annex E imposes a separate one-I0/optional
D0, six-block container contract.
