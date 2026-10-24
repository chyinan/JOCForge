# Natural selected Type11 output length evidence

R23 provides two scoped natural-output observations; they support keeping the approved 160-byte public API limit for the sampled selected path, but do not prove a universal bound across all ADMs/profiles.

## Frozen model primary-metadata run

The frozen writer was freshly invoked from delivered control metadata and independent 580-byte DBMD metadata. It accounted for 4,251 frontend callbacks: the first two have no Type11 output; callbacks 2..4250 produced 4,249 natural Type11 payloads (4,245 normal outputs plus four cached drains). All 3,735 distinct metadata inputs generated a 69-byte payload; each fresh payload matched its archived expected bytes. The writer was rerun before expected payloads were opened. No audio export script, raw PCM, PCM hash or upstream PCM producer was used. The model queue was not rerun in this probe.

## Native bounded secondary run

A bounded 2.125-second selected 1024-kbps native run observed 69 natural Type11 outputs from 71 source invocations. All were 69 bytes. Producer, formatter descriptor, frontend input, collector descriptor and queue copy were linked within each observed native source generation; body equality is not used alone to establish identity. This is a secondary run, not a full-primary native encode.

## Distinct capacity layers

Keep these separate: actual natural output length 69 bytes in these samples; observed native producer buffer capacity 375 bytes; native ID11 queue per-descriptor capacity 2103 bytes from R21; approved public API payload limit 160 bytes from F13.4. Neither 375 nor 2103 replaces the public 160-byte policy. R23 finds no natural selected output above 160 in its sampled model and native scope. The R21 2103/2104 physical queue boundary remains valid.

## Limits

This does not prove every possible selected metadata input or profile emits at most 160 bytes. The model side uses frozen control metadata and does not rerun the upstream control/PCM producer; native coverage is bounded secondary only. The checker independently reproduces serialized length from inputs and validates identity receipts, but it does not regenerate all 4,249 Type11 payload bytes. Fresh payload re-encoding and archive equality are bound by the execution receipt and per-output observations. No PCM, key, authentication result or full-primary native output is included.

The 160-byte public API limit remains approved. Whole F13C and end-to-end product acceptance remain NOT_READY.
