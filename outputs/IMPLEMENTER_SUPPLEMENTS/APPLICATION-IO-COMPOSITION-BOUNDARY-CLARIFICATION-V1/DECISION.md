# Reader and application composition boundary

Decision: retain the approved application-to-io prohibition. This is a local architecture clarification for the already anticipated CLI I/O assembly, not a DEE behavior or audio algorithm specification.

The production composition root may depend on application and io. The minimal permitted edge addition is CLI -> io. Application continues to depend on domain only for internal project code; io continues to depend on domain. Production does not depend on validation. No CLI -> domain edge or new crate is required by this decision.

Keep the concrete reader, ParsedRiff adapter and MeterPcmSourceCursor ownership in io/the CLI shell. Application receives immutable domain-owned frame data, actual counts, identity and clock values through its existing typed interface; it must not own or name an io-specific cursor/adapter. If a current io result wrapper is concrete to io, destructure it in the shell and pass its domain values to application. Add only the minimal explicit value adapter if the existing values need one.

The CLI shell performs read/adapter calls and invokes application coordination. It does not implement spatial, measurement or codec math, nor move domain state computation into main. Application/domain continue to validate the source/phase/count handoff and stage their own results. Do not duplicate scheduling logic merely to bridge crate types.

Do not add a generic port framework, callback registry, utility crate or cyclic dependency for this seam. A short composition function that calls io then application is sufficient. Keep encode unavailable until its actual production dependencies are implemented and verified.

Apply the change in this order: remove the attempted application -> jocforge-io dependency and corresponding type use; document CLI's composition-root -> io edge in DEPENDENCY-RULES and the relevant module/dataflow/API notes; update only that permitted checker edge; then add CLI's concrete io dependency and connect the value handoff. Preserve rejection of application -> io, domain I/O, and production -> validation. Run checks appropriate to these changes, including the existing boundary check; do not waive a failing boundary by adding io to application's allowlist.
