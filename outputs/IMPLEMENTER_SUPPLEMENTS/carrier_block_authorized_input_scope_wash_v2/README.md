# Carrier block authorized input scope

Status: `SEVEN_AUTHORIZED_BWF_IDENTITIES_HASHED_AS_SEPARATE_LINEAGES`; the prior R24 native run remains a distinct input lineage and matches none of them.

This Wash-only v2 addendum records whole-file SHA-256 identities for the seven explicitly authorized ADM BWF inputs. Each file has its own `lineage_id`; the hashes were computed by streaming file bytes, and no audio data was decoded, modified, or copied into this package.

The previously sealed F13C R24 native capture used source-job hash `8fba0b22…7d27cba1f` and input-file hash `adfee1ef…64e81aa`. The identity crosswalk shows that this input hash matches none of the seven authorized BWF identities. R24's native selector, BSI, owner, and generation observations remain attached only to the historical R24 input lineage; no cross-sample native join is made.

One authorized file, `D:\人是猫 - 张卡斯, 洛天依.wav`, was used for an XSD-valid DEE setup attempt. The DDP-related filters failed to initialize before workflow construction, no output file was created, and native input processing remains `NOT_CONFIRMED` / `NOT_RUN`. This setup failure is not a no-hit observation.

The approved selected carrier scope remains the existing decision: fixed block 2 with a complete EMDF container up to 511 bytes; arbitrary placements and fragmentation are not required. `selected_carrier_blocker_revocable=true` is scoped to that decision. Whole F13 and product readiness remain unclaimed.

No unapproved input path or filename, raw audio, PCM, EC-3 payload, license/key content, or private source is included. See the manifest and checker for the sealed file set.
