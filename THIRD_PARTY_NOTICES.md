# Third-party notices

The source repository contains no third-party runtime libraries. Cargo's
workspace has only the two local JOCForge packages and no registry, build, or
development dependencies.

The official Windows release bundle contains FFmpeg and ffprobe as separate
executables. They are not JOCForge source and are not licensed under
Apache-2.0. The release bundle includes the exact FFmpeg license files, source
archive, source hash, build configuration, and binary hashes under
`licenses/FFmpeg/`. The effective configuration uses LGPL-2.1-or-later FFmpeg
code without GPL, nonfree, or version-3 components, with the required
MinGW-w64 winpthreads runtime statically linked into the executables. No
`libwinpthread-1.dll` is required or distributed. The bundle includes the
winpthreads MIT/BSD-3-Clause-Clear license text as
`licenses/FFmpeg/WINPTHREADS-COPYING.txt`.

The FFmpeg pin and configuration are tracked in
[`release/ffmpeg-pin.json`](release/ffmpeg-pin.json). OpenJOC is a separately
provisioned validation tool and is not included in the JOCForge release bundle.
