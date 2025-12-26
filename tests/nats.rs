//! Tests for Lean natural number operations.

mod common;

use lean_sys::safe::LeanNat;

lean_test!(test_nat_zero_one, {
    let zero = LeanNat::zero();
    let one = LeanNat::one();

    assert!(zero.is_zero());
    assert!(!one.is_zero());
    assert!(zero.is_small());
    assert!(one.is_small());
});

lean_test!(test_nat_from_usize, {
    let n = LeanNat::from(42usize);
    assert!(n.is_small());
    assert_eq!(n.to_usize(), Some(42));
});

lean_test!(test_nat_from_u64, {
    let n = LeanNat::from(100u64);
    assert_eq!(n.to_usize(), Some(100));
});

lean_test!(test_nat_add, {
    let a = LeanNat::from(10usize);
    let b = LeanNat::from(32usize);
    let c = a + b;
    assert_eq!(c.to_usize(), Some(42));
});

lean_test!(test_nat_sub, {
    let a = LeanNat::from(50usize);
    let b = LeanNat::from(8usize);
    let c = a - b;
    assert_eq!(c.to_usize(), Some(42));
});

lean_test!(test_nat_sub_saturating, {
    // Subtraction saturates at 0 for natural numbers
    let a = LeanNat::from(5usize);
    let b = LeanNat::from(10usize);
    let c = a - b;
    assert!(c.is_zero());
});

lean_test!(test_nat_mul, {
    let a = LeanNat::from(6usize);
    let b = LeanNat::from(7usize);
    let c = a * b;
    assert_eq!(c.to_usize(), Some(42));
});

lean_test!(test_nat_div, {
    let a = LeanNat::from(84usize);
    let b = LeanNat::from(2usize);
    let c = a / b;
    assert_eq!(c.to_usize(), Some(42));
});

lean_test!(test_nat_mod, {
    let a = LeanNat::from(47usize);
    let b = LeanNat::from(5usize);
    let c = a % b;
    assert_eq!(c.to_usize(), Some(2));
});

lean_test!(test_nat_pow, {
    let base = LeanNat::from(2usize);
    let exp = LeanNat::from(10usize);
    let result = base.pow(exp);
    assert_eq!(result.to_usize(), Some(1024));
});

lean_test!(test_nat_gcd, {
    let a = LeanNat::from(48usize);
    let b = LeanNat::from(18usize);
    let gcd = a.gcd(b);
    assert_eq!(gcd.to_usize(), Some(6));
});

lean_test!(test_nat_comparison_lt, {
    let a = LeanNat::from(10usize);
    let b = LeanNat::from(20usize);
    assert!(a < b);
    assert!(a.lt(&b));
});

lean_test!(test_nat_comparison_le, {
    let a = LeanNat::from(10usize);
    let b = LeanNat::from(10usize);
    assert!(a <= b);
    assert!(a.le(&b));
});

lean_test!(test_nat_comparison_eq, {
    let a = LeanNat::from(42usize);
    let b = LeanNat::from(42usize);
    assert!(a == b);
    assert!(a.nat_eq(&b));
});

lean_test!(test_nat_ordering, {
    let a = LeanNat::from(5usize);
    let b = LeanNat::from(10usize);
    let c = LeanNat::from(5usize);

    assert!(a < b);
    assert!(b > a);
    assert!(a == c);
});

lean_test!(test_nat_succ, {
    let n = LeanNat::from(41usize);
    let n = n.succ();
    assert_eq!(n.to_usize(), Some(42));
});

lean_test!(test_nat_pred, {
    let n = LeanNat::from(43usize);
    let n = n.pred();
    assert_eq!(n.to_usize(), Some(42));
});

lean_test!(test_nat_pred_zero, {
    // pred(0) = 0 for natural numbers
    let zero = LeanNat::zero();
    let n = zero.pred();
    assert!(n.is_zero());
});

lean_test!(test_nat_bitwise_and, {
    let a = LeanNat::from(0b1100usize);
    let b = LeanNat::from(0b1010usize);
    let c = a.land(b);
    assert_eq!(c.to_usize(), Some(0b1000));
});

lean_test!(test_nat_bitwise_or, {
    let a = LeanNat::from(0b1100usize);
    let b = LeanNat::from(0b1010usize);
    let c = a.lor(b);
    assert_eq!(c.to_usize(), Some(0b1110));
});

lean_test!(test_nat_bitwise_xor, {
    let a = LeanNat::from(0b1100usize);
    let b = LeanNat::from(0b1010usize);
    let c = a.lxor(b);
    assert_eq!(c.to_usize(), Some(0b0110));
});

lean_test!(test_nat_shift_left, {
    let n = LeanNat::from(1usize);
    let shift = LeanNat::from(4usize);
    let result = n.shiftl(shift);
    assert_eq!(result.to_usize(), Some(16));
});

lean_test!(test_nat_shift_right, {
    let n = LeanNat::from(16usize);
    let shift = LeanNat::from(2usize);
    let result = n.shiftr(shift);
    assert_eq!(result.to_usize(), Some(4));
});

lean_test!(test_nat_clone, {
    let a = LeanNat::from(42usize);
    let b = a.clone();
    assert_eq!(a.to_usize(), b.to_usize());
});
