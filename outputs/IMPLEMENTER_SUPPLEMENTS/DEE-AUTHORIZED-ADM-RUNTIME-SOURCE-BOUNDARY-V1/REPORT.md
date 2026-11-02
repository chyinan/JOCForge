# Authorized ADM to DEE runtime input boundary

## Same authorized source identity

The hash-bound runtime receipt identifies an authorized 38-channel ADM BWF
fixture by SHA-256
`7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`.
DEE received an ASCII alias, `input_adm.wav`, with the same digest. The
current JOCForge full-run receipt records the same input digest and output
SHA-256 `d9680298ac1d913f408b77a72b45171fc379dabb71de80e44c24d27cbf1de6da`
(17,395,712 bytes). This establishes source-file identity for the two runs;
it does not establish equal audio or segment alignment.

## Bounded DEE reference

The DEE receipt covers `file_position` 2160..2208 and records a 2.048-second
MP4/M4A containing E-AC-3, 48 kHz, six channels (`5.1(side)`), at
1,024,000 bit/s. Its output SHA-256 is
`99e901e7ca836692ef3168a121a19c69fd67c49b3c1d1108efac185a1b50e671`.
This is a DEE reference, not JOCForge output. It is not a full-song reference;
the mapping from this file-position window to source samples/time and to a
matching JOCForge range remains `UNBOUND`.

## Runtime input edge

Read-only WinDbg observation during the normal DEE run joined the same
`OutputWrapper` instance at `StageRunner::Build`'s `combineSources` call and
later at input-property registration. `StageRunner::InitOutput` set
`dee_audio_input_source` to `input_adm.wav`. Static control flow shows that
`combineSources` builds a source registry from two source-category
containers, while the input-property handler attaches the path through a
per-source property map; the path setter does not create that registry.

Thus the authorized ADM identity is joined to the runtime input-property
edge and the bounded DEE reference run. The producer of the two source
containers passed to `combineSources` remains `UNBOUND` between job metadata
and upstream stage containers.

## Next useful comparison and limits

The reference can support a same-input JOCForge audio comparison after the
source window is mapped. No decoded PCM was persisted or compared in this
Wash pass, so audio parity is `NOT_RUN`. No media, encoded output, native
binary, raw addresses, decompilation, private key, or license material is
included. The source package's manifest and listed payload hashes were
verified; it did not provide a separate `package_seal.json`.
