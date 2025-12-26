//! Tests for primitive boxing/unboxing operations.

mod common;

use lean_sys::{lean_box, lean_unbox};

lean_test!(test_box_unbox_zero, {
    let boxed = lean_box(0);
    assert_eq!(lean_unbox(boxed), 0);
});

lean_test!(test_box_unbox_one, {
    let boxed = lean_box(1);
    assert_eq!(lean_unbox(boxed), 1);
});

lean_test!(test_box_unbox_42, {
    let boxed = lean_box(42);
    assert_eq!(lean_unbox(boxed), 42);
});

lean_test!(test_box_unbox_max_scalar, {
    // Maximum value that fits in a scalar (all bits except the tag bit)
    let max = usize::MAX >> 1;
    let boxed = lean_box(max);
    assert_eq!(lean_unbox(boxed), max);
});

lean_test!(test_lean_is_scalar, {
    // Scalars have the lowest bit set
    assert!(lean_sys::lean_is_scalar(lean_box(0)));
    assert!(lean_sys::lean_is_scalar(lean_box(1)));
    assert!(lean_sys::lean_is_scalar(lean_box(42)));
    assert!(lean_sys::lean_is_scalar(lean_box(usize::MAX >> 1)));
});

lean_test!(test_multiple_box_unbox_roundtrips, {
    for n in [0, 1, 2, 10, 100, 1000, 10000, usize::MAX >> 1] {
        assert_eq!(lean_unbox(lean_box(n)), n, "failed for n = {}", n);
    }
});
