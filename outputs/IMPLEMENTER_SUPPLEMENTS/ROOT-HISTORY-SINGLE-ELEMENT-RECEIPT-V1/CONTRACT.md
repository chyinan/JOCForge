# Root-history scalar stop result

A hash-bound four-point receipt follows two non-payload root-history elements through one higher invocation: before shift, after shift, writer-stage entry, and the post-serializer common record. Both are 32-bit control/history scalars. Their values remain zero at all observed points; the common-record fields equal the root's current values.

Static owner flow shifts each next value into current before writer and copies current heads into the common record after serializer. The serializer-consumer stage does not write either scalar. These values therefore do not carry consumer-output identity and cannot close consumer_N to writer_N+1. Stop enumeration of adjacent root-history scalars.

The common record is not established as an access unit. Transport-AU and source-generation identities remain UNBOUND. The next useful target is a real payload or descriptor receipt at the post-consumer contiguous destination and its downstream queue/transport consumer.
