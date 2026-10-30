// pattern: Imperative Shell
//! Mints a local Meter source identity from one prepared programme attempt.

use jocforge_domain::{
    frontend::job_configuration::{SelectedInputKind, SelectedWaveDescriptor},
    meter::{
        prepared_job_identity::PreparedJobInputIdentity,
        provider_set_identity::{MeterProviderSetIdentityError, SelectedMeterProviderSetIdentity},
        sample_block_queue::MeterSourceSampleCount,
        standalone_root::{
            MeterProviderSet, MeterSourceOwnerId, StandaloneMeterIdentity, StandaloneMeterRootError,
        },
    },
    profile::EmdfCarriageProfile,
    transaction::{AttemptId, ProgrammeId},
};

use crate::{PreparedJobInput, ResourceIdentity};

const SELECTED_WAVE: SelectedWaveDescriptor = SelectedWaveDescriptor::new(48_000, 38, 24);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSourceAdmissionError {
    SelectedInputKind,
    UnsupportedWaveProfile,
    UnsupportedCarriageProfile,
    SourceExtentMismatch,
    OwnerId(StandaloneMeterRootError),
    ProviderSetProfileMismatch,
    ProviderSetIdentity(MeterProviderSetIdentityError),
}

/// Parent-issued admission evidence tying one local source key to one prepared
/// programme attempt, input/configuration identity, profile, and typed extent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterSourceAdmissionReceipt {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    owner_id: MeterSourceOwnerId,
    resource_identity: ResourceIdentity,
    prepared_identity: PreparedJobInputIdentity,
    job_xml_sha256: [u8; 32],
    config_sha256: [u8; 32],
    input_kind: SelectedInputKind,
    selected_profile: jocforge_domain::profile::ResolvedEncodingProfile,
    wave: SelectedWaveDescriptor,
    source_samples: MeterSourceSampleCount,
}

impl MeterSourceAdmissionReceipt {
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub fn owner_id(&self) -> &MeterSourceOwnerId {
        &self.owner_id
    }

    pub const fn resource_identity(&self) -> ResourceIdentity {
        self.resource_identity
    }

    pub const fn job_xml_sha256(&self) -> [u8; 32] {
        self.job_xml_sha256
    }

    pub const fn config_sha256(&self) -> [u8; 32] {
        self.config_sha256
    }

    pub const fn input_kind(&self) -> SelectedInputKind {
        self.input_kind
    }

    pub const fn selected_profile(&self) -> jocforge_domain::profile::ResolvedEncodingProfile {
        self.selected_profile
    }

    pub fn selected_profile_identity(&self) -> Result<&'static str, MeterSourceAdmissionError> {
        selected_meter_profile_identity(self.selected_profile.carriage())
    }

    pub const fn wave(&self) -> SelectedWaveDescriptor {
        self.wave
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub fn prepared_identity(&self) -> &PreparedJobInputIdentity {
        &self.prepared_identity
    }

    /// Builds the domain identity from the exact values captured in this receipt.
    pub fn standalone_meter_identity(&self) -> StandaloneMeterIdentity {
        StandaloneMeterIdentity::new(
            self.programme_id.clone(),
            self.attempt,
            self.resource_identity.input_sha256(),
            self.config_sha256,
        )
    }

    /// Confirms the receipt still names this exact prepared input and attempt.
    pub fn matches(
        &self,
        prepared: &PreparedJobInput,
        attempt: AttemptId,
        source_samples: MeterSourceSampleCount,
    ) -> bool {
        self.programme_id == *prepared.programme_id()
            && self.attempt == attempt
            && self.owner_id.as_str()
                == owner_id_value(
                    prepared.programme_id(),
                    attempt,
                    prepared.resource_identity(),
                    prepared.config_sha256(),
                    source_samples,
                )
            && self.resource_identity == prepared.resource_identity()
            && self.prepared_identity == *prepared.domain_identity()
            && self.job_xml_sha256 == prepared.job_xml_sha256()
            && self.config_sha256 == prepared.config_sha256()
            && self.input_kind == prepared.configuration().input_kind()
            && self.selected_profile == prepared.configuration().selected_profile()
            && self.wave == prepared.configuration().wave()
            && self.source_samples == source_samples
            && source_samples.value() == prepared.configuration().source_samples()
    }

    /// Binds a validated static provider-set receipt to this prepared input/attempt identity.
    pub fn with_provider_set_identity(
        self,
        provider_identity: SelectedMeterProviderSetIdentity,
    ) -> Result<MeterSourceProviderSetAdmissionReceipt, MeterSourceAdmissionError> {
        let expected_profile = self.selected_profile_identity()?;
        if provider_identity.selected_profile_id() != expected_profile {
            return Err(MeterSourceAdmissionError::ProviderSetProfileMismatch);
        }
        let provider_set = provider_identity
            .clone()
            .into_meter_provider_set()
            .map_err(MeterSourceAdmissionError::ProviderSetIdentity)?;
        Ok(MeterSourceProviderSetAdmissionReceipt {
            source_admission: self,
            provider_identity,
            provider_set,
        })
    }
}

/// Stable local identity for the one supported source/carrier/topology/rate row and its carriage.
pub fn selected_meter_profile_identity(
    carriage: EmdfCarriageProfile,
) -> Result<&'static str, MeterSourceAdmissionError> {
    match carriage {
        EmdfCarriageProfile::DeployedCompat => Ok(
            "selected-meter-profile-v1/adm-bwf38-pcm24-48khz/5p1-six-carrier/joc-mode0-phase1-index3/single-independent-zero-dependent/48khz-1024kbps-6blocks-1536samples-4096bytes/deployed_compat",
        ),
        EmdfCarriageProfile::EtsiEmdf => Ok(
            "selected-meter-profile-v1/adm-bwf38-pcm24-48khz/5p1-six-carrier/joc-mode0-phase1-index3/single-independent-zero-dependent/48khz-1024kbps-6blocks-1536samples-4096bytes/etsi_emdf",
        ),
        EmdfCarriageProfile::Unsupported => {
            Err(MeterSourceAdmissionError::UnsupportedCarriageProfile)
        }
    }
}

/// One parent admission binding the local source owner and the static provider receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterSourceProviderSetAdmissionReceipt {
    source_admission: MeterSourceAdmissionReceipt,
    provider_identity: SelectedMeterProviderSetIdentity,
    provider_set: MeterProviderSet,
}

impl MeterSourceProviderSetAdmissionReceipt {
    pub const fn source_admission(&self) -> &MeterSourceAdmissionReceipt {
        &self.source_admission
    }

    pub const fn provider_identity(&self) -> &SelectedMeterProviderSetIdentity {
        &self.provider_identity
    }

    pub const fn provider_set(&self) -> &MeterProviderSet {
        &self.provider_set
    }

    pub const fn provider_set_sha256(&self) -> &[u8; 32] {
        self.provider_identity.sha256()
    }
}

/// Mints the local Meter source identity only after the prepared input and typed
/// sample extent agree with the selected ADM BWF profile.
pub fn admit_meter_source_owner(
    prepared: &PreparedJobInput,
    attempt: AttemptId,
    source_samples: MeterSourceSampleCount,
) -> Result<MeterSourceAdmissionReceipt, MeterSourceAdmissionError> {
    let configuration = prepared.configuration();
    if configuration.input_kind() != SelectedInputKind::Adm {
        return Err(MeterSourceAdmissionError::SelectedInputKind);
    }
    if configuration.wave() != SELECTED_WAVE {
        return Err(MeterSourceAdmissionError::UnsupportedWaveProfile);
    }
    if !matches!(
        configuration.selected_profile().carriage(),
        EmdfCarriageProfile::DeployedCompat | EmdfCarriageProfile::EtsiEmdf
    ) {
        return Err(MeterSourceAdmissionError::UnsupportedCarriageProfile);
    }
    if source_samples.value() == 0 || source_samples.value() != configuration.source_samples() {
        return Err(MeterSourceAdmissionError::SourceExtentMismatch);
    }

    let owner_id = MeterSourceOwnerId::new(owner_id_value(
        prepared.programme_id(),
        attempt,
        prepared.resource_identity(),
        prepared.config_sha256(),
        source_samples,
    ))
    .map_err(MeterSourceAdmissionError::OwnerId)?;

    Ok(MeterSourceAdmissionReceipt {
        programme_id: prepared.programme_id().clone(),
        attempt,
        owner_id,
        resource_identity: prepared.resource_identity(),
        prepared_identity: prepared.domain_identity().clone(),
        job_xml_sha256: prepared.job_xml_sha256(),
        config_sha256: prepared.config_sha256(),
        input_kind: configuration.input_kind(),
        selected_profile: configuration.selected_profile(),
        wave: configuration.wave(),
        source_samples,
    })
}

fn owner_id_value(
    programme_id: &ProgrammeId,
    attempt: AttemptId,
    resource_identity: ResourceIdentity,
    config_sha256: [u8; 32],
    source_samples: MeterSourceSampleCount,
) -> String {
    format!(
        "meter-source-owner-v1:{}:{}:{}:{}:{}:{}",
        encode_hex(programme_id.as_str().as_bytes()),
        attempt.value(),
        resource_identity.resource_bytes(),
        encode_hex(&resource_identity.input_sha256()),
        encode_hex(&config_sha256),
        source_samples.value(),
    )
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
