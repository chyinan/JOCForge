# Short-block closure

ETSI TS 102 366 Table E.1.3 defines `numblkscod` values for 1, 2, 3, and 6
audio blocks per syncframe. A General E-AC-3 JOC programme interval must
accumulate six blocks, or 1,536 samples, across the selected I0 and dependent
programme sets. Short independent frames use `convsync=1` at the start of the
group and `convsync=0` for subsequent frames.

JOCForge exposes this contract through the pure `resolve_short_block_plan`
API and the centralized `JocEncodingPlan.block_partition` field. The normal
writer closes the four canonical raw families: `6`, `3+3`, `2+2+2`, and
`1+1+1+1+1+1`. Each returns the corresponding block codes, exact sample
ranges, and `convsync` sequence. The lower-level planner can still inspect
other six-block compositions, but mixed compositions are classified as
`PUBLIC_AMBIGUOUS` and are not enabled by the normal production plan until a
normative grouping rule is closed. Sequential D0..D7 topology remains a
separate contract through `resolve_dependent_topology`.

The advanced `conformance-short` command remains a synthetic syntax fixture.
The normal `encode` command accepts `--block-partition` for the four canonical
families and writes source-derived raw E-AC-3 short syncframes from the actual
1536-sample carrier interval. PCM sample accounting is emitted beside the
`.ec3` output as `.sample-accounting.csv`; the group remains exactly six
blocks/1536 samples and only the final D0 carries JOC/OAMD EMDF. The default
remains `6`. Short framing is raw elementary-stream capability only; CMAF
continues to require six-block samples.
