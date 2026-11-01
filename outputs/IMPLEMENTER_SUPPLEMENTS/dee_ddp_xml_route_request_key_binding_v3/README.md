# DDP task XML route to StageRunner request key

Run145 and Run148 XML contain one filter/audio child whose route text is encode_to_atmos_ddp. The hash-bound Ghidra trace follows SplitProperties through the resolved audio node into a copied child-vector stored at StageRunner +0x68. The first copied child row name at vector begin +0x18 is the route key read by Build. Build compares that string and exact length with callback-created config names, selects the DDP Atmos local config slot 0, calls its factory at vtable slot +0x30, then stores the returned stage at Runner +0x270 and matched config at Runner +0x278.

Run148 DDP image identity and host/DDP same-thread receipt are reused. No new DEE or WinDbg run occurred; the direct factory callback entry was not hit. This package closes the static XML child-vector to request-key source edge, not the global callback ordinal or selected descriptor owner/generation.
