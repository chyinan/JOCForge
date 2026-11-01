# DDP Atmos task route and Runner storage

Run145 and Run148 XML use the exact filter/audio route encode_to_atmos_ddp. The StageRunner Build path compares the requested route with callback-created config names by exact string and length, calls the matching config factory, then stores the returned stage at Runner +0x270 and the matched config at Runner +0x278. The selected DDP Atmos config is local callback slot 0; its factory constructs the DDP JOC stage.

Run148 reused existing receipts: the DDP image hash matches the static image and host/DDP entries share a thread. The direct factory-entry breakpoint did not hit. This binds static selection and stores, not a dynamic callback-entry hit or global ordinal.

The input SHA is reused from existing receipts; no new run or rehash occurred. The XML-to-Runner member edge and owner/generation links remain open. No PCM, payload, bitstream, or private key was read.
