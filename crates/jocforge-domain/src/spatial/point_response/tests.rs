// pattern: Imperative Shell
use super::{RESPONSE_THRESHOLD_BITS, finite_word, threshold_response};

#[test]
fn response_threshold_clears_only_strictly_lower_words() {
    let threshold = finite_word(RESPONSE_THRESHOLD_BITS).unwrap();
    let below = finite_word(0x3586_37bc).unwrap();
    let equal = finite_word(0x3586_37bd).unwrap();
    let above = finite_word(0x3586_37be).unwrap();

    assert_eq!(threshold_response(below, threshold), 0);
    assert_eq!(threshold_response(equal, threshold), 0x3586_37bd);
    assert_eq!(threshold_response(above, threshold), 0x3586_37be);
}
