// pattern: Imperative Shell
//! Verifies caller-root loading of the Wash-approved selected F04 receiver asset subset.

use jocforge_io::provider::load_selected_spatial_audio_provider_bundle;
use std::path::PathBuf;

#[test]
fn approved_receiver_parameter_bundle_loads_from_its_hash_bound_root() {
    let package_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS/F04-RECEIVER-APPROVED-PARAMETERS-V1");
    let bundle = load_selected_spatial_audio_provider_bundle(package_root)
        .expect("the Wash-approved five-file F04 package validates from its caller root");

    assert_eq!(
        bundle.manifest_sha256().to_hex(),
        "161147c7959083738f5426b2280457454b08b73ee58a075322c33880cecc818b"
    );
    let _receiver_providers = bundle.spatial_audio_joc_providers();
}
