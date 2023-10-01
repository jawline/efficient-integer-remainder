/// Implementation of division and modulo over a fixed divisor using pre-calculated multipliers and
/// shifts (avoiding a multi-cycle division). Described in: https://dl.acm.org/doi/pdf/10.1145/178243.178249.
///
/// Used to calculate the correct multiply and shift in a project where divide cannot be used.
use bnum::BUint;

// TODO: I can't figure out how to use Range * 2 to compute the double width multiplier and then
// truncate it to Range so we just use DoubleRange integers. If being used for more than this
// experiment this should be improved as it is very wasteful.

#[derive(Debug)]
struct Divider<const DOUBLE_INT_RANGE: usize> {
    divisor: BUint<DOUBLE_INT_RANGE>,
    multiplier: BUint<DOUBLE_INT_RANGE>,
    sh1: u32,
    sh2: u32,
}

impl<const DOUBLE_INT_RANGE: usize> Divider<DOUBLE_INT_RANGE> {
    fn new(divisor: BUint<DOUBLE_INT_RANGE>) -> Result<Self, String> {
        let const_one = BUint::<DOUBLE_INT_RANGE>::from(1 as u64);

        // We select our value l which we use to raise our input by 2^INT_RANGE+l
        // by taking the log2 and then incrementing it by one if that would be less than the
        // divisor.
        let l = {
            let initial_l = divisor.ilog(BUint::from(2 as u64));
            if const_one << initial_l < divisor {
                initial_l + 1
            } else {
                initial_l
            }
        };

        let two_to_the_l = const_one << l;
        let two_to_the_l_minus_divisor = two_to_the_l - divisor;

        let half_of_representable_range = const_one << (DOUBLE_INT_RANGE / 2);
        let multiplier = (half_of_representable_range * two_to_the_l_minus_divisor / divisor) + 1;

        // The final multiplier is meant to be in the range 0 <= multiplier < 2^INT_RANGE (i.e, it
        // should be a valid in in BUint<DOUBLE_INT_RANGE / 2>. We use this mask to delete any
        // carried bits.
        let valid_multiplier_mask =
            (BUint::from(1 as u64) << (DOUBLE_INT_RANGE / 2)) - BUint::from(1 as u64);
        let multiplier = multiplier & valid_multiplier_mask;

        let sh1 = l.min(1);
        let sh2 = if l == 0 { 0 } else { l - 1 };

        Ok(Self {
            divisor,
            multiplier,
            sh1,
            sh2,
        })
    }

    #[allow(dead_code)]
    fn divide(&self, dividend: BUint<DOUBLE_INT_RANGE>) -> BUint<DOUBLE_INT_RANGE> {
        let prod = dividend * self.multiplier;
        let prod_upper_half = prod >> (DOUBLE_INT_RANGE / 2);
        let quot = (prod_upper_half + ((dividend - prod_upper_half) >> self.sh1)) >> self.sh2;
        return quot;
    }

    #[allow(dead_code)]
    fn remainder(&self, dividend: BUint<DOUBLE_INT_RANGE>) -> BUint<DOUBLE_INT_RANGE> {
        dividend - (self.divisor * self.divide(dividend))
    }
}

fn main() {
    let p = BUint::<512>::from_str_radix("3fffffffffffffffffffffffffffffffb", 16).unwrap();
    let p = Divider::new(p).unwrap();
    println!("{:?}", p);
}

#[cfg(test)]
mod test {
    use super::*;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn general_test(dividend: u64, divisor: u64) {
        if divisor == 0 {
            return;
        }

        let t = Divider::new(BUint::<128>::from(divisor)).unwrap();
        let result: u64 = t.divide(BUint::from(dividend)).try_into().unwrap();
        let remainder_result: u64 = t.remainder(BUint::from(dividend)).try_into().unwrap();

        assert_eq!(dividend / divisor, result);
        assert_eq!(dividend % divisor, remainder_result);
    }

    /** Define an input of 256 bits (64 * 4) for quickcheck so we can fuzz p */
    #[derive(Debug, Clone)]
    struct LargeInput {
        data: [u64; 4],
    }

    impl Arbitrary for LargeInput {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                data: [u64::arbitrary(g); 4],
            }
        }
    }

    #[quickcheck]
    fn test_p(input_bytes: LargeInput) {
        let p = BUint::<512>::from_str_radix("3fffffffffffffffffffffffffffffffb", 16).unwrap();

        // I'm not sure why BUint<N> requires <N> u64's as input, maybe because at time of writing
        // Rust doesn't allow expressions over const generics and this interface at least requires
        // the correct number of bytes with some excess.
        let mut data = [0u64; 512];

        for i in 0..4 {
            data[i] = input_bytes.data[i];
        }

        let dividend = BUint::<512>::from(data);

        let t = Divider::new(p).unwrap();

        let result = t.divide(BUint::from(dividend));
        let remainder_result = t.remainder(BUint::from(dividend));

        assert_eq!(dividend / p, result);
        assert_eq!(dividend % p, remainder_result);
    }
}
