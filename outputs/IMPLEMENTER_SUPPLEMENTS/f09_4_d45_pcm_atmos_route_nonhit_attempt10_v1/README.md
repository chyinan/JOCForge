# F09.4 D45 route reachability boundary, attempt 10

Attempt 10 changed only the DEE filter stage from encode_to_atmos_ddp to pcm_to_atmos_ddp. It kept the authorized ADM input, EC3 output route, and 1024 kbps rate; the candidate XML passed the generated DEE schema. DEE completed with exit code 0 and output progress 100%, but the target owner/history routines did not hit.

The task-local EC3 reached 17,395,712 bytes before cleanup and was removed after metadata-only checks. Its contents were not read or parsed, and no playback occurred. This is a negative reachability observation for this one route/configuration, not proof that the target routines are unreachable or that the output is valid.

The six normal-sink jobs provide evidence that NUL sink is not established as the cause of the earlier unknown exit. The D45 cold-zero state remains supported by the separate static package; later nonzero history remains open.
