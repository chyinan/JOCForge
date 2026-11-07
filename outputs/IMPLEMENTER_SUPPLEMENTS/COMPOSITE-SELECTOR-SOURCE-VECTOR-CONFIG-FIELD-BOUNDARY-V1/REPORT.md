# Composite source-vector configuration-field boundary

## Bound static producer

On the inspected mode-two per-item route, a host materializer takes a queue/deque record pointer and copies a typed vector header from record+0x08 into a temporary source-configuration object. The vector header contains U64 begin/end/capacity pointers and 32-byte source-entry rows. The temporary object is forwarded through the mode-two wrapper to the per-item consumer; the same vector pointer then reaches the reader builder for each indexed row.

This binds the source vector to a typed queue-record field at +0x08. The queue record's symbolic job/class owner, generation, actual entry count, path bytes, and selected runtime branch remain unresolved or uncaptured.

## First break

The first unresolved identity edge is from the queue/deque record pointer to its semantic source-configuration owner and generation. The static 1536 U32 argument on this route is a per-item frame-sample count, not a vector owner or generation token. The actual nested path suffix remains unknown, so storage-handler selection and K-count applicability are still unbound.

The upstream manifest SHA-256 is 347b2b29af855b2736fd8ece036df0997db43e8ed12db2e1e43ff1d1194bc2c0. Its normal and python -O checker passed. No actual source path, media, PCM, payload, key, or license was read.
