# F09.4 D45 PCM-route reachability boundary

Two hash-bound, schema-valid DEE comparison jobs preserved the authorized ADM input and 1024 kbps while changing only the filter stage. Attempt 10 used pcm_to_atmos_ddp and completed code 0; all 25 numeric breakpoints resolved, but the monitored D45 owner/history targets did not hit. Attempt 11 used pcm_to_ddp and completed code 0; the single-pass DLL loaded, but the debugger did not stop on the module-load event, so target breakpoints were not armed.

Static Ghidra analysis shows a 1536-sample PCM-frame guard (zero is allowed for flush) before owner allocation. Two PCM frame adapters call this guard. The callback-registration-to-adapter-table edge remains UNBOUND_STATIC. These experiments do not establish route-wide unreachability or any nonzero D45 state.

Output files were checked only by metadata and deleted. No EC3 bytes were read or parsed; no playback occurred. Cold-zero remains separate static evidence; later nonzero reachability remains open.
