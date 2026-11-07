# Rate writer child owner boundary

Within the bounded writer invocation, a U32 from the rate-conditioned nested record is passed to a generic setter with a separate child pointer. The setter stores that U32 at child +0x4 and returns.

The child pointer has no bound constructor, type/class token, or typed reader of +0x4 in this trace. A later call on another receiver field is a separate, unjoined object path. Therefore the child owner/type and any codec-configuration or BSI effect remain UNBOUND. This is not a whole-image absence claim. No DEE run or media/PCM/payload/key/license access occurred.
