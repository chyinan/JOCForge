# DDP component initialization common gate

Four hash-bound component images share a broad PROCESS_ATTACH pattern: per-image cookie setup, a module-local CRT dispatcher, CRT startup and an initializer table, then the component callback. The selected component callbacks return success in the inspected static view. The CRT initializer gate can still make the attach path fail before that callback.

Each image has its own table and function pointers. The bounded static comparison identifies a runtime-state allocation initializer with a failure sentinel after two failed allocation attempts, a CPU-feature initializer, a multibyte-helper failure predicate, and one unresolved table slot. These are possible failure sites only; no runtime event identifies which initializer failed.

Attempt-12's 44 host-consumed error IDs were all 1114, which classifies the error as DLL initialization failure. It does not identify the failing component or distinguish the requested component from a dependency. The closest common static boundary is the CRT attach initializer gate; the actual initializer remains NOT_IDENTIFIED. No new DEE run or payload/key access occurred.
