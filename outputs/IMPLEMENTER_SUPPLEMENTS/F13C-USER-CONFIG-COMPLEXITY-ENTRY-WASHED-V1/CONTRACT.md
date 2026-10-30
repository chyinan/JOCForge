# F13C user-configuration to complexity entry boundary

## Bounded native/static findings

Static analysis found two separate native record flows: a count-like output is formed from one pair of imported fields, while the BSI complexity byte is copied from a different configuration-record byte. Their owner/generation relation is not established.

The DDP Atmos component statically registers a `data_rate` task-config field. No typed edge has been established from that field to either native record flow. The inspected job template contains `data_rate`, but does not contain explicit parameter-13, complexity, or BSI-count fields. The template is not runtime-validated.

DEE `--schema` produced partial XSD after the DDP plugin DLL initialization failed; the process exited with an access-violation code. Therefore the missing XSD entries do not define a task-configuration rejection boundary and do not prove a field is globally absent.

## Open evidence

The controlled native pair is `NOT_RUN`. Prior parameter-13 values 1 and 16 are rehashed observations, not a controlled same-input pair. Prior count-like values and model ID14 `object_count + 1` observations remain separate; no owner/generation join connects them to the complexity byte. The task/XML/UI field to native complexity byte remains `UNBOUND`.

No DLL, decompiled source, PCM, bitstream, private key, or absolute local path is included in this supplement. The next useful observation must use a valid accepted task configuration and capture the same-run task field, native configuration value, BSI byte, owner identity, and execution status.
