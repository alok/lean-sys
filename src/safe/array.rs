//! Safe wrapper for Lean arrays.
//!
//! [`LeanArray`] provides a type-safe interface to Lean's dynamic arrays.

use crate::*;
use super::object::LeanObject;
use core::marker::PhantomData;

/// A Lean Array containing elements.
///
/// Lean arrays are reference-counted and support efficient in-place
/// updates when exclusively owned (reference count == 1).
///
/// # Type Parameter
///
/// The type parameter `T` is a marker for the element type. Currently,
/// all elements are stored as `LeanObject`, but this provides type
/// documentation and future type-safety potential.
///
/// # Example
///
/// ```rust,ignore
/// use lean_sys::safe::{LeanArray, LeanObject};
///
/// let arr = LeanArray::<LeanObject>::new();
/// let arr = arr.push(LeanObject::from_usize(42));
/// assert_eq!(arr.len(), 1);
/// ```
pub struct LeanArray<T = LeanObject> {
    obj: LeanObject,
    _marker: PhantomData<T>,
}

impl<T> LeanArray<T> {
    /// Create an empty array.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            Self {
                obj: LeanObject::from_raw(lean_mk_empty_array()),
                _marker: PhantomData,
            }
        }
    }

    /// Create an array with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        unsafe {
            Self {
                obj: LeanObject::from_raw(lean_mk_empty_array_with_capacity(lean_box(capacity))),
                _marker: PhantomData,
            }
        }
    }

    /// Create from a raw Lean object pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid Lean array object
    /// - Caller transfers ownership
    #[inline]
    pub unsafe fn from_raw(ptr: *mut lean_object) -> Self {
        debug_assert!(lean_is_array(ptr), "LeanArray::from_raw: not an array");
        Self {
            obj: LeanObject::from_raw(ptr),
            _marker: PhantomData,
        }
    }

    /// Get the underlying LeanObject.
    #[inline]
    pub fn as_object(&self) -> &LeanObject {
        &self.obj
    }

    /// Consume and return the underlying LeanObject.
    #[inline]
    pub fn into_object(self) -> LeanObject {
        self.obj
    }

    /// Get raw pointer to the Lean object.
    #[inline]
    pub fn as_ptr(&self) -> *mut lean_object {
        self.obj.as_ptr()
    }

    /// Consume and return raw pointer.
    #[inline]
    pub fn into_raw(self) -> *mut lean_object {
        self.obj.into_raw()
    }

    /// Get array length.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { lean_array_size(self.obj.as_ptr()) }
    }

    /// Get array capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { lean_array_capacity(self.obj.as_ptr()) }
    }

    /// Check if the array is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this is the sole reference to the array.
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        self.obj.is_exclusive()
    }

    /// Get element at index (borrowed).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    #[inline]
    pub fn get(&self, index: usize) -> LeanObject {
        assert!(index < self.len(), "LeanArray::get: index out of bounds");
        unsafe {
            let ptr = lean_array_get_core(self.obj.as_ptr(), index);
            lean_inc(ptr);
            LeanObject::from_raw(ptr)
        }
    }

    /// Get element at index without bounds checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure `index < len()`.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> LeanObject {
        let ptr = lean_array_get_core(self.obj.as_ptr(), index);
        lean_inc(ptr);
        LeanObject::from_raw(ptr)
    }

    /// Push an element, returning a new array.
    ///
    /// If exclusively owned, may modify in-place.
    #[inline]
    pub fn push(self, value: LeanObject) -> Self {
        unsafe {
            let new_ptr = lean_array_push(self.obj.into_raw(), value.into_raw());
            Self {
                obj: LeanObject::from_raw(new_ptr),
                _marker: PhantomData,
            }
        }
    }

    /// Set element at index, returning a new array.
    ///
    /// If exclusively owned, may modify in-place.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    #[inline]
    pub fn set(self, index: usize, value: LeanObject) -> Self {
        assert!(index < self.len(), "LeanArray::set: index out of bounds");
        unsafe {
            let new_ptr = lean_array_fset(self.obj.into_raw(), lean_box(index), value.into_raw());
            Self {
                obj: LeanObject::from_raw(new_ptr),
                _marker: PhantomData,
            }
        }
    }

    /// Pop the last element, returning the modified array and the element.
    ///
    /// Returns `None` if the array is empty.
    #[inline]
    pub fn pop(self) -> Option<(Self, LeanObject)> {
        if self.is_empty() {
            return None;
        }
        let last_idx = self.len() - 1;
        let elem = self.get(last_idx);
        unsafe {
            let new_ptr = lean_array_pop(self.obj.into_raw());
            Some((
                Self {
                    obj: LeanObject::from_raw(new_ptr),
                    _marker: PhantomData,
                },
                elem,
            ))
        }
    }

    /// Swap two elements.
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    #[inline]
    pub fn swap(self, i: usize, j: usize) -> Self {
        let len = self.len();
        assert!(i < len && j < len, "LeanArray::swap: index out of bounds");
        unsafe {
            let new_ptr = lean_array_fswap(self.obj.into_raw(), lean_box(i), lean_box(j));
            Self {
                obj: LeanObject::from_raw(new_ptr),
                _marker: PhantomData,
            }
        }
    }

    /// Create an iterator over the array elements.
    #[inline]
    pub fn iter(&self) -> LeanArrayIter<'_, T> {
        LeanArrayIter {
            array: self,
            index: 0,
        }
    }
}

impl<T> Default for LeanArray<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for LeanArray<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            obj: self.obj.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> core::fmt::Debug for LeanArray<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "LeanArray {{ len: {}, capacity: {}, exclusive: {} }}",
            self.len(),
            self.capacity(),
            self.is_exclusive()
        )
    }
}

/// Iterator over LeanArray elements.
pub struct LeanArrayIter<'a, T> {
    array: &'a LeanArray<T>,
    index: usize,
}

impl<'a, T> Iterator for LeanArrayIter<'a, T> {
    type Item = LeanObject;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.len() {
            let elem = self.array.get(self.index);
            self.index += 1;
            Some(elem)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.array.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for LeanArrayIter<'a, T> {}

// Collecting LeanObjects into a LeanArray
impl<T> FromIterator<LeanObject> for LeanArray<T> {
    fn from_iter<I: IntoIterator<Item = LeanObject>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut arr = Self::with_capacity(lower);
        for item in iter {
            arr = arr.push(item);
        }
        arr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc as alloc_crate;
    use alloc_crate::vec;
    use alloc_crate::vec::Vec;
    use crate::lean_initialize_runtime_module_locked;

    #[test]
    fn test_empty_array() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let arr: LeanArray = LeanArray::new();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_push_and_get() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let arr: LeanArray = LeanArray::new();
        let arr = arr.push(LeanObject::from_usize(42));
        assert_eq!(arr.len(), 1);
        assert_eq!(arr.get(0).to_usize(), Some(42));
    }

    #[test]
    fn test_iter() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let arr: LeanArray = LeanArray::new()
            .push(LeanObject::from_usize(1))
            .push(LeanObject::from_usize(2))
            .push(LeanObject::from_usize(3));

        let values: Vec<_> = arr.iter().filter_map(|o| o.to_usize()).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }
}
