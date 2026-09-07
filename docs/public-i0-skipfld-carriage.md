# Public I0-only skip-field JOC carriage

Verified: 2026-09-07

JOCForge exposes carriage topology independently from EMDF signaling:

- carriage: `etsi` (I0 plus dependent D0) or `i0-skipfld` (I0-only);
- EMDF profile: `etsi` or `deployed`.

The default is ETSI strict: `etsi` carriage with the `etsi` EMDF profile. The
high-level `--compat deployed` preset selects `i0-skipfld` carriage and the
`deployed` EMDF profile together.

## Public closure

ETSI TS 102 366 V1.4.1 provides the complete carriage path:

- E.2.8.1 requires at least one independently decodable E-AC-3 stream and
  makes dependent streams optional. Therefore an Independent/0-only programme
  is legal at the E-AC-3 topology level.
- E.1.3.2.10 enables full skip-field syntax in every audio block.
- 4.4.3.58 through 4.4.3.60 define `skiple`, the 9-bit `skipl` byte count,
  and the skipped byte field.
- Annex H.1 explicitly permits an EMDF container in the skip fields at the end
  of an audio block, as well as in frame-end `auxdata`.
- Annex H.2.1/H.2.2 define the EMDF sync, container length, payload sizing and
  configuration syntax.

ETSI TS 103 420 V1.2.1 clauses 8.2 through 8.3 and Tables 40 through 41
require JOC metadata in EMDF, with payload IDs 11 (OAMD) and 14 (JOC), once per
frame. The `etsi` EMDF profile uses the published strict payload
configuration, including `codecdatae=1`. Payload IDs 1 and 2 are not needed
for JOC; TS 102 366 Table H.2.3 defines them as programme loudness and
programme-information payloads.

TS 103 420 V1.2.1 clause 8.3 defines the addbsi extension marker used here:
`flag_ec3_extension_type_a` followed by the 8-bit `complexity_index_type_a`.
Its value is bounded by and derived from the public object assignment. This is
the source of the two-byte `01:10` marker; it is not inferred from DEE bytes.

The public gate therefore passes without using DEE bytes or vendor-only fields:

```text
I0_ONLY_JOC_PUBLICLY_ALLOWED       = YES
SKIPFLD_PUBLICLY_DEFINED           = YES
EMDF_IN_SKIPFLD_PUBLICLY_JUSTIFIED = YES
JOC_IN_SKIPFLD_PUBLICLY_JUSTIFIED  = YES
UNKNOWN_REQUIRED_VENDOR_BYTES      = 0
```

DEE `codecdatae=0`, group-ID choices, payload IDs 1/2, alignment omissions, and
downmix index 3 are separate signaling observations. JOCForge copies none of
those unrelated choices. The `deployed` profile changes only `codecdatae` to 0
for payload IDs 11 and 14; this is an observed deployed compatibility
deviation, not an ETSI strict value.

## Implementation boundary

FFmpeg remains an external public E-AC-3 audio-core encoder. JOCForge does not
claim to be a complete PCM-to-E-AC-3 perceptual encoder in this change. The
JOCForge-owned path performs the following deterministic work:

1. FFmpeg produces a 5.1, 48 kHz, six-block Independent/0 core at 896 kbps.
2. JOCForge pads that legal frame to a nominal 4096-byte frame and recomputes
   CRC2. The final nominal stream bitrate is 1024 kbps.
3. JOCForge inserts a public two-byte JOC addbsi marker and the selected EMDF
   container in audio-block skip fields.
4. The output contains one I0 frame per AU and no D0 frame.

The specialized walker is fail-closed for the verified FFmpeg core syntax. It
counts public exponent, bit-allocation and mantissa codeword widths without
decoding or rewriting audio coefficients. It rejects insufficient skip-field
capacity instead of truncating EMDF or overflowing the frame.

## Validation evidence

The full Hanabi output was independently checked with pinned OpenJOC and
FFmpeg:

- 9,042 AUs; 9,042 Independent/0 frames; 4,096 bytes each;
- 0 dependent frames; frame-end `auxdatae` absent in all AUs;
- 9,042 skip-field EMDF containers; 0 malformed candidates;
- payload IDs 11 and 14 in all AUs; complexity index 16;
- OpenJOC strict profile accepted all 9,042 AUs for the strict EMDF mode;
- OpenJOC compatibility validation is the intended gate for the deployed
  signaling deviation;
- FFmpeg decoded the complete stream without errors;
- repeated full encodes are deterministic.

The I0 skip-field carriage is structurally similar to the observed DEE stream.
It is not byte compatible with DEE, does not copy DEE payloads, and makes no
perceptual-quality or official-Dolby claim.
