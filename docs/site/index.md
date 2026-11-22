# JOCForge

Encode 48 kHz ADM BWF masters into E-AC-3 JOC soundtracks. The independent Rust encoder reads beds, moving objects and their positions to produce audio carriers, reconstruction coefficients and spatial metadata.

[Quick start](quick-start.md){ .md-button .md-button--primary }
[View source](https://github.com/chyinan/JOCForge/tree/experimental/independent-encoder){ .md-button }

<div class="grid cards" markdown>

-   :material-speaker-multiple: **[Layouts and objects](layouts.md)**

    Choose a layout, bitrate and coded object count.

-   :material-waveform: **[Substreams and signalling](substreams.md)**

    Assign LFE, split blocks and select the EMDF carriage.

-   :material-file-music: **[Output containers](containers.md)**

    Write CMAF fragments or remux EC3 into MKA or MP4.

-   :material-tune: **[XML jobs and loudness](jobs.md)**

    Set render times, silence, DRC and loudness metering.

</div>
