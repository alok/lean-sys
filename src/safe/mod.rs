//! Safe, high-level wrappers for Lean objects.
//!
//! This module provides memory-safe Rust types that automatically manage
//! Lean's reference counting. Instead of manually calling `lean_inc` and
//! `lean_dec`, these wrappers use Rust's RAII (Drop trait) for automatic
//! cleanup.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use lean_sys::safe::*;
//!
//! // Initialize Lean runtime (once per process)
//! unsafe { lean_sys::lean_initialize_runtime_module(); }
//!
//! // Create values - reference counting is automatic
//! let s = LeanString::from("Hello!");
//! let arr = LeanArray::new().push(LeanObject::from_usize(42));
//! let n = LeanNat::from(100usize);
//!
//! // Clone increments reference count
//! let s2 = s.clone();
//!
//! // Drop decrements (and frees when count hits 0)
//! drop(s);  // s2 still holds a reference
//! ```
//!
//! # Types
//!
//! | Type | Description | Lean Equivalent |
//! |------|-------------|-----------------|
//! | [`LeanObject`] | Base object wrapper | Any Lean value |
//! | [`LeanString`] | UTF-8 string | `String` |
//! | [`LeanArray<T>`] | Dynamic array | `Array α` |
//! | [`LeanNat`] | Natural number | `Nat` |
//! | [`External<T>`] | Wrapped Rust value | `@[extern]` types |
//!
//! # Reference Counting
//!
//! Lean uses reference counting with three states:
//!
//! - **Single-threaded** (`rc > 0`): Fast, non-atomic operations
//! - **Multi-threaded** (`rc < 0`): Thread-safe atomic operations
//! - **Persistent** (`rc = 0`): Compile-time constants, never freed
//!
//! The safe wrappers handle this automatically:
//! - [`Clone`] calls `lean_inc` (increments reference count)
//! - [`Drop`] calls `lean_dec` (decrements, may deallocate)
//!
//! # Thread Safety
//!
//! All safe wrapper types are [`Send`] but not [`Sync`]. This means:
//! - ✓ You can move objects between threads
//! - ✗ You cannot share references across threads without synchronization
//!
//! # In-Place Mutation
//!
//! Lean supports in-place mutation when a value is exclusively owned
//! (reference count == 1). Use [`is_exclusive()`](LeanObject::is_exclusive)
//! to check before mutation operations.
//!
//! # Feature Flags
//!
//! - `safe` (default): Enables this module
//! - `std`: Enables additional `std`-dependent functionality

mod object;
mod string;
mod array;
mod nat;
mod external;

pub use object::LeanObject;
pub use string::LeanString;
pub use array::{LeanArray, LeanArrayIter};
pub use nat::LeanNat;
pub use external::{ExternalClass, External};

/// Trait for types that can be converted to/from Lean objects.
///
/// This trait provides a uniform interface for type conversions
/// at the FFI boundary.
pub trait LeanType: Sized {
    /// Convert from a raw Lean object pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid Lean object of the appropriate type.
    unsafe fn from_raw(ptr: *mut crate::lean_object) -> Self;

    /// Convert to a raw Lean object pointer, transferring ownership.
    fn into_raw(self) -> *mut crate::lean_object;
}

impl LeanType for LeanObject {
    unsafe fn from_raw(ptr: *mut crate::lean_object) -> Self {
        LeanObject::from_raw(ptr)
    }

    fn into_raw(self) -> *mut crate::lean_object {
        LeanObject::into_raw(self)
    }
}

impl LeanType for LeanString {
    unsafe fn from_raw(ptr: *mut crate::lean_object) -> Self {
        LeanString::from_raw(ptr)
    }

    fn into_raw(self) -> *mut crate::lean_object {
        LeanString::into_raw(self)
    }
}

impl LeanType for LeanNat {
    unsafe fn from_raw(ptr: *mut crate::lean_object) -> Self {
        LeanNat::from_raw(ptr)
    }

    fn into_raw(self) -> *mut crate::lean_object {
        LeanNat::into_raw(self)
    }
}

impl<T: 'static> LeanType for External<T> {
    unsafe fn from_raw(ptr: *mut crate::lean_object) -> Self {
        External::from_raw(ptr)
    }

    fn into_raw(self) -> *mut crate::lean_object {
        External::into_raw(self)
    }
}

/// Initialize the Lean runtime.
///
/// This must be called before using any Lean objects. It is safe
/// to call multiple times (subsequent calls are no-ops).
///
/// For multi-threaded applications, also call [`init_task_manager()`]
/// after this.
///
/// # Example
///
/// ```rust,ignore
/// lean_sys::safe::init();
/// lean_sys::safe::init_task_manager(); // If using Task/async
/// ```
pub fn init() {
    use crate::LEAN_INIT_MUTEX;
    use core::sync::atomic::{AtomicBool, Ordering};

    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if !INITIALIZED.swap(true, Ordering::SeqCst) {
        let _guard = LEAN_INIT_MUTEX.lock();
        unsafe {
            crate::lean_initialize_runtime_module();
        }
    }
}

/// Initialize the Lean task manager for async operations.
///
/// Call this after [`init()`] if you plan to use `Task` or
/// async Lean operations.
pub fn init_task_manager() {
    use core::sync::atomic::{AtomicBool, Ordering};

    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if !INITIALIZED.swap(true, Ordering::SeqCst) {
        unsafe {
            crate::lean_init_task_manager();
        }
    }
}

/// Initialize the Lean task manager with a specific number of worker threads.
pub fn init_task_manager_with_workers(num_workers: u32) {
    use core::sync::atomic::{AtomicBool, Ordering};

    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if !INITIALIZED.swap(true, Ordering::SeqCst) {
        unsafe {
            crate::lean_init_task_manager_using(num_workers);
        }
    }
}

/// Mark the end of initialization phase.
///
/// Call this after all initialization is complete to enable
/// certain runtime optimizations.
pub fn mark_end_initialization() {
    unsafe {
        crate::lean_io_mark_end_initialization();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        init();
    }

    #[test]
    fn test_init_is_idempotent() {
        init();
        init();
        init();
        // Should not panic or cause issues
    }

    #[test]
    fn test_object_scalar() {
        setup();
        let obj = LeanObject::from_usize(42);
        assert!(obj.is_scalar());
        assert_eq!(obj.to_usize(), Some(42));
    }

    #[test]
    fn test_string_basic() {
        setup();
        let s = LeanString::from("hello");
        assert_eq!(s.as_str(), Some("hello"));
        assert_eq!(s.byte_len(), 5);
    }

    #[test]
    fn test_array_basic() {
        setup();
        let arr: LeanArray = LeanArray::new();
        assert!(arr.is_empty());

        let arr = arr.push(LeanObject::from_usize(1));
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn test_nat_basic() {
        setup();
        let n = LeanNat::from(42usize);
        assert!(n.is_small());
        assert_eq!(n.to_usize(), Some(42));
    }

    #[test]
    fn test_nat_arithmetic() {
        setup();
        let a = LeanNat::from(10usize);
        let b = LeanNat::from(5usize);

        let sum = a.clone() + b.clone();
        assert_eq!(sum.to_usize(), Some(15));

        let product = a * b;
        assert_eq!(product.to_usize(), Some(50));
    }
}
