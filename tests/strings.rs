//! Tests for Lean string operations.

mod common;

use lean_sys::safe::LeanString;

lean_test!(test_empty_string, {
    let s = LeanString::new();
    assert!(s.is_empty());
    assert_eq!(s.byte_len(), 0);
    assert_eq!(s.char_len(), 0);
    assert_eq!(s.as_str(), Some(""));
});

lean_test!(test_ascii_string, {
    let s = LeanString::from_rust_str("hello");
    assert_eq!(s.byte_len(), 5);
    assert_eq!(s.char_len(), 5);
    assert_eq!(s.as_str(), Some("hello"));
    assert!(!s.is_empty());
});

lean_test!(test_unicode_string, {
    let s = LeanString::from_rust_str("hello 🦀");
    assert_eq!(s.char_len(), 7); // 6 ASCII + 1 emoji
    assert!(s.byte_len() > 7); // Emoji is multi-byte UTF-8
    assert_eq!(s.as_str(), Some("hello 🦀"));
});

lean_test!(test_string_from_trait, {
    let s: LeanString = "test".into();
    assert_eq!(s.as_str(), Some("test"));
});

lean_test!(test_string_clone, {
    let s1 = LeanString::from_rust_str("shared");
    assert!(s1.is_exclusive());

    let s2 = s1.clone();
    // After clone, both are shared
    assert!(!s1.is_exclusive());
    assert!(!s2.is_exclusive());

    // Content is the same
    assert_eq!(s1.as_str(), s2.as_str());
});

lean_test!(test_string_push, {
    let s = LeanString::from_rust_str("abc");
    let s = s.push('d');
    assert_eq!(s.as_str(), Some("abcd"));
    assert_eq!(s.char_len(), 4);
});

lean_test!(test_string_append, {
    let s1 = LeanString::from_rust_str("hello ");
    let s2 = LeanString::from_rust_str("world");
    let result = s1.append(&s2);
    assert_eq!(result.as_str(), Some("hello world"));
});

lean_test!(test_string_equality, {
    let s1 = LeanString::from_rust_str("test");
    let s2 = LeanString::from_rust_str("test");
    let s3 = LeanString::from_rust_str("other");

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
});

lean_test!(test_string_ordering, {
    let a = LeanString::from_rust_str("apple");
    let b = LeanString::from_rust_str("banana");

    assert!(a < b);
    assert!(b > a);
});

lean_test!(test_special_characters, {
    let s = LeanString::from_rust_str("line1\nline2\ttab");
    assert_eq!(s.as_str(), Some("line1\nline2\ttab"));
    assert_eq!(s.char_len(), 15);
});
