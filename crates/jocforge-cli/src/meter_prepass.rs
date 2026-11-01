// pattern: Imperative Shell
//! Builds the selected sample's noncommitting Meter prepass and dependent-owner inputs.

use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use jocforge_application::output_publication::AttemptId;
use jocforge_application::{
    PreparedJobInput,
    independent_meter_callback::{
        MeterKFilterExponentState, StandaloneMeterRoot, generate_selected_spatial_math_geometry,
    },
    meter_finalization_bridge::{MeasurementDependentOwnerInputs, MeterFinalizationBridgeError},
    meter_source_admission::{MeterSourceAdmissionError, admit_meter_source_owner},
    meter_source_bridge::{MeterSourceSampleCount, SharedGainDelayParameters},
};
use jocforge_io::{
    pcm::{
        meter_source_cursor::{MeterPcmSourceCursor, MeterPcmSourceIdentity},
        riff_pcm24::ParsedRiffPcm24,
    },
    provider::load_meter_direct_provider_inventory,
};

use crate::{
    independent_meter_callback::IndependentMeterPrepassCandidate,
    selected_meter_provider_identity::{
        SelectedMeterProviderIdentityError, load_selected_meter_provider_identity,
        selected_meter_profile_id,
    },
    selected_spatial_callback_providers::{
        SelectedIndependentMeterProviderLoadError,
        load_selected_independent_meter_callback_providers,
    },
};

const SELECTED_K_FILTER_COLD_WORDS: [i32; 6] = [0; 6];
const SELECTED_K_FILTER_COLD_EVIDENCE_MANIFEST: &str =
    "c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38";

/// Complete real-input meter calculation proposal and its same-receipt dependents.
///
/// All meter callbacks remain proposals. The programme parent still owns measurement admission,
/// READY installation and callback/source root replacement.
#[must_use]
pub struct SelectedMeterPrepass {
    candidate: IndependentMeterPrepassCandidate,
    dependents: MeasurementDependentOwnerInputs,
}

impl SelectedMeterPrepass {
    pub const fn candidate(&self) -> &IndependentMeterPrepassCandidate {
        &self.candidate
    }

    pub fn terminal_meter_root(&self) -> &StandaloneMeterRoot {
        self.candidate
            .final_callback()
            .application_candidate()
            .meter_proposal()
            .after_root()
    }

    pub const fn dependents(&self) -> &MeasurementDependentOwnerInputs {
        &self.dependents
    }
}

#[derive(Debug)]
pub struct SelectedMeterPrepassError(String);

impl fmt::Display for SelectedMeterPrepassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for SelectedMeterPrepassError {}

fn error(label: &str, detail: impl fmt::Debug) -> SelectedMeterPrepassError {
    SelectedMeterPrepassError(format!("{label}: {detail:?}"))
}

fn repository_root() -> Result<PathBuf, SelectedMeterPrepassError> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|source| error("repository root", source))
}

/// Runs the approved full-file Meter source schedule over the parsed ADM BWF bytes.
pub fn run_selected_meter_prepass(
    prepared: &PreparedJobInput,
    parsed_source: &ParsedRiffPcm24<'_>,
    attempt: AttemptId,
) -> Result<SelectedMeterPrepass, SelectedMeterPrepassError> {
    run_selected_meter_prepass_with_progress(prepared, parsed_source, attempt, |_, _| {})
}

/// Runs the approved full-file Meter schedule and reports completed callbacks.
pub fn run_selected_meter_prepass_with_progress<F>(
    prepared: &PreparedJobInput,
    parsed_source: &ParsedRiffPcm24<'_>,
    attempt: AttemptId,
    progress: F,
) -> Result<SelectedMeterPrepass, SelectedMeterPrepassError>
where
    F: FnMut(u64, u64) + Send,
{
    let root = repository_root()?;
    let supplements = root.join("outputs/IMPLEMENTER_SUPPLEMENTS");
    let spatial_provider_root = supplements.join("ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1");
    let cold_state_root = supplements.join("F01B-F03W-COLD-STATE-ASSETS-V1");
    let direct_meter_root = supplements.join("METER-APPROVED-PARAMETER-ASSETS-V1");
    let meter_roster_root = supplements.join("METER-PROVIDER-SET-IDENTITY-APPLICATION-V1");

    let geometry = generate_selected_spatial_math_geometry()
        .map_err(|failure| error("selected spatial geometry", failure))?;
    let providers = load_selected_independent_meter_callback_providers(
        &spatial_provider_root,
        &cold_state_root,
        &direct_meter_root,
        geometry,
    )
    .map_err(|failure: SelectedIndependentMeterProviderLoadError| {
        error("selected Meter callback providers", failure)
    })?;
    let direct = load_meter_direct_provider_inventory(&direct_meter_root)
        .map_err(|failure| error("selected direct Meter providers", failure))?;
    let selected_profile_id =
        selected_meter_profile_id(prepared.configuration().selected_profile().carriage()).map_err(
            |failure: SelectedMeterProviderIdentityError| error("selected Meter profile", failure),
        )?;
    let provider_identity = load_selected_meter_provider_identity(
        &meter_roster_root,
        &providers,
        &direct,
        selected_profile_id,
    )
    .map_err(|failure| error("selected Meter provider identity", failure))?;

    let source_samples = MeterSourceSampleCount::new(prepared.configuration().source_samples())
        .map_err(|failure| error("selected Meter source extent", failure))?;
    let source_admission = admit_meter_source_owner(prepared, attempt, source_samples)
        .map_err(|failure: MeterSourceAdmissionError| error("Meter source admission", failure))?;
    let bound_provider_set = source_admission
        .with_provider_set_identity(provider_identity)
        .map_err(|failure| error("Meter provider-set admission", failure))?;

    // F06-KFILTER-EXPONENT-INITIALIZATION-V1 binds the selected DPLM cold K state to [0; 6].
    // The separate energy exponent cold value is [48; 6] and is owned by the Meter root.
    debug_assert_eq!(
        SELECTED_K_FILTER_COLD_EVIDENCE_MANIFEST,
        "c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38"
    );
    let gain_parameters = SharedGainDelayParameters::from_approved_direct_parameters(
        direct.shared_dv32_parameters(),
        &providers.providers().window,
    )
    .map_err(|failure| error("shared DV32 gain-delay parameters", failure))?;
    let meter_root = StandaloneMeterRoot::cold_selected_with_post_render_parameters(
        bound_provider_set
            .source_admission()
            .standalone_meter_identity(),
        source_samples,
        bound_provider_set.source_admission().owner_id().clone(),
        bound_provider_set.provider_set().clone(),
        MeterKFilterExponentState::new(SELECTED_K_FILTER_COLD_WORDS),
        gain_parameters,
        direct.k_matrix().clone(),
        direct.post_render_parameters(),
    )
    .map_err(|failure| error("selected StandaloneMeterRoot cold state", failure))?;

    let cursor = MeterPcmSourceCursor::cold(
        parsed_source,
        MeterPcmSourceIdentity::from_selected_resource_sha256(
            prepared.resource_identity().input_sha256(),
        ),
        source_samples,
    )
    .map_err(|failure| error("selected Meter PCM source cursor", failure))?;
    let candidate = providers
        .stage_prepass_with_progress(prepared, &cursor, parsed_source, &meter_root, progress)
        .map_err(|failure| error("selected full-file Meter prepass", failure))?;
    let dependents = candidate.prepare_measurement_dependents(prepared).map_err(
        |failure: MeterFinalizationBridgeError| {
            error("terminal measurement receipt dependents", failure)
        },
    )?;

    Ok(SelectedMeterPrepass {
        candidate,
        dependents,
    })
}
