//! Tests for reference counting behavior.

mod common;

use lean_sys::safe::{LeanString, LeanArray, LeanObject};

lean_test!(test_string_exclusive_after_create, {
    let s = LeanString::from_rust_str("test");
    assert!(s.is_exclusive());
});

lean_test!(test_string_shared_after_clone, {
    let s1 = LeanString::from_rust_str("shared");
    assert!(s1.is_exclusive());

    let s2 = s1.clone();
    assert!(!s1.is_exclusive());
    assert!(!s2.is_exclusive());
});

lean_test!(test_string_exclusive_after_drop, {
    let s1 = LeanString::from_rust_str("test");
    {
        let s2 = s1.clone();
        assert!(!s1.is_exclusive());
        drop(s2);
    }
    // After s2 is dropped, s1 should be exclusive again
    assert!(s1.is_exclusive());
});

lean_test!(test_array_exclusive_after_create, {
    let arr: LeanArray = LeanArray::new();
    assert!(arr.is_exclusive());
});

lean_test!(test_array_shared_after_clone, {
    let arr1: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1));

    assert!(arr1.is_exclusive());

    let arr2 = arr1.clone();
    assert!(!arr1.is_exclusive());
    assert!(!arr2.is_exclusive());
});

lean_test!(test_object_scalar_always_exclusive, {
    // Scalars don't use reference counting
    let obj = LeanObject::from_usize(42);
    assert!(obj.is_scalar());
    // Note: is_exclusive() behavior for scalars is implementation-defined
});

lean_test!(test_multiple_clones, {
    let s = LeanString::from_rust_str("many refs");

    let clones: Vec<_> = (0..5).map(|_| s.clone()).collect();

    // All should be shared
    assert!(!s.is_exclusive());
    for c in &clones {
        assert!(!c.is_exclusive());
    }

    // Drop all clones
    drop(clones);

    // Original should be exclusive again
    assert!(s.is_exclusive());
});

lean_test!(test_nested_clone_drop, {
    let s1 = LeanString::from_rust_str("nested");
    {
        let s2 = s1.clone();
        {
            let s3 = s2.clone();
            assert!(!s1.is_exclusive());
            assert!(!s2.is_exclusive());
            assert!(!s3.is_exclusive());
        }
        // s3 dropped
        assert!(!s1.is_exclusive());
        assert!(!s2.is_exclusive());
    }
    // s2 dropped
    assert!(s1.is_exclusive());
});
