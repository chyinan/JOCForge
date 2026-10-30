// pattern: Functional Core
//! Typed projections of the selected installed48 cold schema and owner map.

use super::{
    F03wAssetManifestError, ProviderFileEvidence, ProviderSha256, ValidatedF01bColdAssetManifest,
};
use serde_json::{Map, Value};

const INSTALLED48_SCHEMA_PATH: &str = "parameters/adm-installed48-state-v1.json";
const INSTALLED48_SCHEMA_ID: &str = "selected-adm-installed48-state.v3";
const OWNER_PROJECTION_PATH: &str = "parameters/selected-owner-projection-v1.json";

const INSTALLED48_TOP_LEVEL_FIELDS: &[&str] = &[
    "schema",
    "purpose",
    "f03w_owned_fields",
    "external_owner_refs",
    "functional_constant_providers",
    "cold_constructors",
    "serialization",
    "commit",
    "failure",
    "reset",
    "fixture_note",
    "native_abi_claim",
];

const OWNED_FIELD_GROUPS: &[&str] = &[
    "clocks",
    "source_cache",
    "event_control",
    "render_audio",
    "source_flags",
    "feature",
    "selection_outer",
    "matrix_assignment",
];
const PROFILE_DIMENSION_FIELDS: &[&str] = &[
    "source_count_N",
    "feature_bands_B",
    "target_capacity_T",
    "axes",
    "active_target_count_n",
];

const OWNER_PROJECTION_TOP_LEVEL_FIELDS: &[&str] = &[
    "schema",
    "authoritative_for",
    "field_groups",
    "u04_admission",
    "j05",
    "queue_rule",
    "commit_rule",
    "private_key_required",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedColdContractError {
    Manifest(F03wAssetManifestError),
    Utf8,
    JsonSyntax,
    Schema,
    Shape,
}

/// Dimensions read from the approved installed48 schema, not inferred from runtime arrays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedInstalled48Dimensions {
    source_count: u8,
    feature_bands: u8,
    target_capacity: u8,
    axes: u8,
}

impl SelectedInstalled48Dimensions {
    pub const fn source_count(self) -> u8 {
        self.source_count
    }

    pub const fn feature_bands(self) -> u8 {
        self.feature_bands
    }

    pub const fn target_capacity(self) -> u8 {
        self.target_capacity
    }

    pub const fn axes(self) -> u8 {
        self.axes
    }
}

/// Parsed schema contract; it does not claim that all of this state has a runtime producer.
pub struct ValidatedInstalled48StateSchema {
    manifest_sha256: ProviderSha256,
    provider_sha256: ProviderSha256,
    dimensions: SelectedInstalled48Dimensions,
}

impl ValidatedInstalled48StateSchema {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn provider_sha256(&self) -> ProviderSha256 {
        self.provider_sha256
    }

    pub const fn dimensions(&self) -> SelectedInstalled48Dimensions {
        self.dimensions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedOwnerProjectionGroupId {
    FrontendAndInstalled48,
    Reader56Targets,
    ComponentLfeJ05,
    JocAfterJ05,
    BodyProposals,
    MetadataTimeline,
    CodecAndPublication,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedOwnerId {
    F03w,
    F04c,
    F05r56,
    F05jl,
    F05l,
    F13,
    F15,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedOwnerExecutor {
    F03w,
    F04c,
    F05r56,
    F05jl,
    F05l,
    F13,
    F12ThenF15,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedCommitOwner {
    SpatialControl,
    SpatialAudio,
    Joc,
    PureValue,
    Metadata,
    CodecAndOutput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedOwnerProjectionGroup {
    id: SelectedOwnerProjectionGroupId,
    executor: SelectedOwnerExecutor,
    receipt_owner: SelectedOwnerId,
    commit_owner: SelectedCommitOwner,
    fields: &'static [&'static str],
}

impl SelectedOwnerProjectionGroup {
    pub const fn id(self) -> SelectedOwnerProjectionGroupId {
        self.id
    }

    pub const fn executor(self) -> SelectedOwnerExecutor {
        self.executor
    }

    pub const fn receipt_owner(self) -> SelectedOwnerId {
        self.receipt_owner
    }

    pub const fn commit_owner(self) -> SelectedCommitOwner {
        self.commit_owner
    }

    pub const fn fields(self) -> &'static [&'static str] {
        self.fields
    }
}

/// Typed owner/commit projection; this is a boundary contract, not commit authority.
pub struct ValidatedSelectedOwnerProjection {
    manifest_sha256: ProviderSha256,
    provider_sha256: ProviderSha256,
    groups: [SelectedOwnerProjectionGroup; 7],
}

impl ValidatedSelectedOwnerProjection {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn provider_sha256(&self) -> ProviderSha256 {
        self.provider_sha256
    }

    pub const fn groups(&self) -> &[SelectedOwnerProjectionGroup; 7] {
        &self.groups
    }
}

/// Validates the installed48 schema through its own cold-bundle entry and parses its dimensions.
pub fn validate_installed48_state_schema(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF01bColdAssetManifest,
) -> Result<ValidatedInstalled48StateSchema, SelectedColdContractError> {
    let bound = manifest
        .bind_schema_asset(evidence, INSTALLED48_SCHEMA_PATH)
        .map_err(SelectedColdContractError::Manifest)?;
    let document = parse_document(bound.bytes())?;
    let fields = document
        .as_object()
        .ok_or(SelectedColdContractError::Schema)?;
    if !exact_keys(fields, INSTALLED48_TOP_LEVEL_FIELDS)
        || string_field(fields, "schema")? != INSTALLED48_SCHEMA_ID
        || string_field(fields, "purpose")?
            != "Production state layout and cross-owner references. Synthetic repeat/truncate payloads are forbidden."
        || fields.get("native_abi_claim") != Some(&Value::Bool(false))
    {
        return Err(SelectedColdContractError::Schema);
    }

    let owned = object_field(fields, "f03w_owned_fields")?;
    if !exact_keys(owned, OWNED_FIELD_GROUPS) {
        return Err(SelectedColdContractError::Schema);
    }
    let selection = object_field(owned, "selection_outer")?;
    let dimensions = object_field(selection, "profile_dimensions")?;
    if !exact_keys(dimensions, PROFILE_DIMENSION_FIELDS)
        || string_field(dimensions, "active_target_count_n")?
            != "runtime integer 0..15; arrays allocate T=15 and consume prefix [0:n]"
    {
        return Err(SelectedColdContractError::Schema);
    }
    let dimensions = SelectedInstalled48Dimensions {
        source_count: exact_dimension(dimensions, "source_count_N", 48)?,
        feature_bands: exact_dimension(dimensions, "feature_bands_B", 24)?,
        target_capacity: exact_dimension(dimensions, "target_capacity_T", 15)?,
        axes: exact_dimension(dimensions, "axes", 3)?,
    };

    let providers = object_field(fields, "functional_constant_providers")?;
    if !exact_keys(providers, &["f01e_cold", "f03_target_cold"])
        || string_field(providers, "f01e_cold")? != "parameters/f01e-selected-constructor-v1.json"
        || string_field(providers, "f03_target_cold")?
            != "parameters/f03-selected-target-cold-v1.json"
    {
        return Err(SelectedColdContractError::Schema);
    }

    let constructors = object_field(fields, "cold_constructors")?;
    if !exact_keys(constructors, OWNED_FIELD_GROUPS)
        || string_field(constructors, "clocks")? != "all zero except remaining_cached_drains=4"
        || string_field(constructors, "event_control")?
            != "parameters/f01e-selected-constructor-v1.json exact canonical cold"
        || string_field(constructors, "selection_outer")?
            != "parameters/f03-selected-target-cold-v1.json plus all declared scratch positive-zero/empty/index-zero constructors"
    {
        return Err(SelectedColdContractError::Schema);
    }

    let external_refs = object_field(fields, "external_owner_refs")?;
    if !exact_keys(
        external_refs,
        &[
            "reader_target",
            "component_lfe_receiver_qmf",
            "joc_after_j05",
            "metadata_queues",
            "source_publication",
        ],
    ) || string_field(object_field(external_refs, "metadata_queues")?, "owner")? != "F13"
        || string_field(object_field(external_refs, "source_publication")?, "owner")? != "F15"
    {
        return Err(SelectedColdContractError::Schema);
    }

    for (field, expected) in [
        (
            "commit",
            "F03W commits only f03w_owned_fields once after its leaf stages and downstream receipts all pass. External state is committed by its unique owner under the enclosing source transaction.",
        ),
        (
            "failure",
            "All owner roots retain complete before bytes and generation; top-level result is RETIRED with commit_count=0.",
        ),
        (
            "reset",
            "Destroy every owner root and call each referenced cold constructor. Missing cold constructor is fatal; unknown arrays may not be zero-filled.",
        ),
    ] {
        if string_field(fields, field)? != expected {
            return Err(SelectedColdContractError::Schema);
        }
    }

    Ok(ValidatedInstalled48StateSchema {
        manifest_sha256: bound.manifest_sha256(),
        provider_sha256: ProviderSha256::digest(bound.bytes()),
        dimensions,
    })
}

/// Validates and parses all owner, receipt, and commit-owner groups from the approved projection.
const GROUP_SPECS: [GroupSpec; 7] = [
    GroupSpec {
        name: "frontend_and_installed48",
        id: SelectedOwnerProjectionGroupId::FrontendAndInstalled48,
        executor: SelectedOwnerExecutor::F03w,
        receipt_owner: SelectedOwnerId::F03w,
        commit_owner: SelectedCommitOwner::SpatialControl,
        fields: &[
            "source_cursor",
            "EOF_cache",
            "event_control",
            "render_diffusion",
            "source_flags",
            "feature",
            "target_state",
            "selector_scratch",
            "u04_scratch",
            "nnls_outer_scratch",
            "matrix",
            "markers",
            "writer_set",
        ],
    },
    GroupSpec {
        name: "reader56_targets",
        id: SelectedOwnerProjectionGroupId::Reader56Targets,
        executor: SelectedOwnerExecutor::F05r56,
        receipt_owner: SelectedOwnerId::F05r56,
        commit_owner: SelectedCommitOwner::SpatialControl,
        fields: &[
            "metadata_history",
            "previous_active",
            "current_target_snapshot",
            "reader_records_16x56",
            "target_words_16x6",
            "reader_invocations",
        ],
    },
    GroupSpec {
        name: "component_lfe_j05",
        id: SelectedOwnerProjectionGroupId::ComponentLfeJ05,
        executor: SelectedOwnerExecutor::F04c,
        receipt_owner: SelectedOwnerId::F04c,
        commit_owner: SelectedCommitOwner::SpatialAudio,
        fields: &[
            "component_overlap_15x1536",
            "component_ring_15x1472",
            "filter_history_15x64",
            "component_parameters",
            "lfe_slots_3x1536",
            "lfe_indices",
            "receiver_state",
            "receiver_audio_qmf",
            "fitting_qmf",
        ],
    },
    GroupSpec {
        name: "joc_after_j05",
        id: SelectedOwnerProjectionGroupId::JocAfterJ05,
        executor: SelectedOwnerExecutor::F05jl,
        receipt_owner: SelectedOwnerId::F05jl,
        commit_owner: SelectedCommitOwner::Joc,
        fields: &[
            "temporal",
            "geometry",
            "fit180",
            "profile_retry",
            "huffman",
            "id14_writer",
            "sequence",
        ],
    },
    GroupSpec {
        name: "body_proposals",
        id: SelectedOwnerProjectionGroupId::BodyProposals,
        executor: SelectedOwnerExecutor::F05l,
        receipt_owner: SelectedOwnerId::F05l,
        commit_owner: SelectedCommitOwner::PureValue,
        fields: &["id11_body", "id14_body"],
    },
    GroupSpec {
        name: "metadata_timeline",
        id: SelectedOwnerProjectionGroupId::MetadataTimeline,
        executor: SelectedOwnerExecutor::F13,
        receipt_owner: SelectedOwnerId::F13,
        commit_owner: SelectedCommitOwner::Metadata,
        fields: &[
            "id1_timer",
            "queue11",
            "queue1",
            "queue2",
            "queue14",
            "emdf_assembly",
        ],
    },
    GroupSpec {
        name: "codec_and_publication",
        id: SelectedOwnerProjectionGroupId::CodecAndPublication,
        executor: SelectedOwnerExecutor::F12ThenF15,
        receipt_owner: SelectedOwnerId::F15,
        commit_owner: SelectedCommitOwner::CodecAndOutput,
        fields: &[
            "codec_state",
            "integrity_receipt",
            "staging_stream",
            "durable_ledger",
            "published_au_count",
        ],
    },
];

pub fn validate_selected_owner_projection(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF01bColdAssetManifest,
) -> Result<ValidatedSelectedOwnerProjection, SelectedColdContractError> {
    let bound = manifest
        .bind_schema_asset(evidence, OWNER_PROJECTION_PATH)
        .map_err(SelectedColdContractError::Manifest)?;
    let document = parse_document(bound.bytes())?;
    let fields = document
        .as_object()
        .ok_or(SelectedColdContractError::Schema)?;
    if !exact_keys(fields, OWNER_PROJECTION_TOP_LEVEL_FIELDS)
        || string_field(fields, "schema")? != "selected-owner-projection.v1"
        || fields.get("private_key_required") != Some(&Value::Bool(false))
    {
        return Err(SelectedColdContractError::Schema);
    }
    validate_string_array(
        fields.get("authoritative_for"),
        &["F03W", "F04C", "F05R56", "F05JL", "F05L", "F13", "F15"],
    )?;

    let group_values = fields
        .get("field_groups")
        .and_then(Value::as_array)
        .filter(|values| values.len() == GROUP_SPECS.len())
        .ok_or(SelectedColdContractError::Shape)?;
    let mut groups = Vec::with_capacity(GROUP_SPECS.len());
    for (value, expected) in group_values.iter().zip(GROUP_SPECS) {
        let group = value.as_object().ok_or(SelectedColdContractError::Shape)?;
        if !exact_keys(
            group,
            &[
                "group",
                "executor",
                "receipt_owner",
                "commit_owner",
                "fields",
            ],
        ) || string_field(group, "group")? != expected.name
            || string_field(group, "executor")? != expected.executor.as_str()
            || string_field(group, "receipt_owner")? != expected.receipt_owner.as_str()
            || string_field(group, "commit_owner")? != expected.commit_owner.as_str()
        {
            return Err(SelectedColdContractError::Schema);
        }
        validate_string_array(group.get("fields"), expected.fields)?;
        groups.push(SelectedOwnerProjectionGroup {
            id: expected.id,
            executor: expected.executor,
            receipt_owner: expected.receipt_owner,
            commit_owner: expected.commit_owner,
            fields: expected.fields,
        });
    }
    validate_owner_projection_rules(fields)?;
    let groups = groups
        .try_into()
        .map_err(|_| SelectedColdContractError::Shape)?;

    Ok(ValidatedSelectedOwnerProjection {
        manifest_sha256: bound.manifest_sha256(),
        provider_sha256: ProviderSha256::digest(bound.bytes()),
        groups,
    })
}

#[derive(Clone, Copy)]
struct GroupSpec {
    name: &'static str,
    id: SelectedOwnerProjectionGroupId,
    executor: SelectedOwnerExecutor,
    receipt_owner: SelectedOwnerId,
    commit_owner: SelectedCommitOwner,
    fields: &'static [&'static str],
}

impl SelectedOwnerExecutor {
    const fn as_str(self) -> &'static str {
        match self {
            Self::F03w => "F03W",
            Self::F04c => "F04C",
            Self::F05r56 => "F05R56",
            Self::F05jl => "F05JL",
            Self::F05l => "F05L",
            Self::F13 => "F13",
            Self::F12ThenF15 => "F12_then_F15",
        }
    }
}

impl SelectedOwnerId {
    const fn as_str(self) -> &'static str {
        match self {
            Self::F03w => "F03W",
            Self::F04c => "F04C",
            Self::F05r56 => "F05R56",
            Self::F05jl => "F05JL",
            Self::F05l => "F05L",
            Self::F13 => "F13",
            Self::F15 => "F15",
        }
    }
}

impl SelectedCommitOwner {
    const fn as_str(self) -> &'static str {
        match self {
            Self::SpatialControl => "spatial_control",
            Self::SpatialAudio => "spatial_audio",
            Self::Joc => "joc",
            Self::PureValue => "none_pure_value",
            Self::Metadata => "metadata",
            Self::CodecAndOutput => "codec_and_output",
        }
    }
}

fn validate_owner_projection_rules(
    fields: &Map<String, Value>,
) -> Result<(), SelectedColdContractError> {
    let admission = object_field(fields, "u04_admission")?;
    if !exact_keys(
        admission,
        &[
            "execute_status",
            "skip_status",
            "selected_count_does_not_override_status",
        ],
    ) || admission.get("selected_count_does_not_override_status") != Some(&Value::Bool(true))
    {
        return Err(SelectedColdContractError::Schema);
    }
    validate_string_array(admission.get("execute_status"), &["CAPACITY_REACHED"])?;
    validate_string_array(admission.get("skip_status"), &["CANDIDATES_EXHAUSTED"])?;

    let j05 = object_field(fields, "j05")?;
    if !exact_keys(
        j05,
        &[
            "frontend_to_source",
            "type10",
            "targets",
            "executor",
            "qmf_reshape",
            "downstream",
        ],
    ) || string_field(j05, "type10")? != "LFE[g-2] concatenated with component15[g]"
        || string_field(j05, "targets")? != "F05R56 target_words[g]"
        || string_field(j05, "executor")? != "F04C"
        || string_field(j05, "qmf_reshape")?
            != "for object o=0..14, slot s=0..47, bin b=0..63: joc[o][s][b] = target_qmf[o//3][s][(o%3)*64+b]; inverse writes the exact same cell"
        || string_field(j05, "downstream")?
            != "F05JL consumes reshaped 15x48x64 projection; must not rerun receiver or QMF"
    {
        return Err(SelectedColdContractError::Schema);
    }
    let frontend_to_source = object_field(j05, "frontend_to_source")?;
    if !exact_keys(
        frontend_to_source,
        &["source_index", "dispatch", "valid_when"],
    ) || string_field(frontend_to_source, "source_index")? != "g-2"
        || string_field(frontend_to_source, "dispatch")? != "g-1"
        || string_field(frontend_to_source, "valid_when")? != "g>=2"
        || string_field(fields, "queue_rule")?
            != "Only F13 mutates metadata queues. F05L/F05JL produce immutable bodies."
        || string_field(fields, "commit_rule")?
            != "F15 coordinates one source-root commit; leaf executor counts remain exactly one."
    {
        return Err(SelectedColdContractError::Schema);
    }
    Ok(())
}

fn parse_document(bytes: &[u8]) -> Result<Value, SelectedColdContractError> {
    std::str::from_utf8(bytes).map_err(|_| SelectedColdContractError::Utf8)?;
    serde_json::from_slice(bytes).map_err(|_| SelectedColdContractError::JsonSyntax)
}

fn object_field<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Map<String, Value>, SelectedColdContractError> {
    fields
        .get(name)
        .and_then(Value::as_object)
        .ok_or(SelectedColdContractError::Shape)
}

fn string_field<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, SelectedColdContractError> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .ok_or(SelectedColdContractError::Shape)
}

fn exact_dimension(
    fields: &Map<String, Value>,
    name: &str,
    expected: u8,
) -> Result<u8, SelectedColdContractError> {
    let value = fields
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or(SelectedColdContractError::Shape)?;
    if value != expected {
        return Err(SelectedColdContractError::Schema);
    }
    Ok(value)
}

fn validate_string_array(
    value: Option<&Value>,
    expected: &[&str],
) -> Result<(), SelectedColdContractError> {
    let values = value
        .and_then(Value::as_array)
        .filter(|values| values.len() == expected.len())
        .ok_or(SelectedColdContractError::Shape)?;
    if values
        .iter()
        .zip(expected)
        .any(|(value, expected)| value.as_str() != Some(*expected))
    {
        return Err(SelectedColdContractError::Schema);
    }
    Ok(())
}

fn exact_keys(fields: &Map<String, Value>, expected: &[&str]) -> bool {
    fields.len() == expected.len() && expected.iter().all(|key| fields.contains_key(*key))
}
