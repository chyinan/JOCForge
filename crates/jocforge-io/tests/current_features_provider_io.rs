// pattern: Imperative Shell
use jocforge_io::provider::load_selected_current_features_math_providers;
use std::path::PathBuf;

fn approved_asset_subset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1")
}

#[test]
fn loads_only_the_exact_approved_current_features_math_pair() {
    let providers = load_selected_current_features_math_providers(approved_asset_subset_root())
        .expect("approved CurrentFeatures assets load from the non-exhaustive subset");

    assert_eq!(providers.processor().sample_rate(), 48_000);
    assert_eq!(providers.processor().window_length(), 3_072);
    assert_eq!(providers.processor().coefficient_u32().len(), 2_048);
    assert_eq!(
        providers
            .processor()
            .receipt()
            .provider_file_sha256()
            .to_hex(),
        "c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b"
    );
    assert_eq!(providers.fft().fft_length(), 4_096);
    assert_eq!(providers.fft().stages().len(), 12);
    assert_eq!(
        providers.fft().receipt().provider_file_sha256().to_hex(),
        "aea85e4fb5dcb0354ac11790466d324f1020a8bf5db3f9d981bc46755751fb33"
    );
    assert_eq!(
        providers
            .processor()
            .receipt()
            .validated_manifest_lineage()
            .approved_source_manifest_sha256(),
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
}
