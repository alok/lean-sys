//! Safe wrapper for Lean strings.
//!
//! [`LeanString`] provides a Rust-friendly interface to Lean's UTF-8 strings.

use crate::*;
use super::object::LeanObject;
use core::ffi::CStr;

/// A Lean String with UTF-8 content.
///
/// Lean strings are immutable, reference-counted, and UTF-8 encoded.
/// This wrapper provides safe access to string data and automatic
/// memory management through the underlying [`LeanObject`].
///
/// # Example
///
/// ```rust,ignore
/// use lean_sys::safe::LeanString;
///
/// let s = LeanString::from_rust_str("Hello, Lean!");
/// assert_eq!(s.to_string(), "Hello, Lean!");
/// assert_eq!(s.char_len(), 12);
/// ```
#[repr(transparent)]
pub struct LeanString(LeanObject);

impl LeanString {
    /// Create a new LeanString from a Rust string slice.
    ///
    /// The string is copied into Lean-managed memory.
    #[inline]
    pub fn from_rust_str(s: &str) -> Self {
        unsafe {
            let ptr = lean_mk_string_from_bytes(s.as_ptr(), s.len());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Create a new empty LeanString.
    #[inline]
    pub fn new() -> Self {
        Self::from_rust_str("")
    }

    /// Create from a raw Lean object pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid Lean string object
    /// - Caller transfers ownership
    #[inline]
    pub unsafe fn from_raw(ptr: *mut lean_object) -> Self {
        debug_assert!(lean_is_string(ptr), "LeanString::from_raw: not a string");
        Self(LeanObject::from_raw(ptr))
    }

    /// Get the underlying LeanObject.
    #[inline]
    pub fn as_object(&self) -> &LeanObject {
        &self.0
    }

    /// Consume and return the underlying LeanObject.
    #[inline]
    pub fn into_object(self) -> LeanObject {
        self.0
    }

    /// Get raw pointer to the Lean object.
    #[inline]
    pub fn as_ptr(&self) -> *mut lean_object {
        self.0.as_ptr()
    }

    /// Consume and return raw pointer.
    #[inline]
    pub fn into_raw(self) -> *mut lean_object {
        self.0.into_raw()
    }

    /// Get as a C string pointer.
    ///
    /// The returned pointer is valid for the lifetime of `self`.
    /// The string is null-terminated.
    #[inline]
    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(lean_string_cstr(self.0.as_ptr()) as *const core::ffi::c_char) }
    }

    /// Get as a byte slice.
    ///
    /// Does not include the null terminator.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_cstr().to_bytes()
    }

    /// Get as a str slice.
    ///
    /// Returns `None` if the string is not valid UTF-8.
    /// (Lean strings should always be valid UTF-8, but this provides safety.)
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        self.as_cstr().to_str().ok()
    }

    /// Get the byte length (not including null terminator).
    #[inline]
    pub fn byte_len(&self) -> usize {
        unsafe { lean_string_size(self.0.as_ptr()).saturating_sub(1) }
    }

    /// Get the character (UTF-8 codepoint) count.
    #[inline]
    pub fn char_len(&self) -> usize {
        unsafe { lean_string_len(self.0.as_ptr()) }
    }

    /// Check if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.byte_len() == 0
    }

    /// Check if this is the sole reference to the string.
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        self.0.is_exclusive()
    }

    /// Append another string, returning a new string.
    ///
    /// If this string is exclusively owned, the append may happen in-place.
    #[inline]
    pub fn append(self, other: &LeanString) -> Self {
        unsafe {
            lean_inc(other.0.as_ptr());
            let ptr = lean_string_append(self.0.into_raw(), other.0.as_ptr());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Push a character to the string.
    ///
    /// If this string is exclusively owned, the push may happen in-place.
    #[inline]
    pub fn push(self, c: char) -> Self {
        unsafe {
            let ptr = lean_string_push(self.0.into_raw(), c as u32);
            Self(LeanObject::from_raw(ptr))
        }
    }
}

impl Default for LeanString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for LeanString {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<&str> for LeanString {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_rust_str(s)
    }
}

impl From<&alloc_crate::string::String> for LeanString {
    #[inline]
    fn from(s: &alloc_crate::string::String) -> Self {
        Self::from_rust_str(s.as_str())
    }
}

impl From<LeanString> for alloc_crate::string::String {
    fn from(s: LeanString) -> Self {
        alloc_crate::string::String::from(s.as_str().unwrap_or(""))
    }
}

impl core::fmt::Debug for LeanString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "LeanString({:?})", s),
            None => write!(f, "LeanString(<invalid utf8>)"),
        }
    }
}

impl core::fmt::Display for LeanString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Some(s) => f.write_str(s),
            None => f.write_str("<invalid utf8>"),
        }
    }
}

impl PartialEq for LeanString {
    fn eq(&self, other: &Self) -> bool {
        unsafe { lean_string_eq(self.0.as_ptr(), other.0.as_ptr()) }
    }
}

impl Eq for LeanString {}

impl PartialOrd for LeanString {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LeanString {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Compare using byte representation
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl core::hash::Hash for LeanString {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

// Need alloc for String
extern crate alloc as alloc_crate;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lean_initialize_runtime_module_locked;

    #[test]
    fn test_empty_string() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let s = LeanString::new();
        assert!(s.is_empty());
        assert_eq!(s.byte_len(), 0);
        assert_eq!(s.char_len(), 0);
    }

    #[test]
    fn test_from_rust_str() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let s = LeanString::from_rust_str("hello");
        assert_eq!(s.byte_len(), 5);
        assert_eq!(s.char_len(), 5);
        assert_eq!(s.as_str(), Some("hello"));
    }

    #[test]
    fn test_unicode() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let s = LeanString::from_rust_str("hello 🦀");
        assert_eq!(s.char_len(), 7); // 6 ASCII + 1 emoji
        assert!(s.byte_len() > 7); // Emoji is multi-byte
    }
}
