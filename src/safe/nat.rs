//! Safe wrapper for Lean natural numbers.
//!
//! [`LeanNat`] provides a safe interface to Lean's arbitrary-precision natural numbers.

use crate::*;
use super::object::LeanObject;

/// A Lean natural number (arbitrary precision).
///
/// Lean uses small nat optimization: values that fit in (usize::BITS - 1) bits
/// are stored as tagged pointers (scalars), while larger values use heap-allocated
/// big integers (GMP).
///
/// # Example
///
/// ```rust,ignore
/// use lean_sys::safe::LeanNat;
///
/// let n = LeanNat::from(42usize);
/// assert_eq!(n.to_usize(), Some(42));
///
/// let big = LeanNat::from(u64::MAX) + LeanNat::from(1usize);
/// assert!(big.to_usize().is_none()); // Too big for usize
/// ```
#[repr(transparent)]
pub struct LeanNat(LeanObject);

impl LeanNat {
    /// Create from a usize value.
    #[inline]
    pub fn from_usize(n: usize) -> Self {
        unsafe { Self(LeanObject::from_raw(lean_usize_to_nat(n))) }
    }

    /// Create from a u64 value.
    #[inline]
    pub fn from_u64(n: u64) -> Self {
        unsafe { Self(LeanObject::from_raw(lean_uint64_to_nat(n))) }
    }

    /// Create from a raw Lean object pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid Lean Nat object (scalar or big nat)
    /// - Caller transfers ownership
    #[inline]
    pub unsafe fn from_raw(ptr: *mut lean_object) -> Self {
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

    /// Get raw pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut lean_object {
        self.0.as_ptr()
    }

    /// Consume and return raw pointer.
    #[inline]
    pub fn into_raw(self) -> *mut lean_object {
        self.0.into_raw()
    }

    /// Check if this is a small (scalar) natural number.
    ///
    /// Small nats are stored as tagged pointers and don't require heap allocation.
    #[inline]
    pub fn is_small(&self) -> bool {
        self.0.is_scalar()
    }

    /// Try to convert to usize.
    ///
    /// Returns `None` if the value is too large to fit in a usize
    /// (i.e., if it's a big nat).
    #[inline]
    pub fn to_usize(&self) -> Option<usize> {
        if self.is_small() {
            Some(lean_unbox(self.0.as_ptr()))
        } else {
            // Big nat - cannot directly convert to usize
            // Would need to call lean_nat_to_uint64 and check if it fits
            None
        }
    }

    /// Create a LeanNat representing zero.
    #[inline]
    pub fn zero() -> Self {
        Self::from_usize(0)
    }

    /// Create a LeanNat representing one.
    #[inline]
    pub fn one() -> Self {
        Self::from_usize(1)
    }

    /// Check if this is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.is_scalar() && lean_unbox(self.0.as_ptr()) == 0
    }

    /// Successor (n + 1).
    #[inline]
    pub fn succ(self) -> Self {
        unsafe {
            let ptr = lean_nat_succ(self.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Predecessor (n - 1, saturating at 0).
    #[inline]
    pub fn pred(self) -> Self {
        unsafe {
            let ptr = lean_nat_sub(self.0.into_raw(), lean_box(1));
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Check if `self < other`.
    #[inline]
    pub fn lt(&self, other: &Self) -> bool {
        unsafe {
            lean_inc(self.0.as_ptr());
            lean_inc(other.0.as_ptr());
            lean_nat_dec_lt(self.0.as_ptr(), other.0.as_ptr()) != 0
        }
    }

    /// Check if `self <= other`.
    #[inline]
    pub fn le(&self, other: &Self) -> bool {
        unsafe {
            lean_inc(self.0.as_ptr());
            lean_inc(other.0.as_ptr());
            lean_nat_dec_le(self.0.as_ptr(), other.0.as_ptr()) != 0
        }
    }

    /// Check if `self == other` using Lean's nat equality.
    #[inline]
    pub fn nat_eq(&self, other: &Self) -> bool {
        unsafe {
            lean_inc(self.0.as_ptr());
            lean_inc(other.0.as_ptr());
            lean_nat_dec_eq(self.0.as_ptr(), other.0.as_ptr()) != 0
        }
    }

    /// Greatest common divisor.
    #[inline]
    pub fn gcd(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_gcd(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Integer division (self / other) using Lean's nat division.
    #[inline]
    pub fn nat_div(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_div(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Modulo (self % other).
    #[inline]
    pub fn modulo(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_mod(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Power (self ^ exp).
    #[inline]
    pub fn pow(self, exp: Self) -> Self {
        unsafe {
            let ptr = lean_nat_pow(self.0.into_raw(), exp.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Bitwise AND.
    #[inline]
    pub fn land(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_land(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Bitwise OR.
    #[inline]
    pub fn lor(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_lor(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Bitwise XOR.
    #[inline]
    pub fn lxor(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_lxor(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Left shift.
    #[inline]
    pub fn shiftl(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_shiftl(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Right shift.
    #[inline]
    pub fn shiftr(self, other: Self) -> Self {
        unsafe {
            let ptr = lean_nat_shiftr(self.0.into_raw(), other.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }
}

impl Clone for LeanNat {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for LeanNat {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl From<usize> for LeanNat {
    #[inline]
    fn from(n: usize) -> Self {
        Self::from_usize(n)
    }
}

impl From<u64> for LeanNat {
    #[inline]
    fn from(n: u64) -> Self {
        Self::from_u64(n)
    }
}

impl From<u32> for LeanNat {
    #[inline]
    fn from(n: u32) -> Self {
        Self::from_usize(n as usize)
    }
}

// Arithmetic operators

impl core::ops::Add for LeanNat {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        unsafe {
            let ptr = lean_nat_add(self.0.into_raw(), rhs.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }
}

impl core::ops::Sub for LeanNat {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        unsafe {
            let ptr = lean_nat_sub(self.0.into_raw(), rhs.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }
}

impl core::ops::Mul for LeanNat {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        unsafe {
            let ptr = lean_nat_mul(self.0.into_raw(), rhs.0.into_raw());
            Self(LeanObject::from_raw(ptr))
        }
    }
}

impl core::ops::Div for LeanNat {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        self.nat_div(rhs)
    }
}

impl core::ops::Rem for LeanNat {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self {
        self.modulo(rhs)
    }
}

impl PartialEq for LeanNat {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.nat_eq(other)
    }
}

impl Eq for LeanNat {}

impl PartialOrd for LeanNat {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LeanNat {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.lt(other) {
            core::cmp::Ordering::Less
        } else if self.nat_eq(other) {
            core::cmp::Ordering::Equal
        } else {
            core::cmp::Ordering::Greater
        }
    }
}

impl core::fmt::Debug for LeanNat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(n) = self.to_usize() {
            write!(f, "LeanNat({})", n)
        } else {
            write!(f, "LeanNat(<big>)")
        }
    }
}

impl core::fmt::Display for LeanNat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(n) = self.to_usize() {
            write!(f, "{}", n)
        } else {
            // For big nats, we'd need to convert to string via Lean
            write!(f, "<big nat>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lean_initialize_runtime_module_locked;

    #[test]
    fn test_small_nat() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let n = LeanNat::from(42usize);
        assert!(n.is_small());
        assert_eq!(n.to_usize(), Some(42));
    }

    #[test]
    fn test_zero_one() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let zero = LeanNat::zero();
        let one = LeanNat::one();
        assert!(zero.is_zero());
        assert!(!one.is_zero());
    }

    #[test]
    fn test_add() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let a = LeanNat::from(10usize);
        let b = LeanNat::from(32usize);
        let c = a + b;
        assert_eq!(c.to_usize(), Some(42));
    }

    #[test]
    fn test_mul() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let a = LeanNat::from(6usize);
        let b = LeanNat::from(7usize);
        let c = a * b;
        assert_eq!(c.to_usize(), Some(42));
    }

    #[test]
    fn test_comparison() {
        unsafe { lean_initialize_runtime_module_locked(); }
        let a = LeanNat::from(10usize);
        let b = LeanNat::from(20usize);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a.clone(), a);
    }
}
