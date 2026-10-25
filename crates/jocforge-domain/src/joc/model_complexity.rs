// pattern: Functional Core
//! Selected F13JO model complexity projection from a validated ID14 object count.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelJocComplexityU8(u8);

impl ModelJocComplexityU8 {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Computes the F13JO model value `U8(joc_object_count + 1)`.
///
/// The caller must supply the semantic object count from an already validated
/// selected EMDF/ID14 model. This leaf performs no EMDF parsing, profile
/// admission, native-source join, carrier selection or error-code mapping.
pub fn derive_model_joc_complexity(joc_object_count: u32) -> Option<ModelJocComplexityU8> {
    let complexity = joc_object_count.checked_add(1)?;
    Some(ModelJocComplexityU8(u8::try_from(complexity).ok()?))
}
