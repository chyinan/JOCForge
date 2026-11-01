# Helper time normalization (supersedes v1 final-writer statement)

On master open, the helper reads binary64 start_time and absolute FFOA from the same source-info object. It stores start_time on the master; relative FFOA is -1 when the absolute value is negative, otherwise max(0, absolute FFOA - start_time). The helper reports offset=start_time and ffoa=start_time+max(relative,0). Master defaults are start_time=0 and relative FFOA=-1; missing output lines leave host result slots at zero.

Three signed32 time components are converted to seconds as c0*3600+c1*60+c2/100000, but that is an intermediate relative-FFOA write. The later source-info getter normalization overwrites it.

For the full-hash sample-6 input, the prior native receipt records offset=FFOA=3610 seconds. This yields relative start sample 0 under the selected file-position equation. The start_time writer source and the helper image loaded by that historical run remain unbound; do not generalize AXML start as a universal resolver.
