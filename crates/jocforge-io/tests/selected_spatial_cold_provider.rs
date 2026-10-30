use jocforge_domain::{profile::EncodingProfile, spatial::F04ParameterProviderError};
use jocforge_io::provider::{
    SelectedSpatialColdProviderLoadError, SelectedSpatialColdProviderRoots,
    load_selected_spatial_cold_provider_candidate,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const F03W_BUNDLE: &str =
    "../../outputs/IMPLEMENTER_SUPPLEMENTS/ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1";
const COLD_BUNDLE: &str = "../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1";
const COLD_PAYLOADS: [&str; 5] = [
    "parameters/adm-installed48-state-v1.json",
    "parameters/f03-selected-target-cold-v1.json",
    "parameters/selected-owner-projection-v1.json",
    "parameters/window-q15.json",
    "parameters/f01e-selected-constructor-v1.json",
];
const PROVIDER_PAYLOADS: [&str; 3] = [
    "parameters/component-parameter-mathematical-v1.json",
    "parameters/diffusion-mathematical-v1.json",
    "parameters/diffusion-constructor-mathematical-v1.json",
];

fn approved_root(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load_from(
    f03w: &Path,
    cold: &Path,
) -> Result<
    jocforge_io::provider::SelectedSpatialColdProviderCandidate,
    SelectedSpatialColdProviderLoadError,
> {
    load_selected_spatial_cold_provider_candidate(
        SelectedSpatialColdProviderRoots::new(f03w, cold),
        EncodingProfile::selected(),
    )
}

struct TemporaryRoot(PathBuf);

impl TemporaryRoot {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jocforge-selected-provider-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("unique temporary root is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn copy_package(&self, source: &Path, payloads: &[&str]) {
        fs::copy(source.join("MANIFEST.json"), self.0.join("MANIFEST.json"))
            .expect("approved root manifest is copied");
        for relative in payloads {
            let destination = self.0.join(relative);
            fs::create_dir_all(destination.parent().unwrap())
                .expect("approved payload parent is created");
            fs::copy(source.join(relative), &destination)
                .expect("approved payload bytes are copied");
        }
    }
}

impl Drop for TemporaryRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn caller_roots_join_to_typed_cold_inputs_without_layout_or_callback_claims() {
    let f03w = approved_root(F03W_BUNDLE);
    let cold = approved_root(COLD_BUNDLE);
    assert!(
        !f03w
            .join("parameters/adm-installed48-state-v1.json")
            .exists()
    );

    let candidate = load_from(&f03w, &cold).expect("approved roots bind independently");
    assert_eq!(
        candidate.lineage().f03w_manifest_sha256().to_hex(),
        "9b81b736e9dd110441d3b59333afac9cbb98886f8bf4ac9854b5d74bdf57783f"
    );
    assert_eq!(
        candidate.lineage().cold_state_manifest_sha256().to_hex(),
        "d15672b723f8091b576c2228ed7685744f522706d89078050b6486a72a068372"
    );
    assert_eq!(
        candidate.installed48_schema().dimensions().source_count(),
        48
    );
    assert_eq!(candidate.owner_projection().groups().len(), 7);
    assert_eq!(candidate.f01e_cold().object_count(), 28);
    assert_eq!(
        candidate.f03_target_cold().current_id_u32(),
        &[u32::MAX; 15]
    );
    assert_eq!(candidate.window_q15().receiver_blend32().len(), 32);
    assert_eq!(
        candidate
            .component_parameters()
            .receipt()
            .provider_file_bytes(),
        1_077
    );
    assert_eq!(candidate.diffusion_parameters().ring_length(), 96);
    assert_eq!(candidate.consumed_provider_lineages().len(), 3);
    assert_eq!(candidate.unvalidated_provider_entry_count_open(), 45);
    assert!(candidate.transitive_provider_consumption_is_open());
    assert!(!candidate.is_runtime_provider_set());
    assert_eq!(
        candidate.f01e_cold().receipt().manifest_sha256(),
        candidate.lineage().cold_state_manifest_sha256()
    );
    assert_eq!(
        candidate
            .f03_target_cold()
            .receipt()
            .validated_manifest_lineage()
            .manifest_sha256(),
        candidate.lineage().cold_state_manifest_sha256()
    );
}

#[test]
fn altered_provider_manifest_is_rejected_at_its_own_root() {
    let f03w = TemporaryRoot::new("f03w-manifest");
    let approved = approved_root(F03W_BUNDLE);
    fs::copy(
        approved.join("MANIFEST.json"),
        f03w.path().join("MANIFEST.json"),
    )
    .expect("approved F03W manifest is copied");
    let manifest_path = f03w.path().join("MANIFEST.json");
    let mut changed = fs::read(&manifest_path).unwrap();
    changed[0] ^= 1;
    fs::write(manifest_path, changed).unwrap();

    let result = load_from(&f03w.0, &approved_root(COLD_BUNDLE));
    assert!(matches!(
        result,
        Err(SelectedSpatialColdProviderLoadError::F03wManifest(_))
    ));
}

#[test]
fn altered_cold_asset_is_rejected_by_its_cold_manifest_not_the_f03w_root() {
    let f03w = approved_root(F03W_BUNDLE);
    let cold = TemporaryRoot::new("cold-payload");
    let approved_cold = approved_root(COLD_BUNDLE);
    cold.copy_package(&approved_cold, &COLD_PAYLOADS);
    let projection_path = cold
        .path()
        .join("parameters/selected-owner-projection-v1.json");
    let mut changed = fs::read(&projection_path).unwrap();
    changed[0] ^= 1;
    fs::write(projection_path, changed).unwrap();

    let result = load_from(&f03w, cold.path());
    assert!(matches!(
        result,
        Err(SelectedSpatialColdProviderLoadError::OwnerProjection(_))
    ));
}

#[test]
fn altered_component_provider_asset_is_rejected_by_the_provider_root() {
    let provider = TemporaryRoot::new("provider-payload");
    let approved_provider = approved_root(F03W_BUNDLE);
    provider.copy_package(&approved_provider, &PROVIDER_PAYLOADS);
    let component_path = provider
        .path()
        .join("parameters/component-parameter-mathematical-v1.json");
    let mut changed = fs::read(&component_path).unwrap();
    changed[0] ^= 1;
    fs::write(component_path, changed).unwrap();

    let result = load_from(provider.path(), &approved_root(COLD_BUNDLE));
    assert!(matches!(
        result,
        Err(SelectedSpatialColdProviderLoadError::ComponentParameters(
            F04ParameterProviderError::ProviderIdentityMismatch
        ))
    ));
}
