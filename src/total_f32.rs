use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

#[derive(Default, Debug, Copy, Clone)]
pub struct TotalF32(pub f32);

impl TotalF32 {
    /// Normalises the float value to an i32
    fn normalise(&self) -> i32 {
        let val = self.0.to_bits() as i32;

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
        val ^ (((val >> 31) as u32) >> 1) as i32
    }
}

impl From<TotalF32> for f32 {
    fn from(TotalF32(f): TotalF32) -> Self {
        f
    }
}

impl From<f32> for TotalF32 {
    fn from(f: f32) -> Self {
        TotalF32(f.into())
    }
}

impl PartialEq for TotalF32 {
    fn eq(&self, other: &Self) -> bool {
        self.normalise() == other.normalise()
    }
}

impl Eq for TotalF32 {}

impl PartialOrd for TotalF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.normalise().cmp(&other.normalise())
    }
}

impl Hash for TotalF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // this value is used for the hash so that we can enforce a constraint from Hash:
        //     When implementing both Hash and Eq, it is important that the following property holds:
        //     k1 == k2 -> hash(k1) == hash(k2)
        //
        // by comparing and hashing the same integer value we guarentee that this property holds
        self.normalise().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl core::ops::Neg for TotalF32 {
        type Output = Self;

        fn neg(self: Self) -> Self {
            let Self(f) = self;
            Self(f.neg())
        }
    }

    #[test]
    fn test_f32_from_total_f32() {
        let f: f32 = 5.0;
        let v = TotalF32(f);
        let v_f: f32 = v.into();
        assert_eq!(v_f, f);
    }

    #[test]
    fn test_total_f32_from_f32() {
        let f: f32 = 5.0;
        let v: TotalF32 = f.into();
        assert_eq!(v, TotalF32(f));
    }

    #[test]
    /// Adapted from https://github.com/rust-lang/rust/pull/72568/files
    fn test_total_f32_cmp() {
        use core::cmp::Ordering;

        fn quiet_bit_mask() -> u32 {
            1 << (f32::MANTISSA_DIGITS - 2)
        }

        fn min_subnorm() -> TotalF32 {
            TotalF32(f32::MIN_POSITIVE / f32::powf(2.0, f32::MANTISSA_DIGITS as f32 - 1.0))
        }

        fn max_subnorm() -> TotalF32 {
            TotalF32(f32::MIN_POSITIVE - min_subnorm().0)
        }

        fn q_nan() -> TotalF32 {
            TotalF32(f32::from_bits(f32::NAN.to_bits() | quiet_bit_mask()))
        }

        fn s_nan() -> TotalF32 {
            TotalF32(f32::from_bits(
                (f32::NAN.to_bits() & !quiet_bit_mask()) + 42,
            ))
        }

        assert_eq!(Ordering::Equal, (-q_nan()).cmp(&-q_nan()));
        assert_eq!(Ordering::Equal, (-s_nan()).cmp(&-s_nan()));
        assert_eq!(
            Ordering::Equal,
            (TotalF32(-f32::INFINITY)).cmp(&TotalF32(-f32::INFINITY))
        );
        assert_eq!(
            Ordering::Equal,
            (TotalF32(-f32::MAX)).cmp(&TotalF32(-f32::MAX))
        );
        assert_eq!(Ordering::Equal, (TotalF32(-2.5_f32)).cmp(&TotalF32(-2.5)));
        assert_eq!(Ordering::Equal, (TotalF32(-1.0_f32)).cmp(&TotalF32(-1.0)));
        assert_eq!(Ordering::Equal, (TotalF32(-1.5_f32)).cmp(&TotalF32(-1.5)));
        assert_eq!(Ordering::Equal, (TotalF32(-0.5_f32)).cmp(&TotalF32(-0.5)));
        assert_eq!(
            Ordering::Equal,
            (TotalF32(-f32::MIN_POSITIVE)).cmp(&TotalF32(-f32::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Equal, (-max_subnorm()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Equal, (-min_subnorm()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Equal, (TotalF32(-0.0_f32)).cmp(&TotalF32(-0.0)));
        assert_eq!(Ordering::Equal, TotalF32(0.0_f32).cmp(&TotalF32(0.0)));
        assert_eq!(Ordering::Equal, min_subnorm().cmp(&min_subnorm()));
        assert_eq!(Ordering::Equal, max_subnorm().cmp(&max_subnorm()));
        assert_eq!(
            Ordering::Equal,
            TotalF32(f32::MIN_POSITIVE).cmp(&TotalF32(f32::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Equal, TotalF32(0.5_f32).cmp(&TotalF32(0.5)));
        assert_eq!(Ordering::Equal, TotalF32(1.0_f32).cmp(&TotalF32(1.0)));
        assert_eq!(Ordering::Equal, TotalF32(1.5_f32).cmp(&TotalF32(1.5)));
        assert_eq!(Ordering::Equal, TotalF32(2.5_f32).cmp(&TotalF32(2.5)));
        assert_eq!(Ordering::Equal, TotalF32(f32::MAX).cmp(&TotalF32(f32::MAX)));
        assert_eq!(
            Ordering::Equal,
            TotalF32(f32::INFINITY).cmp(&TotalF32(f32::INFINITY))
        );
        assert_eq!(Ordering::Equal, s_nan().cmp(&s_nan()));
        assert_eq!(Ordering::Equal, q_nan().cmp(&q_nan()));

        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-s_nan()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-f32::INFINITY)));
        assert_eq!(
            Ordering::Less,
            (TotalF32(-f32::INFINITY)).cmp(&TotalF32(-f32::MAX))
        );
        assert_eq!(Ordering::Less, (TotalF32(-f32::MAX)).cmp(&TotalF32(-2.5)));
        assert_eq!(Ordering::Less, (TotalF32(-2.5_f32)).cmp(&TotalF32(-1.5)));
        assert_eq!(Ordering::Less, (TotalF32(-1.5_f32)).cmp(&TotalF32(-1.0)));
        assert_eq!(Ordering::Less, (TotalF32(-1.0_f32)).cmp(&TotalF32(-0.5)));
        assert_eq!(
            Ordering::Less,
            (TotalF32(-0.5_f32)).cmp(&TotalF32(-f32::MIN_POSITIVE))
        );
        assert_eq!(
            Ordering::Less,
            (TotalF32(-f32::MIN_POSITIVE)).cmp(&-max_subnorm())
        );
        assert_eq!(Ordering::Less, (-max_subnorm()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Less, (-min_subnorm()).cmp(&TotalF32(-0.0)));
        assert_eq!(Ordering::Less, (TotalF32(-0.0_f32)).cmp(&TotalF32(0.0)));
        assert_eq!(Ordering::Less, TotalF32(0.0_f32).cmp(&min_subnorm()));
        assert_eq!(Ordering::Less, min_subnorm().cmp(&max_subnorm()));
        assert_eq!(
            Ordering::Less,
            max_subnorm().cmp(&TotalF32(f32::MIN_POSITIVE))
        );
        assert_eq!(
            Ordering::Less,
            TotalF32(f32::MIN_POSITIVE).cmp(&TotalF32(0.5))
        );
        assert_eq!(Ordering::Less, TotalF32(0.5_f32).cmp(&TotalF32(1.0)));
        assert_eq!(Ordering::Less, TotalF32(1.0_f32).cmp(&TotalF32(1.5)));
        assert_eq!(Ordering::Less, TotalF32(1.5_f32).cmp(&TotalF32(2.5)));
        assert_eq!(Ordering::Less, TotalF32(2.5_f32).cmp(&TotalF32(f32::MAX)));
        assert_eq!(
            Ordering::Less,
            TotalF32(f32::MAX).cmp(&TotalF32(f32::INFINITY))
        );
        assert_eq!(Ordering::Less, TotalF32(f32::INFINITY).cmp(&s_nan()));
        assert_eq!(Ordering::Less, s_nan().cmp(&q_nan()));

        assert_eq!(Ordering::Greater, (-s_nan()).cmp(&-q_nan()));
        assert_eq!(Ordering::Greater, (TotalF32(-f32::INFINITY)).cmp(&-s_nan()));
        assert_eq!(
            Ordering::Greater,
            (TotalF32(-f32::MAX)).cmp(&TotalF32(-f32::INFINITY))
        );
        assert_eq!(
            Ordering::Greater,
            (TotalF32(-2.5_f32)).cmp(&TotalF32(-f32::MAX))
        );
        assert_eq!(Ordering::Greater, (TotalF32(-1.5_f32)).cmp(&TotalF32(-2.5)));
        assert_eq!(Ordering::Greater, (TotalF32(-1.0_f32)).cmp(&TotalF32(-1.5)));
        assert_eq!(Ordering::Greater, (TotalF32(-0.5_f32)).cmp(&TotalF32(-1.0)));
        assert_eq!(
            Ordering::Greater,
            (TotalF32(-f32::MIN_POSITIVE)).cmp(&TotalF32(-0.5))
        );
        assert_eq!(
            Ordering::Greater,
            (-max_subnorm()).cmp(&TotalF32(-f32::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Greater, (-min_subnorm()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Greater, (TotalF32(-0.0_f32)).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Greater, TotalF32(0.0_f32).cmp(&TotalF32(-0.0)));
        assert_eq!(Ordering::Greater, min_subnorm().cmp(&TotalF32(0.0)));
        assert_eq!(Ordering::Greater, max_subnorm().cmp(&min_subnorm()));
        assert_eq!(
            Ordering::Greater,
            TotalF32(f32::MIN_POSITIVE).cmp(&max_subnorm())
        );
        assert_eq!(
            Ordering::Greater,
            TotalF32(0.5_f32).cmp(&TotalF32(f32::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Greater, TotalF32(1.0_f32).cmp(&TotalF32(0.5)));
        assert_eq!(Ordering::Greater, TotalF32(1.5_f32).cmp(&TotalF32(1.0)));
        assert_eq!(Ordering::Greater, TotalF32(2.5_f32).cmp(&TotalF32(1.5)));
        assert_eq!(Ordering::Greater, TotalF32(f32::MAX).cmp(&TotalF32(2.5)));
        assert_eq!(
            Ordering::Greater,
            TotalF32(f32::INFINITY).cmp(&TotalF32(f32::MAX))
        );
        assert_eq!(Ordering::Greater, s_nan().cmp(&TotalF32(f32::INFINITY)));
        assert_eq!(Ordering::Greater, q_nan().cmp(&s_nan()));

        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-s_nan()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-f32::INFINITY)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-f32::MAX)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-2.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-1.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-1.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-0.5)));
        assert_eq!(
            Ordering::Less,
            (-q_nan()).cmp(&TotalF32(-f32::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(-0.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(0.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&min_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&max_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(f32::MIN_POSITIVE)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(0.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(1.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(1.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(2.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(f32::MAX)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF32(f32::INFINITY)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&s_nan()));

        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-f32::INFINITY)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-f32::MAX)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-2.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-1.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-1.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-0.5)));
        assert_eq!(
            Ordering::Less,
            (-s_nan()).cmp(&TotalF32(-f32::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(-0.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(0.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&min_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&max_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(f32::MIN_POSITIVE)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(0.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(1.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(1.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(2.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(f32::MAX)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF32(f32::INFINITY)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&s_nan()));
    }
}
