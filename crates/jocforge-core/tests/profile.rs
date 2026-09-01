// pattern: Functional Core

use jocforge_core::{JocProfile, profile_spec};

#[test]
fn mandatory_profiles_have_distinct_public_input_orders() {
    let flat = profile_spec(JocProfile::Idx1);
    let height = profile_spec(JocProfile::Idx2);

    assert_eq!(flat.downmix_index, 1);
    assert_eq!(flat.input_labels, ["L", "R", "C", "Ls", "Rs", "Lrs", "Rrs"]);
    assert_eq!(height.downmix_index, 2);
    assert_eq!(
        height.input_labels,
        ["L", "R", "C", "Ls", "Rs", "Tfl", "Tfr"]
    );
    assert_ne!(flat.input_labels, height.input_labels);
    assert_ne!(flat.channel_map, height.channel_map);
}
