# Selected carrier admission

SELECTED-CARRIER-SCOPE-CLOSURE-V1 manifest SHA-256 is `29e3b943bfa49417bd69586034b8ba61a8274fb9b3a8af79175b0c0406b38ad2`. The distributed contract payload matches its manifest. Upstream research bindings and offline checker results are coordinator evidence, not local native execution.

`CarrierPlannerState::plan_selected_container` implements the existing product length policy: optional complete container C includes four prefix bytes, C<=511, and B=C-4 is assigned to zero-based block 2 without continuation. None means absent optional payload; zero/empty/truncated container lengths and oversized complete containers fail before planner mutation. This length admission does not replace payload syntax, exact consumption, selected profile or frame feasibility validation.

The existing larger-body planner and fragment writer remain controlled component mechanisms. Product callers must use selected admission; their larger native scheduling domain does not expand product policy. Producer index validation 0..5 is not permission to choose six product placements. Downmix index 3 is a different domain.

Tests cover every length C=5..511, fixed block2 placement, exact B length, rejection state preservation and absent payload. These establish product length policy, not complete EMDF syntax or frame encoding. The selected carrier identity/admission specification blocker is withdrawn. Full F13, same-source frame construction and end-to-end acceptance remain incomplete.
