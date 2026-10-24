# Selected coefficient representation retry

The retry stage consumes one completed fit result and reuses the same clip, sequence, offsets and coefficient state for every attempt. It never reruns receiver, temporal, geometry, solve or score.

Attempts follow the approved order: fine to coarse for every eligible object, full-coarse to sparse-coarse using the same-generation stage0 backup and selected channel, then one lower-band step at a time using the identity-bound functional provider map with the required in-place row reads. Writer and coefficient scratch failures remain attempt-local.

The selected product capacity stays 372 bytes. The 80-byte SYNTHETIC-QMF-FIT-CHECKPOINTS helper exercises retry only: generation0 succeeds after fine→coarse→sparse at 74 bytes; generations1/2 require one additional enum5→4 band step and succeed at 72/73 bytes. Every attempted required length, mode, band enum and final payload matches the assertion-only checkpoints.

The retry result owns no queue or sequence counter. The enclosing source parent must stage the successful immutable body and commit fit/sequence/metadata state once.
