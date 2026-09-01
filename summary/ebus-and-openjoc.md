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

`D:\ADM BWF\打上花火.wav` is readable and is 38-channel, 48 kHz, 24-bit PCM. Its RIFF chunk header currently exposes only `JUNK`, `fmt `, and `data`; no `axml` or `chna` chunk was found before `data`. It will be retained as a real PCM stress input, but JOCForge will not label it ADM/BW64 without those metadata chunks.
