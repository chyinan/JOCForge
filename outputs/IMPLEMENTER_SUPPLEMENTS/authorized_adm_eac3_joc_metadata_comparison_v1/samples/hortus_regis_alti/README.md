# Hortus Regis Alti.wav — metadata-only receipt

Disposition: `ACTIVE_WINDOW_CONFIRMED_BY_BOUNDED_SOURCE_PCM_AND_DEE_METER`. This package contains only the sample receipt, ffprobe/OpenJOC JSON metadata, bounded source-window measurements when applicable, a file manifest, and a verifier. It contains no WAV, EC3, PCM, raw logs, configuration XML, or license material.

The initial R138 window mapped in that run to source frames 0..49999 and was near-silent; it is excluded. The selected active config tokens 86664..86688 mapped in that run to source frames 48000..97999, measured at -50.16 dBFS RMS / -25.17 dBFS peak. This mapping is scoped to this input and run, not a product timebase policy.

The OpenJOC classification is based on in-band syntax. FFprobe's Atmos profile is recorded separately. ETSI strict validation failed 33/33 access units while deployed compatibility passed 33/33; rendering was not verified. No PCM decode or playback was performed.
