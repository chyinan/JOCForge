# D-to-F12 metadata-owner capture preflight status

Status: `BLOCKED_PRECONDITIONS_UNVERIFIED; CAPTURE_NOT_READY`.

A Wash-only preflight reviewed the proposed one-invocation metadata receipt for connecting the worker descriptor/open configuration/profile wrapper to the validated F12 configuration-count owner and source/reset generation. It confirmed that no target was started, attached to, or probed. The proposed receipt schema is a design only; it does not implement or prove a no-write guard, one-shot stop, non-user fixture, or observer.

All four required preflight gates remain blocked:

- A target-wide file/protected-output sink gate for the exact replacement driver/job is not verified.
- A one-shot fail-closed stop is not armed before target execution, and its stop point is not proven to occur before any output sink.
- No accepted hash-bound non-user fixture and generator/job provenance is available for this exact host path.
- No qualified observer proves a typed owner/source-generation transfer from the reader descriptor through the wrapper to the F12 count producer.

Do not launch the target, reuse the output-producing legacy jobs, substitute a writer stub, or treat matching values/order as identity. If the required F12 boundary is only reached after protected output, this capture route remains blocked. This is a bounded preflight finding, not a global assertion that no safe method can be built.

The existing static D-to-open-config-to-wrapper field transformations remain evidence-only. The D-derived metadata to F12 count-owner join, selected invocation, and source/reset generation remain `UNBOUND`. Product and implementation readiness do not change.
