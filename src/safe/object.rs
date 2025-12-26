//! Core safe wrapper for Lean objects with automatic reference counting.
//!
//! [`LeanObject`] provides a memory-safe interface to Lean's reference-counted objects.
//! Reference counts are automatically managed via [`Clone`] and [`Drop`].

use crate::*;
use core::ptr::NonNull;

/// A reference-counted Lean object with automatic memory management.
///
/// # Reference Counting
///
/// Lean uses reference counting for memory management with three states:
/// - **Single-threaded** (`m_rc > 0`): Fast, non-atomic reference counting
/// - **Multi-threaded** (`m_rc < 0`): Thread-safe atomic reference counting
/// - **Persistent** (`m_rc == 0`): Never freed (compile-time constants)
///
/// [`LeanObject`] automatically manages references:
/// - [`Clone`] increments the reference count via `lean_inc`
/// - [`Drop`] decrements the reference count via `lean_dec`
///
/// # Thread Safety
///
/// `LeanObject` is [`Send`] but not [`Sync`]. Lean objects can be moved between
/// threads, but concurrent access requires explicit synchronization.
///
/// # Example
///
/// ```rust,ignore
/// use lean_sys::safe::LeanObject;
///
/// // Objects are automatically reference-counted
/// let obj = LeanObject::from_raw(some_lean_ptr);
/// let obj2 = obj.clone();  // lean_inc called
/// drop(obj);               // lean_dec called (obj2 still holds reference)
/// ```
#[repr(transparent)]
pub struct LeanObject {
    ptr: NonNull<lean_object>,
}

impl LeanObject {
    /// Create from raw pointer, taking ownership.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid, non-null Lean object pointer
    /// - Caller transfers ownership (must not call `lean_dec` on ptr afterwards)
    #[inline]
    pub unsafe fn from_raw(ptr: *mut lean_object) -> Self {
        debug_assert!(!ptr.is_null(), "LeanObject::from_raw called with null pointer");
        Self {
            ptr: NonNull::new_unchecked(ptr),
        }
    }

    /// Create from raw pointer if non-null.
    ///
    /// # Safety
    ///
    /// If `ptr` is non-null, it must be a valid Lean object pointer.
    /// Caller transfers ownership if the pointer is non-null.
    #[inline]
    pub unsafe fn from_raw_nullable(ptr: *mut lean_object) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Get raw pointer (borrowed).
    ///
    /// The returned pointer is valid for the lifetime of `self`.
    /// Do not call `lean_dec` on it.
    #[inline]
    pub fn as_ptr(&self) -> *mut lean_object {
        self.ptr.as_ptr()
    }

    /// Consume and return raw pointer (transfers ownership to caller).
    ///
    /// After calling this, the caller is responsible for calling `lean_dec`
    /// when done with the object.
    #[inline]
    pub fn into_raw(self) -> *mut lean_object {
        let ptr = self.ptr.as_ptr();
        core::mem::forget(self);
        ptr
    }

    /// Check if this is a scalar (tagged immediate value).
    ///
    /// Scalars are small values encoded directly in the pointer
    /// (lowest bit is 1). They don't require heap allocation.
    #[inline]
    pub fn is_scalar(&self) -> bool {
        lean_is_scalar(self.ptr.as_ptr())
    }

    /// Check if this is the sole reference (useful for in-place mutation).
    ///
    /// Returns `true` if the reference count is exactly 1, meaning
    /// this is the only reference to the object.
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        if self.is_scalar() {
            true // Scalars are always "exclusive"
        } else {
            unsafe { lean_is_exclusive(self.ptr.as_ptr()) }
        }
    }

    /// Check if this object is shared (reference count > 1).
    ///
    /// When shared, the object cannot be mutated in-place.
    #[inline]
    pub fn is_shared(&self) -> bool {
        if self.is_scalar() {
            false // Scalars are never "shared"
        } else {
            unsafe { lean_is_shared(self.ptr.as_ptr()) }
        }
    }

    /// Check if this object is persistent (reference count == 0).
    ///
    /// Persistent objects are compile-time constants that are never freed.
    #[inline]
    pub fn is_persistent(&self) -> bool {
        if self.is_scalar() {
            false
        } else {
            unsafe { lean_is_persistent(self.ptr.as_ptr()) }
        }
    }

    /// Check if this is a single-threaded object.
    #[inline]
    pub fn is_single_threaded(&self) -> bool {
        if self.is_scalar() {
            true
        } else {
            unsafe { lean_is_st(self.ptr.as_ptr()) }
        }
    }

    /// Check if this is a multi-threaded object.
    #[inline]
    pub fn is_multi_threaded(&self) -> bool {
        if self.is_scalar() {
            false
        } else {
            unsafe { lean_is_mt(self.ptr.as_ptr()) }
        }
    }

    /// Get the object tag (type discriminator).
    ///
    /// For scalars, this is the unboxed value.
    /// For heap objects, this identifies the object type:
    /// - 0-243: Constructor objects (tag = constructor variant)
    /// - 244: Promise
    /// - 245: Closure
    /// - 246: Array
    /// - 247: StructArray
    /// - 248: ScalarArray
    /// - 249: String
    /// - 250: MPZ (big integer)
    /// - 251: Thunk
    /// - 252: Task
    /// - 253: Ref
    /// - 254: External
    /// - 255: Reserved
    #[inline]
    pub fn tag(&self) -> u32 {
        unsafe { lean_obj_tag(self.ptr.as_ptr()) }
    }

    /// Check if this is a constructor object.
    #[inline]
    pub fn is_ctor(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_ctor(self.ptr.as_ptr()) }
    }

    /// Check if this is a closure.
    #[inline]
    pub fn is_closure(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_closure(self.ptr.as_ptr()) }
    }

    /// Check if this is an array.
    #[inline]
    pub fn is_array(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_array(self.ptr.as_ptr()) }
    }

    /// Check if this is a scalar array.
    #[inline]
    pub fn is_sarray(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_sarray(self.ptr.as_ptr()) }
    }

    /// Check if this is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_string(self.ptr.as_ptr()) }
    }

    /// Check if this is a thunk.
    #[inline]
    pub fn is_thunk(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_thunk(self.ptr.as_ptr()) }
    }

    /// Check if this is a task.
    #[inline]
    pub fn is_task(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_task(self.ptr.as_ptr()) }
    }

    /// Check if this is an external object.
    #[inline]
    pub fn is_external(&self) -> bool {
        !self.is_scalar() && unsafe { lean_is_external(self.ptr.as_ptr()) }
    }

    /// Box a usize value into a LeanObject (scalar).
    ///
    /// The value must fit in (usize::BITS - 1) bits.
    #[inline]
    pub fn from_usize(n: usize) -> Self {
        unsafe { Self::from_raw(lean_box(n)) }
    }

    /// Unbox a scalar LeanObject to usize.
    ///
    /// Returns `None` if this is not a scalar.
    #[inline]
    pub fn to_usize(&self) -> Option<usize> {
        if self.is_scalar() {
            Some(lean_unbox(self.ptr.as_ptr()))
        } else {
            None
        }
    }
}

impl Clone for LeanObject {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { lean_inc(self.ptr.as_ptr()) };
        Self { ptr: self.ptr }
    }
}

impl Drop for LeanObject {
    #[inline]
    fn drop(&mut self) {
        unsafe { lean_dec(self.ptr.as_ptr()) };
    }
}

// Safe to move between threads (Lean handles MT reference counting)
unsafe impl Send for LeanObject {}

// Debug implementation
impl core::fmt::Debug for LeanObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_scalar() {
            write!(f, "LeanObject::Scalar({})", lean_unbox(self.ptr.as_ptr()))
        } else {
            write!(
                f,
                "LeanObject::Heap {{ tag: {}, exclusive: {} }}",
                self.tag(),
                self.is_exclusive()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_roundtrip() {
        for n in [0, 1, 42, 1000, usize::MAX >> 1] {
            let obj = LeanObject::from_usize(n);
            assert!(obj.is_scalar());
            assert_eq!(obj.to_usize(), Some(n));
        }
    }

    #[test]
    fn test_scalar_clone() {
        let obj = LeanObject::from_usize(42);
        let obj2 = obj.clone();
        assert_eq!(obj.to_usize(), obj2.to_usize());
    }
}
