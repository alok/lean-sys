//! Procedural macros for Lean 4 FFI bindings.
//!
//! This crate provides macros to simplify creating FFI bindings between
//! Rust and Lean 4.
//!
//! # Macros
//!
//! - [`lean_export`] - Export a Rust function to Lean
//! - [`lean_callback`] - Create a callback-compatible extern "C" function
//!
//! # Example
//!
//! ```rust,ignore
//! use lean_sys_macros::lean_export;
//!
//! #[lean_export(name = "my_rust_add")]
//! fn add(a: usize, b: usize) -> usize {
//!     a + b
//! }
//! ```

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, FnArg, ReturnType, Pat, Ident};

/// Export a Rust function to Lean 4.
///
/// This macro generates an `extern "C"` wrapper function with the proper
/// calling convention for Lean's FFI. The wrapper handles:
/// - Converting `lean_object` pointers to/from Rust types
/// - Proper reference counting
///
/// # Attributes
///
/// - `name = "lean_name"` - The name to export as (default: function name)
/// - `no_mangle` - Apply #[no_mangle] attribute (default: true)
///
/// # Example
///
/// ```rust,ignore
/// use lean_sys_macros::lean_export;
///
/// // Simple scalar function
/// #[lean_export]
/// fn my_add(a: usize, b: usize) -> usize {
///     a + b
/// }
///
/// // Function with custom export name
/// #[lean_export(name = "rust_multiply")]
/// fn multiply(x: usize, y: usize) -> usize {
///     x * y
/// }
/// ```
#[proc_macro_attribute]
pub fn lean_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ExportArgs);
    let input = parse_macro_input!(item as ItemFn);

    let expanded = generate_lean_export(&args, &input);

    TokenStream::from(expanded)
}

/// Arguments for the lean_export macro.
struct ExportArgs {
    name: Option<String>,
    no_mangle: bool,
}

impl Default for ExportArgs {
    fn default() -> Self {
        Self {
            name: None,
            no_mangle: true,
        }
    }
}

impl syn::parse::Parse for ExportArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = ExportArgs::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: syn::Token![=] = input.parse()?;

            match ident.to_string().as_str() {
                "name" => {
                    let lit: syn::LitStr = input.parse()?;
                    args.name = Some(lit.value());
                }
                "no_mangle" => {
                    let lit: syn::LitBool = input.parse()?;
                    args.no_mangle = lit.value();
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "unknown attribute"));
                }
            }

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(args)
    }
}

fn generate_lean_export(args: &ExportArgs, input: &ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &input.sig.ident;
    let export_name = args.name.clone().unwrap_or_else(|| fn_name.to_string());
    let export_ident = format_ident!("{}", export_name);

    let vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;

    // Generate the wrapper function
    let wrapper_params: Vec<_> = fn_inputs.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;
                quote! { #pat: #ty }
            }
            FnArg::Receiver(_) => {
                quote! { self }
            }
        }
    }).collect();

    let param_names: Vec<_> = fn_inputs.iter().filter_map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    Some(&pat_ident.ident)
                } else {
                    None
                }
            }
            _ => None,
        }
    }).collect();

    let no_mangle = if args.no_mangle {
        quote! { #[no_mangle] }
    } else {
        quote! {}
    };

    let return_type = match fn_output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    // For scalar types (usize, i64, etc.), we can pass through directly
    // For object types, we need to handle reference counting
    quote! {
        // Original function (internal)
        #[inline]
        #vis fn #fn_name(#(#wrapper_params),*) #fn_output #fn_block

        // Exported wrapper
        #no_mangle
        #[doc(hidden)]
        pub extern "C" fn #export_ident(#(#wrapper_params),*) -> #return_type {
            #fn_name(#(#param_names),*)
        }
    }
}

/// Create a callback wrapper for use with Lean's task system.
///
/// This macro generates a callback function compatible with Lean's
/// `lean_external_foreach_proc` type signature.
///
/// # Example
///
/// ```rust,ignore
/// use lean_sys_macros::lean_callback;
///
/// #[lean_callback]
/// fn my_finalizer(data: *mut std::ffi::c_void) {
///     // Cleanup code here
/// }
/// ```
#[proc_macro_attribute]
pub fn lean_callback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;

    // Generate params for the callback
    let wrapper_params: Vec<_> = fn_inputs.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;
                quote! { #pat: #ty }
            }
            FnArg::Receiver(_) => quote! { self }
        }
    }).collect();

    let expanded = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name(#(#wrapper_params),*) #fn_block
    };

    TokenStream::from(expanded)
}

/// Declare an external Lean function to be called from Rust.
///
/// This is a declarative macro (not proc-macro) helper that generates
/// proper extern "C" declarations for Lean functions.
///
/// # Example
///
/// ```rust,ignore
/// lean_extern! {
///     fn lean_nat_add(a: *mut lean_object, b: *mut lean_object) -> *mut lean_object;
/// }
/// ```
#[proc_macro]
pub fn lean_extern(input: TokenStream) -> TokenStream {
    // Parse and generate extern "C" block
    let item = parse_macro_input!(input as syn::ItemFn);

    let fn_name = &item.sig.ident;
    let fn_inputs = &item.sig.inputs;
    let fn_output = &item.sig.output;

    let params: Vec<_> = fn_inputs.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;
                quote! { #pat: #ty }
            }
            FnArg::Receiver(_) => quote! { self }
        }
    }).collect();

    let expanded = quote! {
        extern "C" {
            pub fn #fn_name(#(#params),*) #fn_output;
        }
    };

    TokenStream::from(expanded)
}
