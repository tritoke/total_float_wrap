//! Floating point wrapper structs providing a total ordering for comparision and a safe
//! hash implementation, allowing it to be used as the key in a HashMap / HashSet etc.
//! Ordering agrees with `total_cmp` / IEEE 754's totalOrd.
//!
//! ## Example Usage
//!
//! ```rust
//! use std::collections::HashMap;
//! use total_float_wrap::TotalF64;
//!
//! let mut map: HashMap<TotalF64, u64> = HashMap::new();
//! 
//! map.insert(1.0.into(), 10);
//!
//! assert_eq!(map.get(&1.0.into()), Some(&10));
//! ```

mod total_f32;
pub use total_f32::TotalF32;

mod total_f64;
pub use total_f64::TotalF64;

trait FloatNormalise {
    type I;

    fn normalise(&self) -> Self::I;
}

// copied from https://github.com/rust-lang/rust/pull/72568/files
//
// In case of negatives, flip all the bits except the sign
// to achieve a similar layout as two's complement integers
//
// Why does this work? IEEE 754 floats consist of three fields:
// Sign bit, exponent and mantissa. The set of exponent and mantissa
// fields as a whole have the property that their bitwise order is
// equal to the numeric magnitude where the magnitude is defined.
// The magnitude is not normally defined on NaN values, but
// IEEE 754 totalOrder defines the NaN values also to follow the
// bitwise order. This leads to order explained in the doc comment.
// However, the representation of magnitude is the same for negative
// and positive numbers – only the sign bit is different.
// To easily compare the floats as signed integers, we need to
// flip the exponent and mantissa bits in case of negative numbers.
// We effectively convert the numbers to "two's complement" form.
//
// To do the flipping, we construct a mask and XOR against it.
// We branchlessly calculate an "all-ones except for the sign bit"
// mask from negative-signed values: right shifting sign-extends
// the integer, so we "fill" the mask with sign bits, and then
// convert to unsigned to push one more zero bit.
// On positive values, the mask is all zeros, so it's a no-op.
impl FloatNormalise for f64 {
    type I = i64;

    fn normalise(&self) -> Self::I {
        let val = self.0.to_bits() as i32;
        val ^ (((val >> 31) as u32) >> 1) as i32
    }
}

#[repr(transparent)]
struct TotalOrd<T: FloatNormalise>(T)
