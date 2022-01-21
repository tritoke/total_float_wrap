use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

#[derive(Default, Debug, Copy, Clone)]
pub struct TotalF64(pub f64);

impl TotalF64 {
    /// Normalises the float value to an i64
    fn normalise(&self) -> i64 {
        let val = self.0.to_bits() as i64;

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
        val ^ (((val >> 63) as u64) >> 1) as i64
    }
}

impl From<TotalF64> for f64 {
    fn from(TotalF64(f): TotalF64) -> Self {
        f
    }
}

impl From<f64> for TotalF64 {
    fn from(f: f64) -> Self {
        TotalF64(f.into())
    }
}

impl PartialEq for TotalF64 {
    fn eq(&self, other: &Self) -> bool {
        self.normalise() == other.normalise()
    }
}

impl Eq for TotalF64 {}

impl PartialOrd for TotalF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.normalise().cmp(&other.normalise())
    }
}

impl Hash for TotalF64 {
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

    impl core::ops::Neg for TotalF64 {
        type Output = Self;

        fn neg(self: Self) -> Self {
            let Self(f) = self;
            Self(f.neg())
        }
    }

    #[test]
    fn test_f64_from_total_f64() {
        let f: f64 = 5.0;
        let v = TotalF64(f);
        let v_f: f64 = v.into();
        assert_eq!(v_f, f);
    }

    #[test]
    fn test_total_f64_from_f64() {
        let f: f64 = 5.0;
        let v: TotalF64 = f.into();
        assert_eq!(v, TotalF64(f));
    }

    #[test]
    /// Adapted from https://github.com/rust-lang/rust/pull/72568/files
    fn test_total_f64_cmp() {
        use core::cmp::Ordering;

        fn quiet_bit_mask() -> u64 {
            1 << (f64::MANTISSA_DIGITS - 2)
        }

        fn min_subnorm() -> TotalF64 {
            TotalF64(f64::MIN_POSITIVE / f64::powf(2.0, f64::MANTISSA_DIGITS as f64 - 1.0))
        }

        fn max_subnorm() -> TotalF64 {
            TotalF64(f64::MIN_POSITIVE - min_subnorm().0)
        }

        fn q_nan() -> TotalF64 {
            TotalF64(f64::from_bits(f64::NAN.to_bits() | quiet_bit_mask()))
        }

        fn s_nan() -> TotalF64 {
            TotalF64(f64::from_bits(
                (f64::NAN.to_bits() & !quiet_bit_mask()) + 42,
            ))
        }

        assert_eq!(Ordering::Equal, (-q_nan()).cmp(&-q_nan()));
        assert_eq!(Ordering::Equal, (-s_nan()).cmp(&-s_nan()));
        assert_eq!(
            Ordering::Equal,
            (TotalF64(-f64::INFINITY)).cmp(&TotalF64(-f64::INFINITY))
        );
        assert_eq!(
            Ordering::Equal,
            (TotalF64(-f64::MAX)).cmp(&TotalF64(-f64::MAX))
        );
        assert_eq!(Ordering::Equal, (TotalF64(-2.5_f64)).cmp(&TotalF64(-2.5)));
        assert_eq!(Ordering::Equal, (TotalF64(-1.0_f64)).cmp(&TotalF64(-1.0)));
        assert_eq!(Ordering::Equal, (TotalF64(-1.5_f64)).cmp(&TotalF64(-1.5)));
        assert_eq!(Ordering::Equal, (TotalF64(-0.5_f64)).cmp(&TotalF64(-0.5)));
        assert_eq!(
            Ordering::Equal,
            (TotalF64(-f64::MIN_POSITIVE)).cmp(&TotalF64(-f64::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Equal, (-max_subnorm()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Equal, (-min_subnorm()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Equal, (TotalF64(-0.0_f64)).cmp(&TotalF64(-0.0)));
        assert_eq!(Ordering::Equal, TotalF64(0.0_f64).cmp(&TotalF64(0.0)));
        assert_eq!(Ordering::Equal, min_subnorm().cmp(&min_subnorm()));
        assert_eq!(Ordering::Equal, max_subnorm().cmp(&max_subnorm()));
        assert_eq!(
            Ordering::Equal,
            TotalF64(f64::MIN_POSITIVE).cmp(&TotalF64(f64::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Equal, TotalF64(0.5_f64).cmp(&TotalF64(0.5)));
        assert_eq!(Ordering::Equal, TotalF64(1.0_f64).cmp(&TotalF64(1.0)));
        assert_eq!(Ordering::Equal, TotalF64(1.5_f64).cmp(&TotalF64(1.5)));
        assert_eq!(Ordering::Equal, TotalF64(2.5_f64).cmp(&TotalF64(2.5)));
        assert_eq!(Ordering::Equal, TotalF64(f64::MAX).cmp(&TotalF64(f64::MAX)));
        assert_eq!(
            Ordering::Equal,
            TotalF64(f64::INFINITY).cmp(&TotalF64(f64::INFINITY))
        );
        assert_eq!(Ordering::Equal, s_nan().cmp(&s_nan()));
        assert_eq!(Ordering::Equal, q_nan().cmp(&q_nan()));

        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-s_nan()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-f64::INFINITY)));
        assert_eq!(
            Ordering::Less,
            (TotalF64(-f64::INFINITY)).cmp(&TotalF64(-f64::MAX))
        );
        assert_eq!(Ordering::Less, (TotalF64(-f64::MAX)).cmp(&TotalF64(-2.5)));
        assert_eq!(Ordering::Less, (TotalF64(-2.5_f64)).cmp(&TotalF64(-1.5)));
        assert_eq!(Ordering::Less, (TotalF64(-1.5_f64)).cmp(&TotalF64(-1.0)));
        assert_eq!(Ordering::Less, (TotalF64(-1.0_f64)).cmp(&TotalF64(-0.5)));
        assert_eq!(
            Ordering::Less,
            (TotalF64(-0.5_f64)).cmp(&TotalF64(-f64::MIN_POSITIVE))
        );
        assert_eq!(
            Ordering::Less,
            (TotalF64(-f64::MIN_POSITIVE)).cmp(&-max_subnorm())
        );
        assert_eq!(Ordering::Less, (-max_subnorm()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Less, (-min_subnorm()).cmp(&TotalF64(-0.0)));
        assert_eq!(Ordering::Less, (TotalF64(-0.0_f64)).cmp(&TotalF64(0.0)));
        assert_eq!(Ordering::Less, TotalF64(0.0_f64).cmp(&min_subnorm()));
        assert_eq!(Ordering::Less, min_subnorm().cmp(&max_subnorm()));
        assert_eq!(
            Ordering::Less,
            max_subnorm().cmp(&TotalF64(f64::MIN_POSITIVE))
        );
        assert_eq!(
            Ordering::Less,
            TotalF64(f64::MIN_POSITIVE).cmp(&TotalF64(0.5))
        );
        assert_eq!(Ordering::Less, TotalF64(0.5_f64).cmp(&TotalF64(1.0)));
        assert_eq!(Ordering::Less, TotalF64(1.0_f64).cmp(&TotalF64(1.5)));
        assert_eq!(Ordering::Less, TotalF64(1.5_f64).cmp(&TotalF64(2.5)));
        assert_eq!(Ordering::Less, TotalF64(2.5_f64).cmp(&TotalF64(f64::MAX)));
        assert_eq!(
            Ordering::Less,
            TotalF64(f64::MAX).cmp(&TotalF64(f64::INFINITY))
        );
        assert_eq!(Ordering::Less, TotalF64(f64::INFINITY).cmp(&s_nan()));
        assert_eq!(Ordering::Less, s_nan().cmp(&q_nan()));

        assert_eq!(Ordering::Greater, (-s_nan()).cmp(&-q_nan()));
        assert_eq!(Ordering::Greater, (TotalF64(-f64::INFINITY)).cmp(&-s_nan()));
        assert_eq!(
            Ordering::Greater,
            (TotalF64(-f64::MAX)).cmp(&TotalF64(-f64::INFINITY))
        );
        assert_eq!(
            Ordering::Greater,
            (TotalF64(-2.5_f64)).cmp(&TotalF64(-f64::MAX))
        );
        assert_eq!(Ordering::Greater, (TotalF64(-1.5_f64)).cmp(&TotalF64(-2.5)));
        assert_eq!(Ordering::Greater, (TotalF64(-1.0_f64)).cmp(&TotalF64(-1.5)));
        assert_eq!(Ordering::Greater, (TotalF64(-0.5_f64)).cmp(&TotalF64(-1.0)));
        assert_eq!(
            Ordering::Greater,
            (TotalF64(-f64::MIN_POSITIVE)).cmp(&TotalF64(-0.5))
        );
        assert_eq!(
            Ordering::Greater,
            (-max_subnorm()).cmp(&TotalF64(-f64::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Greater, (-min_subnorm()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Greater, (TotalF64(-0.0_f64)).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Greater, TotalF64(0.0_f64).cmp(&TotalF64(-0.0)));
        assert_eq!(Ordering::Greater, min_subnorm().cmp(&TotalF64(0.0)));
        assert_eq!(Ordering::Greater, max_subnorm().cmp(&min_subnorm()));
        assert_eq!(
            Ordering::Greater,
            TotalF64(f64::MIN_POSITIVE).cmp(&max_subnorm())
        );
        assert_eq!(
            Ordering::Greater,
            TotalF64(0.5_f64).cmp(&TotalF64(f64::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Greater, TotalF64(1.0_f64).cmp(&TotalF64(0.5)));
        assert_eq!(Ordering::Greater, TotalF64(1.5_f64).cmp(&TotalF64(1.0)));
        assert_eq!(Ordering::Greater, TotalF64(2.5_f64).cmp(&TotalF64(1.5)));
        assert_eq!(Ordering::Greater, TotalF64(f64::MAX).cmp(&TotalF64(2.5)));
        assert_eq!(
            Ordering::Greater,
            TotalF64(f64::INFINITY).cmp(&TotalF64(f64::MAX))
        );
        assert_eq!(Ordering::Greater, s_nan().cmp(&TotalF64(f64::INFINITY)));
        assert_eq!(Ordering::Greater, q_nan().cmp(&s_nan()));

        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-s_nan()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-f64::INFINITY)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-f64::MAX)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-2.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-1.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-1.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-0.5)));
        assert_eq!(
            Ordering::Less,
            (-q_nan()).cmp(&TotalF64(-f64::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(-0.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(0.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&min_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&max_subnorm()));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(f64::MIN_POSITIVE)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(0.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(1.0)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(1.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(2.5)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(f64::MAX)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&TotalF64(f64::INFINITY)));
        assert_eq!(Ordering::Less, (-q_nan()).cmp(&s_nan()));

        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-f64::INFINITY)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-f64::MAX)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-2.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-1.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-1.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-0.5)));
        assert_eq!(
            Ordering::Less,
            (-s_nan()).cmp(&TotalF64(-f64::MIN_POSITIVE))
        );
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&-max_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&-min_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(-0.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(0.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&min_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&max_subnorm()));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(f64::MIN_POSITIVE)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(0.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(1.0)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(1.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(2.5)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(f64::MAX)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&TotalF64(f64::INFINITY)));
        assert_eq!(Ordering::Less, (-s_nan()).cmp(&s_nan()));
    }
}
