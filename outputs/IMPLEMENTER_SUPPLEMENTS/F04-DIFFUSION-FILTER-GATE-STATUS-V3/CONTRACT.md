# F04 diffusion gate producer edge: V3 status

This is the latest bounded, read-only static status for the opaque filter-gate row. The approved F04 filter arithmetic and constructor rules remain closed for their declared typed inputs; this package addresses only the missing producer/selected-callback identity.

A reporter-attributed static chain carries an opaque row pointer from a callback-context member through a dispatcher and wrapper to the four-lane filter caller. The caller tests outer U32 word 1 == 0, then row U32 word 11 != 0, row F32 word 10 > +0.0, and row U32 word 0 == 0. A bounded writer search did not identify the unique owner/writer of that pointer or row words.

No typed mapping is established from the opaque row to audio_diffuse, width/spread, quiet, F04FM mode/lane, selected tail, or callback/source generation. The two upstream row-context builder paths were not disambiguated for the selected ADM callback. The frozen R19 projection accepts preconstructed actions/tail/route/PCM and is not the producer for these gates.

Keep producer and selected-callback edges UNBOUND/NO. Native execution, dynamic capture and full-song replay are NOT_RUN. No field names should be inferred from row shape or branch order.
