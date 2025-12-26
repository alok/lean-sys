# lean-sys Roadmap

This roadmap outlines the evolution of lean-sys from low-level FFI bindings to a comprehensive, safe, and ergonomic Rust interface for Lean 4.

## Vision

Transform lean-sys from a manual `lean.h` translation into a production-grade Rust ecosystem for Lean 4 interop:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Rust Application                            │
├─────────────────────────────────────────────────────────────────┤
│  lean-sys-derive     │  lean-sys-macros    │  High-level APIs   │
│  #[derive(LeanType)] │  #[lean_export]     │  LeanString, etc.  │
├─────────────────────────────────────────────────────────────────┤
│                     lean-sys (safe layer)                       │
│  LeanObject, LeanArray<T>, LeanNat, External<T>, etc.          │
├─────────────────────────────────────────────────────────────────┤
│                     lean-sys (raw FFI)                          │
│  lean_object, lean_inc, lean_dec, lean_box, lean_unbox, etc.   │
├─────────────────────────────────────────────────────────────────┤
│                     Lean Runtime (libleanshared)                │
│  Reference counting, allocation, task scheduler, IO            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation (API Completeness)

**Goal**: Ensure 100% coverage of `lean.h` APIs and fix any gaps.

### 1.1 Missing Thread/Initialization APIs

Current gap: Thread lifecycle management is incomplete.

```rust
// src/thread.rs additions
extern "C" {
    /// Initialize Lean state for current thread (call at thread start)
    pub fn lean_initialize_thread();

    /// Finalize Lean state for current thread (call before thread exit)
    pub fn lean_finalize_thread();

    /// Mark end of initialization phase (enables certain optimizations)
    pub fn lean_io_mark_end_initialization();

    /// Setup command-line arguments for Lean runtime
    pub fn lean_setup_args(argc: c_int, argv: *mut *mut c_char) -> *mut *mut c_char;
}
```

### 1.2 Missing IO Result Helpers

```rust
// src/io.rs additions
#[inline]
pub unsafe fn lean_io_result_is_ok(r: *const lean_object) -> bool {
    lean_obj_tag(r) == 0
}

#[inline]
pub unsafe fn lean_io_result_is_error(r: *const lean_object) -> bool {
    lean_obj_tag(r) == 1
}

#[inline]
pub unsafe fn lean_io_result_get_value(r: b_lean_obj_arg) -> b_lean_obj_res {
    lean_ctor_get(r, 0)
}

#[inline]
pub unsafe fn lean_io_result_get_error(r: b_lean_obj_arg) -> b_lean_obj_res {
    lean_ctor_get(r, 0)
}

extern "C" {
    pub fn lean_io_result_show_error(r: b_lean_obj_arg);
}
```

### 1.3 Audit Against Latest lean.h

Compare current bindings against:
- `~/lean4/stage0/src/include/lean/lean.h` (reference)
- `$(lean --print-prefix)/include/lean/lean.h` (installed version)

Document any new functions in Lean 4.20+ that need binding.

### 1.4 Deliverables

- [ ] Thread initialization functions
- [ ] IO result helpers (is_ok, is_error, get_value, get_error, show_error)
- [ ] lean_setup_args for CLI integration
- [ ] Audit document listing any remaining gaps
- [ ] Update to track latest stable Lean version

---

## Phase 2: Safe Wrapper Types

**Goal**: Provide memory-safe Rust types that automate reference counting.

### 2.1 Core LeanObject Wrapper

```rust
// src/safe/object.rs
use core::ptr::NonNull;
use core::marker::PhantomData;

/// A reference-counted Lean object with automatic memory management.
///
/// # Reference Counting
///
/// - `Clone` calls `lean_inc` (increments reference count)
/// - `Drop` calls `lean_dec` (decrements, may deallocate)
///
/// # Thread Safety
///
/// `LeanObject` is `Send` but not `Sync`. Lean objects can be moved between
/// threads, but concurrent access requires explicit synchronization.
#[repr(transparent)]
pub struct LeanObject {
    ptr: NonNull<lean_object>,
}

impl LeanObject {
    /// Create from raw pointer, taking ownership.
    ///
    /// # Safety
    /// - `ptr` must be a valid, non-null Lean object pointer
    /// - Caller transfers ownership (must not call lean_dec on ptr)
    #[inline]
    pub unsafe fn from_raw(ptr: *mut lean_object) -> Self {
        debug_assert!(!ptr.is_null());
        Self { ptr: NonNull::new_unchecked(ptr) }
    }

    /// Create from raw pointer if non-null.
    #[inline]
    pub unsafe fn from_raw_nullable(ptr: *mut lean_object) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Get raw pointer (borrowed).
    #[inline]
    pub fn as_ptr(&self) -> *mut lean_object {
        self.ptr.as_ptr()
    }

    /// Consume and return raw pointer (transfers ownership to caller).
    #[inline]
    pub fn into_raw(self) -> *mut lean_object {
        let ptr = self.ptr.as_ptr();
        core::mem::forget(self);
        ptr
    }

    /// Check if this is a scalar (tagged immediate value).
    #[inline]
    pub fn is_scalar(&self) -> bool {
        lean_is_scalar(self.ptr.as_ptr())
    }

    /// Check if this is the sole reference (useful for in-place mutation).
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        unsafe { lean_is_exclusive(self.ptr.as_ptr()) }
    }

    /// Check if this object is shared (reference count > 1).
    #[inline]
    pub fn is_shared(&self) -> bool {
        unsafe { lean_is_shared(self.ptr.as_ptr()) }
    }

    /// Get the object tag (type discriminator).
    #[inline]
    pub fn tag(&self) -> u32 {
        unsafe { lean_obj_tag(self.ptr.as_ptr()) }
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
```

### 2.2 Type-Specific Wrappers

```rust
// src/safe/string.rs
/// A Lean String with UTF-8 content.
#[repr(transparent)]
pub struct LeanString(LeanObject);

impl LeanString {
    /// Create from Rust string slice.
    pub fn from_str(s: &str) -> Self {
        unsafe {
            let ptr = lean_mk_string_from_bytes(s.as_ptr(), s.len());
            Self(LeanObject::from_raw(ptr))
        }
    }

    /// Get as C string pointer (valid for lifetime of self).
    pub fn as_cstr(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(lean_string_cstr(self.0.as_ptr()))
        }
    }

    /// Get UTF-8 byte length.
    pub fn byte_len(&self) -> usize {
        unsafe { lean_string_size(self.0.as_ptr()) - 1 } // Exclude null terminator
    }

    /// Get character (codepoint) count.
    pub fn char_len(&self) -> usize {
        unsafe { lean_string_len(self.0.as_ptr()) }
    }

    /// Convert to Rust String.
    pub fn to_string(&self) -> String {
        self.as_cstr().to_string_lossy().into_owned()
    }
}

impl From<&str> for LeanString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl AsRef<LeanObject> for LeanString {
    fn as_ref(&self) -> &LeanObject {
        &self.0
    }
}
```

```rust
// src/safe/array.rs
/// A Lean Array containing elements of type T.
pub struct LeanArray<T> {
    obj: LeanObject,
    _marker: PhantomData<T>,
}

impl<T: LeanType> LeanArray<T> {
    /// Create an empty array.
    pub fn new() -> Self {
        unsafe {
            Self {
                obj: LeanObject::from_raw(lean_mk_empty_array()),
                _marker: PhantomData,
            }
        }
    }

    /// Get array length.
    pub fn len(&self) -> usize {
        unsafe { lean_array_size(self.obj.as_ptr()) }
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get element at index (panics if out of bounds).
    pub fn get(&self, index: usize) -> T::Borrowed<'_> {
        assert!(index < self.len(), "index out of bounds");
        unsafe {
            let ptr = lean_array_get_core(self.obj.as_ptr(), index);
            T::borrow_from_raw(ptr)
        }
    }

    /// Push element, returning new array.
    pub fn push(self, value: T) -> Self {
        unsafe {
            let new_ptr = lean_array_push(self.obj.into_raw(), value.into_raw());
            Self {
                obj: LeanObject::from_raw(new_ptr),
                _marker: PhantomData,
            }
        }
    }
}
```

```rust
// src/safe/nat.rs
/// A Lean natural number (arbitrary precision).
pub struct LeanNat(LeanObject);

impl LeanNat {
    /// Create from usize (small nat optimization).
    pub fn from_usize(n: usize) -> Self {
        unsafe {
            Self(LeanObject::from_raw(lean_usize_to_nat(n)))
        }
    }

    /// Try to convert to usize (returns None if too large).
    pub fn to_usize(&self) -> Option<usize> {
        if self.is_small() {
            Some(unsafe { lean_unbox(self.0.as_ptr()) })
        } else {
            None // Big nat - would need conversion
        }
    }

    /// Check if this is a small (unboxed) natural.
    pub fn is_small(&self) -> bool {
        self.0.is_scalar()
    }
}

impl From<usize> for LeanNat {
    fn from(n: usize) -> Self {
        Self::from_usize(n)
    }
}

// Arithmetic operations
impl core::ops::Add for LeanNat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        unsafe {
            Self(LeanObject::from_raw(
                lean_nat_add(self.0.into_raw(), rhs.0.into_raw())
            ))
        }
    }
}
```

### 2.3 External Object Helper

```rust
// src/safe/external.rs
use core::any::TypeId;

/// A registered external class for type T.
pub struct ExternalClass<T: 'static> {
    class: *mut lean_external_class,
    _marker: PhantomData<T>,
}

impl<T: 'static> ExternalClass<T> {
    /// Register a new external class for type T.
    ///
    /// This should be called once per type, typically at initialization.
    pub fn register() -> Self {
        unsafe extern "C" fn finalize<T>(data: *mut c_void) {
            drop(Box::from_raw(data as *mut T));
        }

        unsafe extern "C" fn foreach<T>(_data: *mut c_void, _f: b_lean_obj_arg) {
            // Default: no nested Lean objects to traverse
        }

        let class = unsafe {
            lean_register_external_class(Some(finalize::<T>), Some(foreach::<T>))
        };

        Self {
            class,
            _marker: PhantomData,
        }
    }

    /// Wrap a Rust value as a Lean external object.
    pub fn wrap(&self, value: T) -> External<T> {
        let boxed = Box::into_raw(Box::new(value));
        unsafe {
            External {
                obj: LeanObject::from_raw(lean_alloc_external(self.class, boxed as *mut c_void)),
                _marker: PhantomData,
            }
        }
    }
}

/// A Lean external object wrapping a Rust value of type T.
pub struct External<T: 'static> {
    obj: LeanObject,
    _marker: PhantomData<T>,
}

impl<T: 'static> External<T> {
    /// Get a reference to the wrapped value.
    pub fn get(&self) -> &T {
        unsafe {
            &*(lean_get_external_data(self.obj.as_ptr()) as *const T)
        }
    }

    /// Get a mutable reference if this is the sole reference.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.obj.is_exclusive() {
            Some(unsafe {
                &mut *(lean_get_external_data(self.obj.as_ptr()) as *mut T)
            })
        } else {
            None
        }
    }
}

impl<T: 'static> AsRef<LeanObject> for External<T> {
    fn as_ref(&self) -> &LeanObject {
        &self.obj
    }
}
```

### 2.4 Deliverables

- [ ] `LeanObject` - core wrapper with Clone/Drop
- [ ] `LeanString` - UTF-8 string wrapper
- [ ] `LeanArray<T>` - generic array wrapper
- [ ] `LeanNat`, `LeanInt` - numeric wrappers
- [ ] `ExternalClass<T>`, `External<T>` - external object helpers
- [ ] `LeanType` trait for type conversions
- [ ] Feature flag: `safe` (default on)

---

## Phase 3: Proc-Macro Crate

**Goal**: Eliminate boilerplate for Lean FFI.

### 3.1 lean-sys-macros Crate Structure

```
lean-sys-macros/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── lean_export.rs    # #[lean_export] attribute
│   ├── lean_extern.rs    # #[lean_extern] attribute
│   └── codegen.rs        # Shared code generation
```

### 3.2 #[lean_export] Macro

```rust
// Usage:
#[lean_export("my_add")]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

// Generates:
#[no_mangle]
pub unsafe extern "C" fn my_add(
    _a: lean_sys::lean_obj_arg,
    _b: lean_sys::lean_obj_arg,
) -> lean_sys::lean_obj_res {
    let a: u32 = lean_sys::lean_unbox_uint32(_a);
    lean_sys::lean_dec(_a);
    let b: u32 = lean_sys::lean_unbox_uint32(_b);
    lean_sys::lean_dec(_b);
    let result = add(a, b);
    lean_sys::lean_box_uint32(result)
}
```

### 3.3 #[lean_extern] Macro

```rust
// Usage:
#[lean_extern("lean_my_func")]
fn my_func(s: &str, n: usize) -> String;

// Generates extern declaration + safe wrapper:
extern "C" {
    fn lean_my_func(s: lean_obj_arg, n: usize) -> lean_obj_res;
}

fn my_func(s: &str, n: usize) -> String {
    unsafe {
        let s_lean = LeanString::from(s);
        let result = lean_my_func(s_lean.into_raw(), n);
        LeanString::from_raw(result).to_string()
    }
}
```

### 3.4 Type Mapping

| Rust Type | Lean Type | Boxing/Unboxing |
|-----------|-----------|-----------------|
| `u8`..`u64` | `UInt8`..`UInt64` | `lean_box/unbox_uintN` |
| `i8`..`i64` | `Int8`..`Int64` | `lean_box/unbox_intN` |
| `usize` | `USize` | `lean_box/unbox_usize` |
| `f64` | `Float` | `lean_box/unbox_float` |
| `bool` | `Bool` | `lean_box/unbox_uint8` |
| `String`, `&str` | `String` | `LeanString` conversion |
| `Vec<T>` | `Array T` | `LeanArray<T>` conversion |
| `()` | `Unit` | `lean_box(0)` |
| `Result<T, E>` | `Except E T` | Constructor 0/1 |

### 3.5 Deliverables

- [ ] `lean-sys-macros` crate with proc-macros
- [ ] `#[lean_export]` attribute macro
- [ ] `#[lean_extern]` attribute macro
- [ ] Type mapping documentation
- [ ] Compile-time validation of supported types

---

## Phase 4: Testing Infrastructure

**Goal**: Comprehensive testing against real Lean runtime.

### 4.1 Integration Test Framework

```rust
// tests/common/mod.rs
use std::sync::Once;
static INIT: Once = Once::new();

pub fn init_lean() {
    INIT.call_once(|| {
        unsafe {
            lean_sys::lean_initialize_runtime_module();
            lean_sys::lean_io_mark_end_initialization();
        }
    });
}

#[macro_export]
macro_rules! lean_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            $crate::common::init_lean();
            $body
        }
    };
}
```

### 4.2 Test Categories

```rust
// tests/primitives.rs
lean_test!(test_box_unbox_roundtrip, {
    for n in [0, 1, 42, usize::MAX >> 1] {
        assert_eq!(lean_unbox(lean_box(n)), n);
    }
});

lean_test!(test_string_roundtrip, {
    let cases = ["", "hello", "🦀 Rust + Lean 🔮", "null\0embedded"];
    for s in cases {
        let ls = LeanString::from(s);
        assert_eq!(ls.to_string(), s.replace('\0', ""));
    }
});

// tests/arrays.rs
lean_test!(test_array_push_pop, {
    let arr = LeanArray::<LeanNat>::new();
    assert!(arr.is_empty());

    let arr = arr.push(42.into());
    assert_eq!(arr.len(), 1);
});

// tests/refcount.rs
lean_test!(test_clone_increments_rc, {
    let obj = LeanString::from("test");
    assert!(obj.as_ref().is_exclusive());

    let obj2 = obj.clone();
    assert!(obj.as_ref().is_shared());
    assert!(obj2.as_ref().is_shared());

    drop(obj2);
    assert!(obj.as_ref().is_exclusive());
});

// tests/external.rs
lean_test!(test_external_object, {
    static CLASS: Lazy<ExternalClass<String>> = Lazy::new(|| {
        ExternalClass::register()
    });

    let ext = CLASS.wrap("hello".to_string());
    assert_eq!(ext.get(), "hello");
});
```

### 4.3 CI Matrix

```yaml
# .github/workflows/test.yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        lean: [v4.20.0, v4.23.0, v4.25.0, nightly]
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Lean
        uses: leanprover/lean4-action@v1
        with:
          version: ${{ matrix.lean }}

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          toolchain: ${{ matrix.rust }}

      - name: Build
        run: cargo build --all-features

      - name: Test
        run: cargo test --all-features

      - name: Miri (nightly only)
        if: matrix.rust == 'nightly'
        run: |
          rustup component add miri
          cargo +nightly miri test --lib
```

### 4.4 Deliverables

- [ ] Integration test framework with init helper
- [ ] Primitive type tests (boxing, strings, arrays, nats)
- [ ] Reference counting tests
- [ ] External object tests
- [ ] Multi-Lean-version CI matrix
- [ ] Miri testing for undefined behavior

---

## Phase 5: Documentation & Examples

### 5.1 Comprehensive API Docs

Every public item documented with:
- Purpose and usage
- Safety requirements for unsafe functions
- Examples
- Reference to corresponding Lean/C API

### 5.2 Guide: "Lean FFI from Rust"

```markdown
# Lean FFI from Rust

## Quick Start

```rust
use lean_sys::safe::*;

fn main() {
    // Initialize Lean runtime (once per process)
    lean_sys::init();

    // Create Lean values
    let s = LeanString::from("Hello from Rust!");
    let arr = LeanArray::new().push(42.into()).push(100.into());

    // Values are automatically reference-counted
    let s2 = s.clone();  // lean_inc
    drop(s);             // lean_dec (s2 still holds reference)
}
```

## Calling Lean from Rust

```rust
// In Lean:
// @[export rust_callable]
// def myLeanFunc (n : Nat) : String := ...

#[lean_extern("rust_callable")]
fn my_lean_func(n: usize) -> String;

fn main() {
    lean_sys::init();
    let result = my_lean_func(42);
    println!("Lean says: {}", result);
}
```

## Calling Rust from Lean

```rust
// In Rust:
#[lean_export("my_rust_func")]
fn my_rust_func(s: &str) -> usize {
    s.len()
}

// In Lean:
// @[extern "my_rust_func"]
// opaque myRustFunc : String → USize
```
```

### 5.3 Example Projects

```
examples/
├── hello_lean/          # Minimal example calling Lean from Rust
├── rust_plugin/         # Rust library callable from Lean
├── bidirectional/       # Two-way FFI
└── external_objects/    # Wrapping Rust types for Lean
```

### 5.4 Deliverables

- [ ] Complete rustdoc for all public APIs
- [ ] "Lean FFI from Rust" guide
- [ ] 4 example projects
- [ ] Troubleshooting guide (common errors)
- [ ] Architecture diagram in README

---

## Timeline

| Phase | Description | Effort |
|-------|-------------|--------|
| 1 | Foundation (API completeness) | 1 week |
| 2 | Safe wrapper types | 2 weeks |
| 3 | Proc-macro crate | 1 week |
| 4 | Testing infrastructure | 1 week |
| 5 | Documentation & examples | 1 week |

## Success Metrics

- [ ] 100% coverage of lean.h public API
- [ ] Zero unsafe blocks needed for basic usage
- [ ] < 5 lines of code to call Lean from Rust
- [ ] Works with Lean 4.20+
- [ ] All tests pass on Linux, macOS, Windows
- [ ] Documentation coverage > 90%

---

## Contributing

Contributions welcome! Priority areas:
1. Additional safe wrapper types
2. Platform-specific fixes
3. Documentation improvements
4. Example projects
