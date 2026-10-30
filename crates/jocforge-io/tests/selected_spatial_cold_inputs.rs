// pattern: Imperative Shell
use jocforge_domain::{
    profile::{
        AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
        EncodingProfile, JocDownmixProfile, SubstreamTopology,
    },
    spatial::F04ParameterProviderError,
};
use jocforge_io::provider::{
    SelectedSpatialColdProviderLoadError, SelectedSpatialColdProviderRoots,
    load_selected_spatial_cold_provider_candidate,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const COLD_BUNDLE: &str = "F01B-F03W-COLD-STATE-ASSETS-V1";
const PROVIDER_BUNDLE: &str = "ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1";
const COLD_FILES: &[&str] = &[
    "MANIFEST.json",
    "parameters/adm-installed48-state-v1.json",
    "parameters/f03-selected-target-cold-v1.json",
    "parameters/selected-owner-projection-v1.json",
    "parameters/window-q15.json",
    "parameters/f01e-selected-constructor-v1.json",
];
const PROVIDER_FILES: &[&str] = &[
    "MANIFEST.json",
    "parameters/diffusion-mathematical-v1.json",
    "parameters/diffusion-constructor-mathematical-v1.json",
    "parameters/component-parameter-mathematical-v1.json",
];

fn approved_bundle(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS")
        .join(name)
}

fn load(
    cold_root: &Path,
    provider_root: &Path,
    profile: EncodingProfile,
) -> Result<
    jocforge_io::provider::SelectedSpatialColdProviderCandidate,
    SelectedSpatialColdProviderLoadError,
> {
    load_selected_spatial_cold_provider_candidate(
        SelectedSpatialColdProviderRoots::new(provider_root, cold_root),
        profile,
    )
}

struct TemporaryBundles {
    root: PathBuf,
    cold: PathBuf,
    provider: PathBuf,
}

impl TemporaryBundles {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "jocforge-f01b-f03w-provider-test-{}-{nonce}",
            std::process::id()
        ));
        let cold = root.join("cold");
        let provider = root.join("provider");
        fs::create_dir_all(&cold).expect("temporary cold bundle is created");
        fs::create_dir_all(&provider).expect("temporary provider bundle is created");
        Self {
            root,
            cold,
            provider,
        }
    }

    fn copy_cold(&self, files: &[&str]) {
        copy_files(&approved_bundle(COLD_BUNDLE), &self.cold, files);
    }

    fn copy_provider(&self, files: &[&str]) {
        copy_files(&approved_bundle(PROVIDER_BUNDLE), &self.provider, files);
    }

    fn copy_all(&self) {
        self.copy_cold(COLD_FILES);
        self.copy_provider(PROVIDER_FILES);
    }
}

impl Drop for TemporaryBundles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn copy_files(source: &Path, destination: &Path, files: &[&str]) {
    for relative in files {
        let output = destination.join(relative);
        fs::create_dir_all(output.parent().expect("asset path has parent"))
            .expect("temporary asset parent is created");
        fs::copy(source.join(relative), output).expect("approved asset is copied");
    }
}

#[test]
fn loads_selected_cold_schemas_and_spatial_parameters_from_two_roots() {
    let candidate = load(
        &approved_bundle(COLD_BUNDLE),
        &approved_bundle(PROVIDER_BUNDLE),
        EncodingProfile::selected(),
    )
    .expect("approved selected cold and provider assets validate");

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
    assert_eq!(candidate.consumed_provider_lineages().len(), 3);
    assert!(candidate.consumed_provider_lineages().iter().all(|entry| {
        entry.manifest_sha256().to_hex()
            == "9b81b736e9dd110441d3b59333afac9cbb98886f8bf4ac9854b5d74bdf57783f"
    }));
    assert_eq!(candidate.declared_provider_entry_count(), 48);
    assert_eq!(candidate.unvalidated_provider_entry_count_open(), 45);
    assert!(candidate.transitive_provider_consumption_is_open());
    assert!(!candidate.is_runtime_provider_set());
}

#[test]
fn rejects_reusing_one_directory_for_both_bundle_roots() {
    let root = approved_bundle(COLD_BUNDLE);

    assert!(matches!(
        load(&root, &root, EncodingProfile::selected()),
        Err(SelectedSpatialColdProviderLoadError::PackageRootsNotDistinct)
    ));
}

#[test]
fn rejects_provider_manifest_before_payload_reads() {
    let bundles = TemporaryBundles::new();
    bundles.copy_cold(COLD_FILES);
    fs::write(bundles.provider.join("MANIFEST.json"), b"not-json")
        .expect("invalid provider manifest is written");

    assert!(matches!(
        load(
            &bundles.cold,
            &bundles.provider,
            EncodingProfile::selected()
        ),
        Err(SelectedSpatialColdProviderLoadError::F03wManifest(_))
    ));
}

#[test]
fn rejects_cold_manifest_before_cold_payload_reads() {
    let bundles = TemporaryBundles::new();
    fs::write(bundles.cold.join("MANIFEST.json"), b"not-json")
        .expect("invalid cold manifest is written");
    bundles.copy_provider(PROVIDER_FILES);

    assert!(matches!(
        load(
            &bundles.cold,
            &bundles.provider,
            EncodingProfile::selected()
        ),
        Err(SelectedSpatialColdProviderLoadError::ColdManifest(_))
    ));
}

#[test]
fn rejects_mutated_consumed_provider_bytes() {
    let bundles = TemporaryBundles::new();
    bundles.copy_all();
    let asset = bundles
        .provider
        .join("parameters/component-parameter-mathematical-v1.json");
    let mut bytes = fs::read(&asset).expect("component asset exists");
    bytes[0] ^= 1;
    fs::write(asset, bytes).expect("mutated provider bytes are written");

    assert!(matches!(
        load(
            &bundles.cold,
            &bundles.provider,
            EncodingProfile::selected()
        ),
        Err(SelectedSpatialColdProviderLoadError::ComponentParameters(_))
    ));
}

#[test]
fn rejects_missing_cold_payload_even_if_the_same_path_exists_in_provider_root() {
    let bundles = TemporaryBundles::new();
    let cold_files = COLD_FILES
        .iter()
        .copied()
        .filter(|path| *path != "parameters/f03-selected-target-cold-v1.json")
        .collect::<Vec<_>>();
    bundles.copy_cold(&cold_files);
    bundles.copy_provider(PROVIDER_FILES);
    fs::copy(
        approved_bundle(COLD_BUNDLE).join("parameters/f03-selected-target-cold-v1.json"),
        bundles
            .provider
            .join("parameters/f03-selected-target-cold-v1.json"),
    )
    .expect("cold target is also placed under the provider root");

    assert!(matches!(
        load(
            &bundles.cold,
            &bundles.provider,
            EncodingProfile::selected()
        ),
        Err(SelectedSpatialColdProviderLoadError::FileRead {
            bundle: jocforge_io::provider::SelectedProviderRoot::ColdState,
            relative_path: "parameters/f03-selected-target-cold-v1.json",
            ..
        })
    ));
}

#[test]
fn rejects_profile_mismatch_without_substitution() {
    let profile = EncodingProfile::new(
        AuthoredInputProfile::Unsupported,
        CoreCarrierProfile::SelectedFivePointOneSixCarrier,
        JocDownmixProfile::SelectedMode0Phase1Index3,
        SubstreamTopology::SelectedSingleIndependentZeroDependent,
        CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
        EmdfCarriageProfile::DeployedCompat,
    );
    assert!(matches!(
        load(
            &approved_bundle(COLD_BUNDLE),
            &approved_bundle(PROVIDER_BUNDLE),
            profile,
        ),
        Err(SelectedSpatialColdProviderLoadError::ComponentParameters(
            F04ParameterProviderError::UnsupportedProfile
        ))
    ));
}
