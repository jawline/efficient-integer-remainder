// Implementation of division and modulo over a fixed divisor using pre-calculated multipliers and
// shifts (avoiding a multi-cycle division). Described in: https://dl.acm.org/doi/pdf/10.1145/178243.178249 .
use bnum::BUint;

// TODO: I can't figure out how to use Range * 2 to compute the double width multiplier and then
// truncate it to Range so we just use DoubleRange integers. If being used for more than this
// experiment this should be improved as it is very wasteful.

struct Divider<const DOUBLE_INT_RANGE: usize> {
    divisor: BUint<DOUBLE_INT_RANGE>,
    multiplier: BUint<DOUBLE_INT_RANGE>,
    sh1: u32,
    sh2: u32,
}

impl<const DOUBLE_INT_RANGE: usize> Divider<DOUBLE_INT_RANGE> {
    fn new(divisor: BUint<DOUBLE_INT_RANGE>) -> Self {
        let valid_range_mask =
            (BUint::from(1 as u64) << (DOUBLE_INT_RANGE / 2)) - BUint::from(1 as u64);

        let l = divisor.ilog(BUint::from(2 as u64));

        println!("Divisor: {}, L: {}", divisor, l);

        let multiplier = {
            let half_range_plus_one = BUint::from(1 as u64) << (DOUBLE_INT_RANGE / 2);
            let two_to_l = BUint::<DOUBLE_INT_RANGE>::from(1 as u64) << l;
            println!("2^l: {}", two_to_l);
            let two_to_l_minus_d = two_to_l - divisor;
            println!("2^l - divisor: {}", two_to_l_minus_d);
            let result_off_by_one = half_range_plus_one * two_to_l_minus_d / divisor;
            result_off_by_one + 1
        };

        // We only want the bits of the multiplier within our int range
        let multiplier = multiplier & valid_range_mask;
        let sh1 = l.min(1);
        let sh2 = if l == 0 { 0 } else { l - 1 };

        Self {
            divisor,
            multiplier,
            sh1,
            sh2,
        }
    }

    fn divide(&self, dividend: BUint<DOUBLE_INT_RANGE>) -> BUint<DOUBLE_INT_RANGE> {
        let prod = dividend * self.multiplier;
        let prod_upper_half = prod >> (DOUBLE_INT_RANGE / 2);
        let quot = (prod_upper_half + ((dividend - prod_upper_half) >> self.sh1)) >> self.sh2;
        return quot;
    }

    fn remainder(&self, dividend: BUint<DOUBLE_INT_RANGE>) -> BUint<DOUBLE_INT_RANGE> {
        dividend - (self.divisor * self.divide(dividend))
    }
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tests() {
        for i in 1..10000 {
            println!("Initializing divider for {}", i);
            let t = Divider::new(BUint::<512>::from(i as u64));

            for k in 0..40 {
                let k = BUint::<512>::from(k as u64);
                let dividend_as_int: u64 = k.try_into().unwrap();
                let divisor_as_int: u64 = t.divisor.try_into().unwrap();
                let result: u64 = t.divide(k).try_into().unwrap();
                let remainder_result: u64 = t.remainder(k).try_into().unwrap();

                println!(
                    "Step: {} / {} = {} (ref {})",
                    dividend_as_int,
                    divisor_as_int,
                    result,
                    dividend_as_int / divisor_as_int
                );
                assert_eq!(dividend_as_int / divisor_as_int, result);
                assert_eq!(dividend_as_int % divisor_as_int, remainder_result);
            }
        }
    }
}
