use jocforge_domain::drc::none_drc_dialnorm_offset::{
    NoneDrcDialnormOffsetError, none_drc_dialnorm_offset_word,
};
use sha2::{Digest, Sha256};

const AUTHORIZED_MATH_PARAMETERS: &[u8] =
    include_bytes!("fixtures/none_drc/authorized_math_parameters.json");
const PUBLIC_DISTRIBUTION_SCOPE: &[u8] =
    include_bytes!("fixtures/none_drc/public_distribution_scope_v2/SCOPE.json");
const AUTHORIZED_MATH_PARAMETERS_SHA256: &str =
    "f1a552443315414ad499118d905ef9fe6612f178dfe28c16aa5d9fa6b2002166";
const AUTHORIZED_MATH_PARAMETERS_BYTES: usize = 2_010;

#[test]
fn exact_math_provider_is_hash_bound_and_all_selected_words_match() {
    assert_eq!(
        AUTHORIZED_MATH_PARAMETERS.len(),
        AUTHORIZED_MATH_PARAMETERS_BYTES
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(AUTHORIZED_MATH_PARAMETERS)),
        AUTHORIZED_MATH_PARAMETERS_SHA256
    );

    let provider: serde_json::Value =
        serde_json::from_slice(AUTHORIZED_MATH_PARAMETERS).expect("approved math provider parses");
    assert_eq!(
        provider["schema"].as_str(),
        Some("clean.none-drc.math-functional-values.v1")
    );
    let domain = provider["dialnorm_domain"].as_array().unwrap();
    assert_eq!(domain.len(), 2);
    assert_eq!(domain[0].as_u64(), Some(1));
    assert_eq!(domain[1].as_u64(), Some(31));
    assert_eq!(
        provider["dialnorm_formula"].as_str(),
        Some("R32((31-d)/144)")
    );
    let expected_words = provider["dialnorm_words"].as_array().unwrap();
    assert_eq!(expected_words.len(), 31);

    for (index, expected_word) in expected_words.iter().enumerate() {
        let dialnorm = i32::try_from(index + 1).unwrap();
        assert_eq!(
            none_drc_dialnorm_offset_word(dialnorm),
            Ok(u32::try_from(expected_word.as_u64().unwrap()).unwrap()),
            "dialnorm={dialnorm}"
        );
    }

    let distribution_scope: serde_json::Value =
        serde_json::from_slice(PUBLIC_DISTRIBUTION_SCOPE).unwrap();
    assert_eq!(
        distribution_scope["decision"].as_str(),
        Some("PUBLIC_DISTRIBUTION_AUTHORIZED_FOR_NONPRIVATE_FUNCTIONAL_AND_MATHEMATICAL_MATERIAL")
    );
    assert_eq!(
        distribution_scope["functional_source_package"]["manifest_sha256"].as_str(),
        Some("c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a")
    );
    assert_eq!(
        distribution_scope["previous_internal_only_labels"].as_str(),
        Some("SUPERSEDED_FOR_THIS_SCOPE_BY_USER_CLARIFICATION")
    );
    assert_eq!(
        distribution_scope["mathematical_provider_source"]["git_blob_sha1"].as_str(),
        Some("8ee95ed63232cda8a4a431b2be3460ba93563f98")
    );
    assert_eq!(
        distribution_scope["original_hash_sealed_packages_modified"],
        serde_json::Value::Bool(false)
    );
}

#[test]
fn values_outside_selected_dialnorm_domain_are_unsupported_profiles() {
    for dialnorm in [-1, 0, 32, 256] {
        assert_eq!(
            none_drc_dialnorm_offset_word(dialnorm),
            Err(NoneDrcDialnormOffsetError::UnsupportedProfile),
            "dialnorm={dialnorm}"
        );
    }
    assert_eq!(
        NoneDrcDialnormOffsetError::UnsupportedProfile.code(),
        "UNSUPPORTED_PROFILE"
    );
}
