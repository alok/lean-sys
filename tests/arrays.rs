//! Tests for Lean array operations.

mod common;

use lean_sys::safe::{LeanArray, LeanObject};

lean_test!(test_empty_array, {
    let arr: LeanArray = LeanArray::new();
    assert!(arr.is_empty());
    assert_eq!(arr.len(), 0);
});

lean_test!(test_array_with_capacity, {
    let arr: LeanArray = LeanArray::with_capacity(100);
    assert!(arr.is_empty());
    assert!(arr.capacity() >= 100);
});

lean_test!(test_push_single, {
    let arr: LeanArray = LeanArray::new();
    let arr = arr.push(LeanObject::from_usize(42));
    assert_eq!(arr.len(), 1);
    assert_eq!(arr.get(0).to_usize(), Some(42));
});

lean_test!(test_push_multiple, {
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1))
        .push(LeanObject::from_usize(2))
        .push(LeanObject::from_usize(3));

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get(0).to_usize(), Some(1));
    assert_eq!(arr.get(1).to_usize(), Some(2));
    assert_eq!(arr.get(2).to_usize(), Some(3));
});

lean_test!(test_array_set, {
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1))
        .push(LeanObject::from_usize(2));

    let arr = arr.set(0, LeanObject::from_usize(10));
    assert_eq!(arr.get(0).to_usize(), Some(10));
    assert_eq!(arr.get(1).to_usize(), Some(2));
});

lean_test!(test_array_pop, {
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1))
        .push(LeanObject::from_usize(2));

    let (arr, elem) = arr.pop().unwrap();
    assert_eq!(elem.to_usize(), Some(2));
    assert_eq!(arr.len(), 1);
});

lean_test!(test_array_pop_empty, {
    let arr: LeanArray = LeanArray::new();
    assert!(arr.pop().is_none());
});

lean_test!(test_array_swap, {
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1))
        .push(LeanObject::from_usize(2))
        .push(LeanObject::from_usize(3));

    let arr = arr.swap(0, 2);
    assert_eq!(arr.get(0).to_usize(), Some(3));
    assert_eq!(arr.get(1).to_usize(), Some(2));
    assert_eq!(arr.get(2).to_usize(), Some(1));
});

lean_test!(test_array_iter, {
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(10))
        .push(LeanObject::from_usize(20))
        .push(LeanObject::from_usize(30));

    let sum: usize = arr.iter().filter_map(|o| o.to_usize()).sum();
    assert_eq!(sum, 60);
});

lean_test!(test_array_iter_exact_size, {
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1))
        .push(LeanObject::from_usize(2));

    let iter = arr.iter();
    assert_eq!(iter.len(), 2);
});

lean_test!(test_array_clone, {
    let arr1: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(42));

    assert!(arr1.is_exclusive());

    let arr2 = arr1.clone();
    // After clone, both point to the same data
    assert!(!arr1.is_exclusive());
    assert!(!arr2.is_exclusive());

    // Content is the same
    assert_eq!(arr1.get(0).to_usize(), arr2.get(0).to_usize());
});

lean_test!(test_array_from_iter, {
    let objects: Vec<LeanObject> = (0..5).map(LeanObject::from_usize).collect();
    let arr: LeanArray = objects.into_iter().collect();

    assert_eq!(arr.len(), 5);
    for i in 0..5 {
        assert_eq!(arr.get(i).to_usize(), Some(i));
    }
});

lean_test!(test_array_exclusive_modification, {
    // When exclusively owned, modifications can happen in place
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(1));

    assert!(arr.is_exclusive());
    let arr = arr.push(LeanObject::from_usize(2));
    // Still exclusive (may have been modified in place)
    assert!(arr.is_exclusive());
});
