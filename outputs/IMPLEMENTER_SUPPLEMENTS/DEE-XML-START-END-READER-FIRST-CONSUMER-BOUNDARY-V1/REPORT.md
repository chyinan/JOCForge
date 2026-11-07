# XML start/end reader boundary

The checked XSD declares start and end as optional strings, with schema defaults first_frame_of_action and end_of_file. A sanitized R138 document receipt contains start=86640 and end=86664 text. Neither fact identifies the values selected by a runtime job. Omitted values remain unbound at runtime.

The selected host route only observed source/target property keys; it did not observe start or end keys. A separate file-position/embedded-timecode range-record path reaches a Reader cursor/total comparison, but it is not identified as the XML start/end path. The XML leaves and omission state have no typed edge to a concrete host key/value consumer, and no start/end-to-reader-position join is established.

Conclusion: selected values, omission state, owner/generation, and effective reader position remain UNBOUND. Runtime and new Ghidra work were NOT_RUN. No media, PCM, payload, key, or license was read.
