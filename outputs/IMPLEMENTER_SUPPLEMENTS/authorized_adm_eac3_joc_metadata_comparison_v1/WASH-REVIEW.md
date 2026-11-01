# Wash review: authorized ADM multi-input E-AC-3 JOC metadata

Two different authorized ADM BWF inputs produced consistent metadata-only E-AC-3/JOC results on the hash-bound R138-derived route:

- Both outputs are raw E-AC-3, 48 kHz, 5.1(side), 1,024,000 bps, with OpenJOC in-band JOC signaling in all 33 parsed AUs, profile index 3 five_x_phase, OAMD object metadata (16 slots), JOC reconstruction rows 15, and complexity index 16.
- First Love's tested 24-token window is explicitly excluded as a valid active-audio sample: DEE measured -120 LUFS and zero sample/true peak.
- Hortus Regis Alti's initial window was near-silent. A bounded source scan found an active adjacent window; the one-run DEE log maps tokens 86664-86688 to source PCM frames 48000-97999 for that input only. The source-window RMS/peak and DEE integrated loudness are recorded in its receipt.

Both runs report ETSI strict EMDF failure and deployed-compatibility pass. This is a strict interoperability risk, not evidence that JOC is absent; compatibility pass does not prove general decoder interoperability. render_verified=false; no playback or full-track validation was performed. Run-time input SHA was not persisted in the run manifest; identity is through the hash-bound config/log/alias and source-copy records. No PCM or encoded payload is present in this package.
