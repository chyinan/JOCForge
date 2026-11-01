# F06 BSI owner +0x34 zero default

## Scope

This supplement binds the constructor-time default of the typed active BSI owner field at owner-relative `+0x34` for the traced allocation and descriptor path. It does not define the field's later per-run value or map it numerically to standard E-AC-3 header DialNorm.

## Bound construction path

The successful context-size query returns a block that is zeroed before the context constructor runs. The BSI descriptor is installed at context `+0x3948` and its pointer at context `+0x828`. The descriptor initializer obtains the owner at descriptor `+0x138` through 32-byte alignment. The owner factory clears 200 bytes from that owner base; the typed owner initializer then stores a zero dword at owner `+0x34`. The field is typed as signed U16, so its constructor-time default before conditional updates is zero.

The sealed full-run pointer observation matches the static context/descriptor/owner offsets and matches the same owner token at the serializer parameter. That parent package binds one final EC3 FileWriter frame from the same owner generation; its observed header U5 was 17. The current-run BSI U16 was not reread. These observations do not establish numeric equality or a transform between the owner U16 and header U5.

## Limits

Alternate writers across all dependencies and lifetimes were not exhaustively excluded. The owner U16 to standard E-AC-3 DialNorm mapping remains unbound. The bounded same-offset DDP candidate constructor was not hit; a same-offset single-pass candidate is a different typed AudioInputToAudioSource object. No PCM or EC3 body bytes were read, no key/license contents were read, and no new encode run occurred.
