# lean-sys

[![crates.io](https://img.shields.io/crates/v/lean-sys)](https://crates.io/crates/lean-sys)
[![docs.rs](https://img.shields.io/docsrs/lean-sys)](https://docs.rs/crate/lean-sys/latest)
[![lean version](https://img.shields.io/badge/lean-4.23.0-lightgray.svg)](#lean-version-requirements)

Rust bindings to [Lean 4](https://github.com/leanprover/lean4)'s C API.

## Features

- **Raw FFI bindings** to all Lean 4 C API functions
- **Safe wrappers** with automatic reference counting (`LeanObject`, `LeanString`, `LeanArray`, `LeanNat`)
- **Proc-macros** for easier FFI code generation (optional `macros` feature)
- **No-std compatible** (uses `alloc` crate)

## Quick Start

```rust
use lean_sys::safe::*;

fn main() {
    // Initialize Lean runtime (required before any Lean operations)
    init();

    // Create and manipulate Lean strings
    let s = LeanString::from_rust_str("Hello from Rust!");
    println!("Lean string: {}", s);
    println!("Length: {} chars, {} bytes", s.char_len(), s.byte_len());

    // Create arrays
    let arr = LeanArray::new()
        .push(LeanObject::from_usize(10))
        .push(LeanObject::from_usize(20))
        .push(LeanObject::from_usize(30));

    // Iterate over arrays
    let sum: usize = arr.iter()
        .filter_map(|obj| obj.to_usize())
        .sum();
    println!("Sum: {}", sum);

    // Natural number arithmetic
    let a = LeanNat::from(100usize);
    let b = LeanNat::from(42usize);
    let result = a + b;
    println!("100 + 42 = {:?}", result.to_usize());
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Your Rust Code                        │
├─────────────────────────────────────────────────────────┤
│  lean_sys::safe                                          │
│  ├── LeanObject  (RAII wrapper with auto refcounting)    │
│  ├── LeanString  (UTF-8 strings)                         │
│  ├── LeanArray   (dynamic arrays)                        │
│  ├── LeanNat     (arbitrary-precision naturals)          │
│  └── External<T> (embed Rust types in Lean objects)      │
├─────────────────────────────────────────────────────────┤
│  lean_sys (raw FFI)                                      │
│  └── lean_object, lean_inc, lean_dec, lean_box, ...      │
├─────────────────────────────────────────────────────────┤
│  Lean 4 Runtime (libleanshared)                          │
└─────────────────────────────────────────────────────────┘
```

## Safe Wrappers

The `safe` module provides memory-safe wrappers that handle reference counting automatically:

### LeanObject

Core wrapper for any Lean object. Implements `Clone` (increments ref count) and `Drop` (decrements ref count).

```rust
use lean_sys::safe::LeanObject;

let obj = LeanObject::from_usize(42);
assert!(obj.is_scalar());
assert_eq!(obj.to_usize(), Some(42));

let obj2 = obj.clone();  // Both point to same object, refcount = 2
assert!(!obj.is_exclusive());
```

### LeanString

UTF-8 string wrapper with full Unicode support.

```rust
use lean_sys::safe::LeanString;

let s = LeanString::from_rust_str("Hello 🦀");
assert_eq!(s.char_len(), 7);  // Unicode codepoints
assert!(s.byte_len() > 7);    // UTF-8 bytes

let s2 = s.push('!');
let s3 = s2.append(&LeanString::from_rust_str(" World"));
```

### LeanArray

Generic dynamic arrays with push, pop, get, set, and swap operations.

```rust
use lean_sys::safe::{LeanArray, LeanObject};

let arr: LeanArray = LeanArray::new()
    .push(LeanObject::from_usize(1))
    .push(LeanObject::from_usize(2));

// Iterate
for elem in arr.iter() {
    println!("{:?}", elem.to_usize());
}

// Modify (returns new array, may reuse memory if exclusive)
let arr = arr.set(0, LeanObject::from_usize(100));
```

### LeanNat

Arbitrary-precision natural numbers with arithmetic operations.

```rust
use lean_sys::safe::LeanNat;

let a = LeanNat::from(2usize);
let b = LeanNat::from(100usize);
let result = a.pow(b);  // 2^100

// Small values fit in a pointer (no allocation)
let small = LeanNat::from(42usize);
assert!(small.is_small());
```

## Features

| Feature | Description |
|---------|-------------|
| `default` | Enables `mimalloc` and `static` linking |
| `mimalloc` | Use mimalloc allocator (recommended for performance) |
| `static` | Static linking to Lean runtime |
| `macros` | Enable proc-macros from `lean-sys-macros` |

## Lean Version Requirements

This crate requires Lean 4 to be installed. The `lean` command must be in your PATH.

Tested with Lean versions:
- v4.20.0+
- v4.23.0 (current CI target)

## Raw FFI

For advanced use cases, you can access the raw C API directly:

```rust
use lean_sys::*;

unsafe {
    lean_initialize_runtime_module();

    let n = lean_box(42);
    assert!(lean_is_scalar(n));
    assert_eq!(lean_unbox(n), 42);

    let s = lean_mk_string("hello");
    lean_inc(s);  // Manual reference counting
    lean_dec(s);
    lean_dec(s);
}
```

## Contributing

See [ROADMAP.md](ROADMAP.md) for the development plan and areas that need work.

## License

MIT OR Apache-2.0
