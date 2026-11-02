# Sample 6 fresh DEE reference gate

The current JOCForge full-track output is real and independently validated:
FFprobe parses it as 48 kHz, 1024 kb/s, 5.1(side) E-AC-3 JOC, and FFmpeg
decode-only completes with exit 0. The output contains 4,247 access units and
is 135.904 seconds. No PCM was saved.

A fresh same-input DEE audio reference was not generated. The local DEE 2.3
guide documents automatic product-license discovery from the application
folder or `DEE_PLUGIN_PATH`; omitting a license argument therefore does not
prove no key/license will be opened. No DEE process was launched and no
license/key path was passed or read. Consequently PCM/audio-essence and BSI
comparison against fresh DEE remain NOT_RUN, not PASS.

Current JOCForge observations: U5 dialnorm is 17 while an existing same-input
DEE receipt reports U5=21; the Rust selected-LUFS input and the DEE scalar unit
are absent/unbound, so the root cause is unresolved. The BSI downmix fields
reported by Wash are all zero, but the local FFprobe does not expose their
numeric members and fresh DEE values are unavailable. No old Python prototype
or historical control-group payload was used. The private-key boundary is
preserved.
