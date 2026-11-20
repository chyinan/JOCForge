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

#[test]
fn panner_weights_match_exact_rounding_on_the_q15_axis() {
    for coordinate in 0..=32767_u16 {
        let ratio = f32::from(coordinate) / 32767.0;
        let folded = if ratio > 0.5 { 1.0 - ratio } else { ratio };
        let angle = ((folded * f32::from_bits(super::PI_OVER_FOUR_BITS)) * 0.5) * 4.0;
        let exact = crate::numeric::exact_trigonometry::exact_r64_to_r32(angle).unwrap();
        let (cosine, sine) = super::rounded_pan_weights(angle).unwrap();
        assert_eq!(
            (cosine.to_bits(), sine.to_bits()),
            (exact.cosine.to_bits(), exact.sine.to_bits()),
            "coordinate {coordinate}"
        );
    }
}
