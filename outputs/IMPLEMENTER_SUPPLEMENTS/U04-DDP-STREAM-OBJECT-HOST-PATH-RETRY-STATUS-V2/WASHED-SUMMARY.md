# DDP host-path retry status update

Disposition: RETRY_FAILURE_METADATA_ONLY. This adds launch-attempt boundaries to the prior washed v8 static host-path package. It contains no successful native frame observation.

Source v8 package manifest SHA-256: 49fe7373af9ec66d7aebfab9641d18ba8b3d96e5b4168ffa7d50cad7bb92d1d5.
Source EVIDENCE.json SHA-256: 2cd9f2ba300d043e0618ca8869e2f66acc45ed3fefbe2fcea0205b749a6786e8.
Normal and optimized source checkers and all listed payload hashes passed.

Run 12 replayed the approved 2160..2208 AtmosMezz/1024kbps/EC-3 short-job template with only temporary and output paths changed. Source, executable, and DDP component hashes matched the earlier successful run; the DDP family still failed initialization before measurement.
Run 13 used legacy XML without misc/temp_dir and failed before measurement.
Run 14 added misc/temp_dir and used the full D-drive test installation; DDP initialization still failed with workflow error Unknown filter. No DDP frame hook ran and no reference output was produced.
Available-memory evidence did not support the memory-pressure hypothesis.

These retries do not alter the v8 static chain: AtmosMezz storage provider creates a stream_object; DEE RunInput places it in a StageRunner input node; RunAudioFilter passes it to DDP. The storage producer's PCM pointer table and its ADM UID/CHNA identity remain unbound. The DDP local slot to frame channel index is a static non-mode-1 edge; runtime branch and values are not captured. Selector columns 11..47 map to global IDs only in a separate same-input reference job, not the current job's DDP invocation.

Status remains: new native frame capture NOT_CAPTURED; Gate1 false; U04, Type10, six-plane, and full output NOT_RUN. This is launch diagnosis, not a positive observation or product acceptance.
