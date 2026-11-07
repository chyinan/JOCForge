# EmpireLoudness subcontext +0x3C initializer

Within the same successful subcontext construction, the Bool-configured field +0x164 is set on its conditional mode branch, while the Float32 source field +0x3C is initialized separately to -120.0. The constructor does not derive +0x3C from +0x164.

The already bound result getter reads +0x3C and copies it into its result block on the active engine branch. This slice binds only the constructor's initial value; it does not inspect later writers or establish the final runtime value. Bool-selected +0x164 to +0x3C/result causality remains UNBOUND.

This is a bounded static edge, not a global absence claim. No DEE run, runtime capture, media, PCM, payload, key, or license access occurred.
