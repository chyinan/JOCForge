// pattern: Functional Core
//! F12S.8 selected main-q search over complete per-q scratch candidates.

use super::selected_frame_trial::{
    SelectedFrameTrialFatalError, SelectedFrameTrialOutcome, SelectedFrameTrialPlan,
    SelectedMainFrameCandidate, build_selected_frame_trial,
};

const INITIAL_MAIN_Q: u16 = 16;
const MAIN_Q_LOW: u16 = 16;
const MAIN_Q_HIGH: u16 = 784;
const MAX_MIDPOINT_TRIALS: u8 = 10;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedMainQOutcome {
    Best(Box<SelectedMainFrameCandidate>),
    /// No q produced a complete frame; the enclosing source is fatal per F12S.8.
    NoBest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMainQSearchResult {
    outcome: SelectedMainQOutcome,
    attempted_qs: Vec<u16>,
    midpoint_trials: u8,
}

impl SelectedMainQSearchResult {
    pub const fn outcome(&self) -> &SelectedMainQOutcome {
        &self.outcome
    }

    pub fn attempted_qs(&self) -> &[u16] {
        &self.attempted_qs
    }

    pub const fn midpoint_trials(&self) -> u8 {
        self.midpoint_trials
    }
}

/// Preserves a preflight or producer error instead of turning it into a q failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedMainQSearchError<E> {
    /// The caller's preflight/F11 error, preserved without mapping it to q failure.
    Upstream(E),
    Trial(SelectedFrameTrialFatalError),
}

/// Runs the selected q=16 seed and bounded midpoint search against one fixed plan.
///
/// Pass the caller's plan-construction result directly. Its error type may wrap
/// F11 converter production or other preflight failures, which are preserved as
/// `Upstream`. Each q delegates to a fresh `build_selected_frame_trial` call;
/// this function neither recomputes upstream plans nor commits the winner.
pub fn search_selected_main_q<E>(
    upstream_result: Result<&SelectedFrameTrialPlan<'_>, E>,
) -> Result<SelectedMainQSearchResult, SelectedMainQSearchError<E>> {
    let plan = upstream_result.map_err(SelectedMainQSearchError::Upstream)?;
    let decision = run_q_search(|q| {
        let trial = build_selected_frame_trial(plan, q).map_err(SelectedMainQSearchError::Trial)?;
        Ok(match trial {
            SelectedFrameTrialOutcome::Built(candidate) => Some(candidate),
            SelectedFrameTrialOutcome::Infeasible(_) => None,
        })
    })?;

    Ok(SelectedMainQSearchResult {
        outcome: decision
            .best
            .map_or(SelectedMainQOutcome::NoBest, |candidate| {
                SelectedMainQOutcome::Best(candidate)
            }),
        attempted_qs: decision.attempted_qs,
        midpoint_trials: decision.midpoint_trials,
    })
}

struct QSearchDecision<T> {
    best: Option<T>,
    attempted_qs: Vec<u16>,
    midpoint_trials: u8,
}

fn run_q_search<T, E, F>(mut trial: F) -> Result<QSearchDecision<T>, E>
where
    F: FnMut(u16) -> Result<Option<T>, E>,
{
    let mut attempted_qs = Vec::with_capacity(1 + usize::from(MAX_MIDPOINT_TRIALS));
    attempted_qs.push(INITIAL_MAIN_Q);
    let mut best = trial(INITIAL_MAIN_Q)?;

    let mut low = MAIN_Q_LOW;
    let mut high = MAIN_Q_HIGH;
    let mut midpoint_trials = 0_u8;
    while high - low > 1 && midpoint_trials < MAX_MIDPOINT_TRIALS {
        let q = u16::midpoint(low, high);
        midpoint_trials += 1;
        attempted_qs.push(q);
        match trial(q)? {
            Some(candidate) => {
                best = Some(candidate);
                low = q;
            }
            None => high = q,
        }
    }

    Ok(QSearchDecision {
        best,
        attempted_qs,
        midpoint_trials,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::convert::Infallible;

    const FRAME_PACKAGE_MANIFEST: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/MANIFEST.json"
    );
    const INPUT_ONLY_PLANS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/INPUT-ONLY-PLANS.json"
    );
    const FRAME_PACKAGE_MANIFEST_SHA256: &str =
        "d9085785d5484d236d4b24925a52954fc0bb01153c2e173e2d07d8f6e1b4a5f6";
    const INPUT_ONLY_PLANS_SHA256: &str =
        "2a5b7a11cc2d429435a2b0c2fe88255f5eeffed295c0954239e8b2a6791d7d74";

    #[test]
    fn input_only_suppressed_plan_has_no_output_seed_and_all_fail_order_is_exact() {
        let manifest: Value = serde_json::from_slice(FRAME_PACKAGE_MANIFEST).unwrap();
        let plans: Value = serde_json::from_slice(INPUT_ONLY_PLANS).unwrap();
        assert_eq!(
            sha256(FRAME_PACKAGE_MANIFEST),
            FRAME_PACKAGE_MANIFEST_SHA256
        );
        assert_eq!(sha256(INPUT_ONLY_PLANS), INPUT_ONLY_PLANS_SHA256);
        assert_eq!(
            manifest["package_id"],
            "CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1"
        );
        assert_eq!(plans["input_only"], true);
        assert_eq!(plans["expected_frame_or_au_bytes_included"], false);
        assert_eq!(plans["prototype_expected_outputs_used_as_seed"], false);

        let source0 = plans["plans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|plan| plan["id"] == "source0_suppressed")
            .unwrap();
        assert_eq!(source0["inputs"]["payload"], "ABSENT_PUBLISHED_PAYLOAD_V1");
        assert_eq!(source0["inputs"]["build_full_frame_scratch"], true);

        let decision = run_q_search::<u16, Infallible, _>(|_| Ok(None)).unwrap();
        assert_eq!(
            decision.attempted_qs,
            [16, 400, 208, 112, 64, 40, 28, 22, 19, 17]
        );
        assert_eq!(decision.midpoint_trials, 9);
        assert_eq!(decision.best, None);
    }

    #[test]
    fn successful_midpoints_retain_the_last_candidate_and_stop_after_ten() {
        let decision = run_q_search::<u16, Infallible, _>(|q| Ok(Some(q))).unwrap();
        assert_eq!(
            decision.attempted_qs,
            [16, 400, 592, 688, 736, 760, 772, 778, 781, 782, 783]
        );
        assert_eq!(decision.midpoint_trials, 10);
        assert_eq!(decision.best, Some(783));
    }

    #[test]
    fn boundary_success_at_four_hundred_retains_that_candidate() {
        let decision = run_q_search::<u16, Infallible, _>(|q| Ok((q <= 400).then_some(q))).unwrap();
        assert_eq!(
            decision.attempted_qs,
            [16, 400, 592, 496, 448, 424, 412, 406, 403, 401]
        );
        assert_eq!(decision.midpoint_trials, 9);
        assert_eq!(decision.best, Some(400));
    }

    #[test]
    fn preflight_and_converter_errors_are_preserved_before_any_trial() {
        let error = search_selected_main_q::<&'static str>(Err("F11 converter producer failed"))
            .unwrap_err();
        assert_eq!(
            error,
            SelectedMainQSearchError::Upstream("F11 converter producer failed")
        );
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
}
