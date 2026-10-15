// pattern: Imperative Shell
use jocforge_io::provider::{read_package_manifest_file, read_provider_file};
use std::path::PathBuf;

fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../jocforge-domain/tests/fixtures/f01e_provider")
}

#[test]
fn io_adapters_read_immutable_manifest_and_provider_evidence() {
    let root = package_root();
    let manifest = read_package_manifest_file(root.join("MANIFEST.json")).unwrap();
    let provider = read_provider_file(root.join("f01e-selected-constructor-v1.json")).unwrap();
    assert_eq!(
        manifest.observed_sha256().to_hex(),
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    assert_eq!(provider.observed_byte_length(), 20138);
    assert_eq!(
        provider.observed_sha256().to_hex(),
        "b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb"
    );
}
