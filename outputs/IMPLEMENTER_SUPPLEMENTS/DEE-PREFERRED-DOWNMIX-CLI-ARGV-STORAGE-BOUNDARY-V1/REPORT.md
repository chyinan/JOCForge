# DEE preferred-downmix CLI argv-to-storage boundary

## Bound static entry path

The upstream Ghidra receipt follows the MSVC startup path from argc and wide argv through wide-to-narrow conversion to a pointer vector. At the app vtable +0x08 call, RCX is DdpjocEncoderApp, EDX is argc, and R8 is the converted argv vector. The resolved target is the app runner method, whose decompiled form takes only this and argc and has no read/forward edge for R8.

The option schema separately binds encoder to Boost typed_value<string> storage at DdpjocEncoderApp+0x360. The runner reads that field, uses ddpjoc: only if it is empty, and copies the string into runner configuration, where the composite parser handles it.

## First break

No static edge binds converted argv through Boost parse/store/notify to the +0x360 typed value. The app-entry dispatch at vtable +0x08 is the exact first break: R8 carries argv but the target body does not consume or forward it. Nearby option-state validation does not close that write. Actual child argv/runtime remains uncaptured; no claim is made that Boost storage is absent elsewhere.

The upstream manifest SHA-256 is b614131e246adc9950a73dd469d7a39d46fabba9596a6ed1da10e9401181134f. Its normal and python -O checker passed. The target image hash is from the parent project record; the installed image was not rehashed. No runtime, media, PCM, payload, key, or license content is included.
