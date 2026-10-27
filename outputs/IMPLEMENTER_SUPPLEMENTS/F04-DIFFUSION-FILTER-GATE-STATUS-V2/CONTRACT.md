# F04 diffusion filter gate status: v2 source rehash

Classification: HASH_BOUND STATIC-STATUS UPDATE. This v2 supersedes the prior Wash status package and does not add a DSP rule.

The approved F04F/F04FM/F04FCM formulas, parameter generation, sample precision/order, cold/reset rules, and four-lane filter contract remain closed for their declared typed inputs. The fresh Ghidra report traces an opaque row pointer from callback-context relative field +0xA0 through dispatcher/wrapper to the filter caller. That caller checks outer U32 word 1 == 0, then U32 word 11 != 0, F32 word 10 > +0.0, and U32 word 0 == 0 before the four-lane filter call.

Wash independently rehashed all five supplied source identities: four frozen R19 Python modules and the target DLL. The corrected F04G width-semantics manifest reference also matches the current washed manifest. Wash did not reproduce the Ghidra analysis. Treat the gate shape as reporter-attributed static evidence, not a normative field mapping.

A bounded writer search did not find a unique writer for callback-context +0xA0 or the pointed row words. The exact open edge is the producer/owner of +0xA0 and row words 0, 1, 10, 11 to typed audio_diffuse, width/spread domain, quiet, F04FM lane/output, selected tail and callback/source-generation receipt. Keep every semantic alias UNBOUND. The R19 Python A4 projection receives preconstructed actions/tail rows/routes/PCM and does not generate the typed gate. The 288 WAV-bound calls remain HOLD/UNBOUND.

No native dynamic capture or selected ADM generation was observed. This package contains no source code, decompilation, machine-code bytes, absolute addresses, PCM or key. It is informational only and does not justify a production gate or field alias.
