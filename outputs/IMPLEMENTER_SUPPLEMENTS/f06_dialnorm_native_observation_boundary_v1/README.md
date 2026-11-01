# F06 measurement to first-AU DialNorm observation

One R138-equivalent DEE reference job used the authorized ADM bytes through an ASCII alias. WinDbg observed the DDP Atmos module load in the same process, and its resolved image SHA-256 matched the expected target. DEE exited 0.

The native log reports integrated/measured/ITU loudness around -33.025146/-33.03/-33.03, but does not state units. OpenJOC read only seven bytes of the first independent E-AC-3 syncframe header and reported DialNorm U5=31. The temporary EC3 was recorded by size only, never read or hashed, and is absent now.

Under the approved F12 model formula, interpreting -33.03 as selected_lufs predicts 31 because the result is at the lower clamp boundary. This is one matching sample at a saturated boundary; it cannot establish native units, rounding, or default/update behavior. Ghidra connects the measurement result to a per-source record and separately connects owner U16 +0x34 to the BSI writer, but no typed same-owner/generation join connects them.

This is a Wash-only observation, not full output validation or proof of native formula parity. No private key, license contents, PCM, or payload body was accessed.
