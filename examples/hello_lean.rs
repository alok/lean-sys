//! A simple example demonstrating basic lean-sys usage.
//!
//! Run with: cargo run --example hello_lean --no-default-features

use lean_sys::safe::*;

fn main() {
    // Initialize the Lean runtime
    init();
    println!("Lean runtime initialized!");

    // === String Demo ===
    println!("\n=== String Demo ===");
    let greeting = LeanString::from_rust_str("Hello from Rust! 🦀");
    println!("Created: {}", greeting);
    println!("  Char length: {}", greeting.char_len());
    println!("  Byte length: {}", greeting.byte_len());
    println!("  Is exclusive: {}", greeting.is_exclusive());

    // Clone shares the underlying data
    let greeting2 = greeting.clone();
    println!("After clone:");
    println!("  Original exclusive: {}", greeting.is_exclusive());
    println!("  Clone exclusive: {}", greeting2.is_exclusive());

    // === Array Demo ===
    println!("\n=== Array Demo ===");
    let arr: LeanArray = LeanArray::new()
        .push(LeanObject::from_usize(10))
        .push(LeanObject::from_usize(20))
        .push(LeanObject::from_usize(30));

    println!("Array length: {}", arr.len());
    println!("Array capacity: {}", arr.capacity());

    print!("Elements: ");
    for (i, elem) in arr.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", elem.to_usize().unwrap_or(0));
    }
    println!();

    let sum: usize = arr.iter().filter_map(|o| o.to_usize()).sum();
    println!("Sum: {}", sum);

    // === Natural Number Demo ===
    println!("\n=== Natural Number Demo ===");
    let a = LeanNat::from(100usize);
    let b = LeanNat::from(42usize);

    println!("a = {:?}", a.to_usize());
    println!("b = {:?}", b.to_usize());

    let sum = a.clone() + b.clone();
    println!("a + b = {:?}", sum.to_usize());

    let product = a.clone() * b.clone();
    println!("a * b = {:?}", product.to_usize());

    let diff = a.clone() - b.clone();
    println!("a - b = {:?}", diff.to_usize());

    let quotient = a.clone() / b.clone();
    println!("a / b = {:?}", quotient.to_usize());

    let remainder = a % b;
    println!("a % b = {:?}", remainder.to_usize());

    // Power
    let base = LeanNat::from(2usize);
    let exp = LeanNat::from(10usize);
    let power = base.pow(exp);
    println!("2^10 = {:?}", power.to_usize());

    // === Scalar Demo ===
    println!("\n=== Scalar Demo ===");
    let scalar = LeanObject::from_usize(42);
    println!("Is scalar: {}", scalar.is_scalar());
    println!("Value: {:?}", scalar.to_usize());

    println!("\nDone! All Lean objects automatically cleaned up.");
}
