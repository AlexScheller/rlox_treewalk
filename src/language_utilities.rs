// The use case for this simple discriminant equality check is the Token type, an enum containing
// heterogenous values whose equality comparators cannot be automatically derived. In the cvase of
// this enum, we don't actually care about comparing the equality of the values anyway.
pub fn enum_variant_equal<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
