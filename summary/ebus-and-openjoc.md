# JOCForge investigation summary

## EBU public ADM ecosystem

- `https://github.com/ebu/libadm` — current `master` inspected at commit `aed1ac5aea2844da374712a2b51339a09485df6f`; Apache-2.0; C++11; CMake; Boost 1.57+ header components. Upstream CMake declares version `0.14.0`. It models ITU-R BS.2076 XML and exposes ADM elements/references, but documents incomplete sub-element coverage and no S-ADM.
- `https://github.com/ebu/libbw64` — current `master` inspected at commit `d3dae5c74deada286994c8a4c2a03edd82d65589`; Apache-2.0; C++ header-only; CMake; no runtime dependencies. Upstream CMake declares version `0.10.0`. It reads/writes BW64/RF64, `ds64`, `axml`, `chna`, and integer PCM.
- `https://github.com/ebu/libear` — current `master` inspected at commit `2db69f8fcea0bc5db8a78e14a9c2ae6ed4283c15`; Apache-2.0; C++14; CMake; Boost 1.57+ and recursive submodules. It is a BS.2127 ADM render core, not an encoder.

## Integration decision

The current host has Rust and FFmpeg but no CMake/compiler/Boost toolchain exposed in PATH. Full C++ FFI would add a disproportionate overnight build risk. JOCForge therefore keeps the three EBU projects as public semantic/reference authorities and uses a narrow Rust BW64/axml/chna subset for the controlled fixture. Migration to libadm/libbw64 interop remains an explicit follow-up. libear is reserved for an independent render comparison when the host supports it.

## OpenJOC boundary

OpenJOC v0.15.0 is used only as an independently built external validator, decoder, and renderer. Its existing public tests and summaries were consulted for ETSI syntax cross-checks. JOCForge production code does not depend on OpenJOC crates or decoder internals.

## Supplied real file observation

`D:\ADM BWF\打上花火.wav` is the supplied Logic Pro ADM-BWF export. Its recorded
physical size is `1,583,325,474` bytes and its SHA-256 is
`adfee1efadd5d331497ec55a1c94b89c7f1a717ad959d6d5c726efb0b64e81aa`. It is a
RIFF/WAVE 38-channel, 48 kHz, 24-bit PCM file with `JUNK`, `fmt `, `data`,
`axml`, `chna`, and `dbmd`; the metadata chunks occur after the large `data`
chunk. The seek-aware scanner handles that ordering and the narrow model
census counts 38 tracks, 38 TrackUIDs, 29 authored objects (28 `Objects`, 1
`DirectSpeakers`). JOCForge censuses it successfully but rejects encoding
before PCM loading because the current source-derived MVP admits at most 15
dynamic objects; no scene reduction is performed.
