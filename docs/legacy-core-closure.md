# Legacy-core closure

The advanced `legacy-ac3` carriage packages one public 48 kHz, six-block,
CRC-valid original-syntax AC-3 I0 (`bsid` 6 or 8) with one E-AC-3 D0 carrying
the strict OAMD/JOC EMDF. The current closed cells are public idx0..idx4 at
1280 kbps: 640 kbps AC-3 I0 plus 640 kbps E-AC-3 D0.

Evidence in `artifacts/legacy-core-closure/validation.json` and the CLI
regression run shows pinned OpenJOC strict inspect, OpenJOC 7.1.4 render, and
FFmpeg decode all pass for the synthetic raw elementary stream.

Other profiles, LFE-present legacy cores, lower/higher rates, CMAF packaging,
and multiple dependents remain fail-closed until their exact public topology
and independent decoder contract are closed.
