//! Common test infrastructure for lean-sys integration tests.

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the Lean runtime for tests.
///
/// This function is idempotent and thread-safe - it will only
/// initialize the runtime once, even if called from multiple tests.
pub fn init_lean() {
    INIT.call_once(|| {
        unsafe {
            lean_sys::lean_initialize_runtime_module_locked();
        }
    });
}

/// Macro for writing Lean integration tests with automatic initialization.
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
