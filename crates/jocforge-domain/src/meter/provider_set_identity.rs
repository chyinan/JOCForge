// pattern: Functional Core
//! Local, deterministic identity for one fully validated selected Meter provider roster.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::standalone_root::{MeterProviderIdentity, MeterProviderSet, StandaloneMeterRootError};

const PAYLOAD_TAG: &[u8] = b"JOCFORGE_SELECTED_METER_PROVIDER_SET_V1\0";

/// Hashes a validator's canonical fixed-width value-word representation.
pub fn validated_value_words_sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterProviderSetIdentityError {
    EmptyProfile,
    EmptyRoster,
    DuplicateExpectedMember,
    DuplicateReceiptMember,
    MissingOrUnexpectedMember,
    SourceIdentityMismatch,
    ProfileMismatch,
    EmptyOwnerId,
    EmptyValidatorIdentity,
    NonAsciiIdentity,
    PayloadLengthOverflow,
    ProviderSet(StandaloneMeterRootError),
}

/// One member expected by the hash-bound selected-profile roster.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedMeterProviderMember {
    member_id: String,
    approved_source_sha256: [u8; 32],
}

impl ExpectedMeterProviderMember {
    pub fn new(member_id: impl Into<String>, approved_source_sha256: [u8; 32]) -> Self {
        Self {
            member_id: member_id.into(),
            approved_source_sha256,
        }
    }

    pub fn member_id(&self) -> &str {
        &self.member_id
    }

    pub const fn approved_source_sha256(&self) -> &[u8; 32] {
        &self.approved_source_sha256
    }
}

/// Hash-bound roster of expected selected-profile provider members.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMeterProviderRoster {
    package_manifest_sha256: [u8; 32],
    source_closure_manifest_sha256: [u8; 32],
    members: Vec<ExpectedMeterProviderMember>,
}

impl SelectedMeterProviderRoster {
    pub fn new(
        package_manifest_sha256: [u8; 32],
        source_closure_manifest_sha256: [u8; 32],
        members: Vec<ExpectedMeterProviderMember>,
    ) -> Result<Self, MeterProviderSetIdentityError> {
        if members.is_empty() {
            return Err(MeterProviderSetIdentityError::EmptyRoster);
        }
        let mut member_ids = members
            .iter()
            .map(|member| member.member_id.as_str())
            .collect::<Vec<_>>();
        member_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        if member_ids
            .iter()
            .any(|member_id| member_id.is_empty() || !member_id.is_ascii())
            || member_ids.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(MeterProviderSetIdentityError::DuplicateExpectedMember);
        }
        Ok(Self {
            package_manifest_sha256,
            source_closure_manifest_sha256,
            members,
        })
    }

    pub const fn package_manifest_sha256(&self) -> &[u8; 32] {
        &self.package_manifest_sha256
    }

    pub const fn source_closure_manifest_sha256(&self) -> &[u8; 32] {
        &self.source_closure_manifest_sha256
    }

    pub fn members(&self) -> &[ExpectedMeterProviderMember] {
        &self.members
    }
}

/// Immutable receipt emitted only after one member's owner validator succeeds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterProviderMemberReceipt {
    member_id: String,
    owner_id: String,
    approved_source_sha256: [u8; 32],
    validator_id: String,
    validator_version: String,
    selected_profile_id: String,
    validated_value_words_sha256: [u8; 32],
}

impl MeterProviderMemberReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        member_id: impl Into<String>,
        owner_id: impl Into<String>,
        approved_source_sha256: [u8; 32],
        validator_id: impl Into<String>,
        validator_version: impl Into<String>,
        selected_profile_id: impl Into<String>,
        validated_value_words_sha256: [u8; 32],
    ) -> Result<Self, MeterProviderSetIdentityError> {
        let member_id = member_id.into();
        let owner_id = owner_id.into();
        let validator_id = validator_id.into();
        let validator_version = validator_version.into();
        let selected_profile_id = selected_profile_id.into();
        if member_id.is_empty() {
            return Err(MeterProviderSetIdentityError::MissingOrUnexpectedMember);
        }
        if !member_id.is_ascii() || !owner_id.is_ascii() {
            return Err(MeterProviderSetIdentityError::NonAsciiIdentity);
        }
        if owner_id.is_empty() {
            return Err(MeterProviderSetIdentityError::EmptyOwnerId);
        }
        if validator_id.is_empty() || validator_version.is_empty() {
            return Err(MeterProviderSetIdentityError::EmptyValidatorIdentity);
        }
        if selected_profile_id.is_empty() {
            return Err(MeterProviderSetIdentityError::EmptyProfile);
        }
        if !validator_id.is_ascii()
            || !validator_version.is_ascii()
            || !selected_profile_id.is_ascii()
        {
            return Err(MeterProviderSetIdentityError::NonAsciiIdentity);
        }
        Ok(Self {
            member_id,
            owner_id,
            approved_source_sha256,
            validator_id,
            validator_version,
            selected_profile_id,
            validated_value_words_sha256,
        })
    }

    pub fn member_id(&self) -> &str {
        &self.member_id
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub const fn approved_source_sha256(&self) -> &[u8; 32] {
        &self.approved_source_sha256
    }

    pub fn validator_id(&self) -> &str {
        &self.validator_id
    }

    pub fn validator_version(&self) -> &str {
        &self.validator_version
    }

    pub fn selected_profile_id(&self) -> &str {
        &self.selected_profile_id
    }

    pub const fn validated_value_words_sha256(&self) -> &[u8; 32] {
        &self.validated_value_words_sha256
    }
}

/// Local static-provider identity and domain `MeterProviderSet` projection.
///
/// The digest identifies validated static provider receipts only. It is not an authentication
/// token, DSP result, warm-state digest, or proof that a selected-input prepass has run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMeterProviderSetIdentity {
    selected_profile_id: String,
    members: Vec<MeterProviderMemberReceipt>,
    sha256: [u8; 32],
}

impl SelectedMeterProviderSetIdentity {
    /// Validates the exact roster, sorts member receipts by ASCII identity, and computes the
    /// Wash-approved local payload recipe. The payload has fixed field order and no float values.
    pub fn new(
        expected_members: &[ExpectedMeterProviderMember],
        selected_profile_id: impl Into<String>,
        mut receipts: Vec<MeterProviderMemberReceipt>,
    ) -> Result<Self, MeterProviderSetIdentityError> {
        let selected_profile_id = selected_profile_id.into();
        if selected_profile_id.is_empty() {
            return Err(MeterProviderSetIdentityError::EmptyProfile);
        }
        if expected_members.is_empty() {
            return Err(MeterProviderSetIdentityError::EmptyRoster);
        }

        let mut expected = BTreeMap::new();
        for member in expected_members {
            if member.member_id.is_empty()
                || !member.member_id.is_ascii()
                || expected
                    .insert(member.member_id.as_str(), member.approved_source_sha256)
                    .is_some()
            {
                return Err(MeterProviderSetIdentityError::DuplicateExpectedMember);
            }
        }

        receipts.sort_by(|left, right| {
            left.member_id
                .as_bytes()
                .cmp(right.member_id.as_bytes())
                .then_with(|| left.owner_id.as_bytes().cmp(right.owner_id.as_bytes()))
        });
        if receipts.len() != expected.len()
            || receipts
                .windows(2)
                .any(|pair| pair[0].member_id == pair[1].member_id)
        {
            return Err(MeterProviderSetIdentityError::DuplicateReceiptMember);
        }

        for receipt in &receipts {
            let expected_sha = expected
                .get(receipt.member_id.as_str())
                .ok_or(MeterProviderSetIdentityError::MissingOrUnexpectedMember)?;
            if expected_sha != &receipt.approved_source_sha256 {
                return Err(MeterProviderSetIdentityError::SourceIdentityMismatch);
            }
            if receipt.selected_profile_id != selected_profile_id {
                return Err(MeterProviderSetIdentityError::ProfileMismatch);
            }
        }
        if expected.keys().any(|member_id| {
            !receipts
                .iter()
                .any(|receipt| receipt.member_id == *member_id)
        }) {
            return Err(MeterProviderSetIdentityError::MissingOrUnexpectedMember);
        }

        let payload = provider_set_payload(&selected_profile_id, &receipts)?;
        let sha256 = Sha256::digest(payload).into();
        Ok(Self {
            selected_profile_id,
            members: receipts,
            sha256,
        })
    }

    pub fn selected_profile_id(&self) -> &str {
        &self.selected_profile_id
    }

    pub fn members(&self) -> &[MeterProviderMemberReceipt] {
        &self.members
    }

    pub const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    /// Projects the receipt into the root's sorted member identity type.
    pub fn into_meter_provider_set(
        self,
    ) -> Result<MeterProviderSet, MeterProviderSetIdentityError> {
        let providers = self
            .members
            .into_iter()
            .map(|member| {
                MeterProviderIdentity::new(member.member_id, member.approved_source_sha256)
                    .map_err(MeterProviderSetIdentityError::ProviderSet)
            })
            .collect::<Result<Vec<_>, _>>()?;
        MeterProviderSet::new(self.sha256, providers)
            .map_err(MeterProviderSetIdentityError::ProviderSet)
    }
}

fn provider_set_payload(
    selected_profile_id: &str,
    members: &[MeterProviderMemberReceipt],
) -> Result<Vec<u8>, MeterProviderSetIdentityError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(PAYLOAD_TAG);
    append_text(&mut bytes, selected_profile_id)?;
    append_count(&mut bytes, members.len())?;
    for member in members {
        append_text(&mut bytes, &member.member_id)?;
        append_text(&mut bytes, &member.owner_id)?;
        bytes.extend_from_slice(&member.approved_source_sha256);
        append_text(&mut bytes, &member.validator_id)?;
        append_text(&mut bytes, &member.validator_version)?;
        append_text(&mut bytes, &member.selected_profile_id)?;
        bytes.extend_from_slice(&member.validated_value_words_sha256);
    }
    Ok(bytes)
}

fn append_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), MeterProviderSetIdentityError> {
    append_count(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn append_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), MeterProviderSetIdentityError> {
    let value =
        u32::try_from(count).map_err(|_| MeterProviderSetIdentityError::PayloadLengthOverflow)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_and_receipts() -> (
        Vec<ExpectedMeterProviderMember>,
        Vec<MeterProviderMemberReceipt>,
    ) {
        let expected = vec![
            ExpectedMeterProviderMember::new("D11-FFT", [0x11; 32]),
            ExpectedMeterProviderMember::new("D12-K", [0x22; 32]),
        ];
        let receipts = vec![
            MeterProviderMemberReceipt::new(
                "D12-K",
                "meter",
                [0x22; 32],
                "validated-k-matrix",
                "1",
                "selected-adm-bwf-v1",
                [0x42; 32],
            )
            .unwrap(),
            MeterProviderMemberReceipt::new(
                "D11-FFT",
                "spatial_control",
                [0x11; 32],
                "validated-feature-fft",
                "1",
                "selected-adm-bwf-v1",
                [0x41; 32],
            )
            .unwrap(),
        ];
        (expected, receipts)
    }

    #[test]
    fn exact_roster_and_member_order_produce_one_local_provider_set_identity() {
        let (expected, receipts) = expected_and_receipts();
        let first = SelectedMeterProviderSetIdentity::new(
            &expected,
            "selected-adm-bwf-v1",
            receipts.clone(),
        )
        .unwrap();
        let reversed = SelectedMeterProviderSetIdentity::new(
            &expected,
            "selected-adm-bwf-v1",
            receipts.into_iter().rev().collect(),
        )
        .unwrap();

        assert_eq!(first, reversed);
        assert_eq!(first.members()[0].member_id(), "D11-FFT");
        assert_eq!(first.members()[1].member_id(), "D12-K");
        let set = first.into_meter_provider_set().unwrap();
        assert_eq!(set.providers()[0].id(), "D11-FFT");
        assert_eq!(set.providers()[1].id(), "D12-K");
    }

    #[test]
    fn digest_changes_when_one_validated_value_word_digest_changes() {
        let (expected, receipts) = expected_and_receipts();
        let before = SelectedMeterProviderSetIdentity::new(
            &expected,
            "selected-adm-bwf-v1",
            receipts.clone(),
        )
        .unwrap();
        let mut changed = receipts;
        changed[0] = MeterProviderMemberReceipt::new(
            "D12-K",
            "meter",
            [0x22; 32],
            "validated-k-matrix",
            "1",
            "selected-adm-bwf-v1",
            [0x43; 32],
        )
        .unwrap();
        let after =
            SelectedMeterProviderSetIdentity::new(&expected, "selected-adm-bwf-v1", changed)
                .unwrap();
        assert_ne!(before.sha256(), after.sha256());
    }

    #[test]
    fn incomplete_or_wrong_profile_member_rosters_reject() {
        let (expected, mut receipts) = expected_and_receipts();
        assert_eq!(
            SelectedMeterProviderSetIdentity::new(
                &expected,
                "selected-adm-bwf-v1",
                receipts[..1].to_vec(),
            )
            .err(),
            Some(MeterProviderSetIdentityError::DuplicateReceiptMember)
        );
        receipts[0] = MeterProviderMemberReceipt::new(
            "D12-K",
            "meter",
            [0x22; 32],
            "validated-k-matrix",
            "1",
            "unsupported-profile",
            [0x42; 32],
        )
        .unwrap();
        assert_eq!(
            SelectedMeterProviderSetIdentity::new(&expected, "selected-adm-bwf-v1", receipts,)
                .err(),
            Some(MeterProviderSetIdentityError::ProfileMismatch)
        );
    }
}
