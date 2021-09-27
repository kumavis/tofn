use libpaillier::unknown_order::BigNumber;

/// Check if `x` is a member of the multiplicative group `Z*_n`
pub(super) fn member_of_mul_group(x: &BigNumber, n: &BigNumber) -> bool {
    if x < &BigNumber::one() || x >= n {
        return false;
    }

    if !x.gcd(n).is_one() {
        return false;
    }

    true
}
