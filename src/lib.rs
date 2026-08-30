#![warn(missing_docs)]
// Enable 'unused' warnings for doctests (are disabled by default)
#![doc(test(attr(warn(unused))))]
// Fail on warnings in doctests
#![doc(test(attr(deny(warnings))))]

//! Library for including and transforming Rust code blocks of Markdown files as doctests
//!
//! See the [`md_doctest!`] macro.

use std::fs;

use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse_macro_input;

mod parse_macro;
use parse_macro::ParsedMdDoctestConfig;

mod parse_md;
use parse_md::parse_md;

mod transform;
use transform::transform;

/// Macro for including and transforming Rust code blocks of Markdown files as doctests
///
/// This macro is intended to be called from `src/lib.rs`. Usage of the macro should be guarded with
/// `#[cfg(doctest)]` to only execute it when running doctests, and not when creating a release build.
///
/// Example:
/// ```
/// #[cfg(doctest)]
/// markdown_doctest::md_doctest!(
///     "../README.md",
///     transforms = {
///         *: {
///             // insert the import as first line
///             ^ => "use std::fs;",
///             // replace the file path
///             (*<"my-file.txt">*) => "test-resource.txt",
///             // return Ok to allow using `?`
///             $ => "Ok::<(), Box<dyn std::error::Error>>(())",
///         },
///     }
/// );
/// ```
///
/// When running `cargo test` the code blocks from the Markdown file will then be executed as doctests.
///
/// See the [Usage guide](https://github.com/Marcono1234/markdown-doctest/blob/main/Usage.md) for details
/// and examples.
/*
 * Use concise but self-describing name `md_doctest`, so that it is not excessively long when using
 * full qualified name, but also works when used with an import (in a `#[cfg(doctest)] mod`)
 */
#[proc_macro]
pub fn md_doctest(input: TokenStream) -> TokenStream {
    let macro_config = parse_macro_input!(input as ParsedMdDoctestConfig);

    // Name prefix of the generated struct
    let struct_name_prefix = "MarkdownDoctest_Line_";

    let call_site = Span::call_site();
    let Some(mut file_path) = call_site.local_file() else {
        // Caller file path is apparently not available for rust-analyzer, see https://github.com/rust-lang/rust-analyzer/issues/15950
        // Cannot do anything then because Markdown file path has to be resolved against caller file path

        /*
         * This might lead to recompilation between rust-analyzer and rustc due to mismatching output?
         *
         * But this cannot be easily solved?
         * - using a path relative to the project root (env `CARGO_MANIFEST_DIR`) might work,
         *   but would be inconsistent with built-in macros such as `include_str!`, and would then
         *   also require calling the `include_bytes!` hack below with an absolute path (in case that
         *   is even possible)
         * - the code below also uses `call_site.line()`, and rust-analyzer seems to use hardcoded 1
         *   as result there, so that would lead to different results as well
         */

        // TODO(rust): Emit compiler warning here? once supported, see https://github.com/rust-lang/rust/issues/54140

        // Emit a dummy struct with always failing doctest, so in case this ends up being executed
        // as doctest it fails instead of silently passing without executing any tests
        /*
         * rust-analyzer then offers a "Run doctest" button in the UI
         *
         * This is probably acceptable / desired here:
         * - rust-analyzer runs the test with `cargo test --doc ...`, and therefore runs the real macro result
         *   doctest instead of this always failing one
         * - rust-analyzer does not pass `--exact` when running the test (https://github.com/rust-lang/rust-analyzer/issues/20643);
         *   so even though `call_site.line()` is unavailable and the full name cannot be constructed,
         *   using just the prefix allows it to actually run the doctests (this is quite hacky though)
         */
        let struct_name = format_ident!("{struct_name_prefix}");
        return quote! {
            /// ```rust
            /// panic!("IDE does not support `Span::local_file()`; this is a dummy placeholder");
            /// ```
            struct #struct_name;
        }
        .into();
    };

    let file_path_lit = macro_config.file_path;
    let file_path_span = file_path_lit.span();
    let file_path_str = file_path_lit.value();

    // Go to parent directory
    if !file_path.pop() {
        // This should normally not occur, it looks like even for a relative file path "file.rs"
        // `pop()` would succeed, and the parent will be an empty relative path (as desired)
        panic!("local file path is empty");
    }
    file_path.push(file_path_str.clone());
    let md_content = match fs::read_to_string(&file_path) {
        Ok(s) => s,
        Err(e) => {
            return syn::Error::new(
                file_path_span,
                format!("failed to read file {}: {e}", file_path.to_string_lossy()),
            )
            .to_compile_error()
            .into();
        }
    };

    let md_parsed = match parse_md(&md_content) {
        Ok(v) => v,
        Err(e) => {
            return syn::Error::new(file_path_span, format!("parsing Markdown failed: {e}"))
                .to_compile_error()
                .into();
        }
    };
    if md_parsed.is_empty() {
        return syn::Error::new(
            file_path_span,
            "did not find any ```rust code blocks in file",
        )
        .to_compile_error()
        .into();
    }

    let transformed = match transform(file_path_span, &md_parsed, &macro_config.transforms) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    if macro_config.debug {
        let mut debug_path = file_path.clone();
        // For simplicity just append file extension to existing one (if any); don't remove
        // or transform existing one (e.g. removing it and adding it back at the end) to not
        // make any wrong assumptions which might not always be true
        debug_path.add_extension("md_doctest_debug.md");
        fs::write(debug_path, &transformed).expect("failed writing debug file");
    }

    // TODO(rust): Ideally would attach Span pointing to Markdown file to transformed LitStr,
    // but currently not possible, see https://github.com/rust-lang/rfcs/issues/2869
    // similar to how standard `#[doc = include_str!("../README.md")]` reports the README as source when running doctests
    // but should then report line number 1 (as dummy), because lines will be off due to transformed source
    // -> when creating debug file, could report that as Span instead; line numbers might be correct then
    // -> or always report debug file as Span (even when not creating it), because reporting wrong line numbers in original
    //    Markdown file would be confusing, especially if IDE tries to highlight errors there then

    // TODO(rust): Hacky workaround for not being able to add custom Span:
    //   Insert dummy `#[doc = "\n\n..."]` which shifts line numbers reported by doctests, and makes it a bit easier
    //   to derive the line numbers from the doctest output;
    //   `+ n` at the end is to make sure opening ```rust is really reported at line 1,
    //   not completely sure why it is needed, maybe compiler somehow subtracts from line number based on number
    //   of generated lines?
    let offset = 1000 - call_site.line() % 1000 + 3;
    let prefix = if offset == 0 {
        proc_macro2::TokenStream::new()
    } else {
        let doc_str = "\n".repeat(offset);
        quote! {
            #[doc = #doc_str]
        }
    };

    // Make struct names unique within file
    // Including the line number also makes it easier to associate doctest console output with
    // macro call, because line numbers from doctest correspond to transformed Markdown content
    let line_number = call_site.line();
    let struct_name = format_ident!("{struct_name_prefix}{line_number}");
    quote! {
        #prefix
        #[doc = #transformed]
        struct #struct_name;
        // Make sure macro is re-run when file changes
        // TODO(rust): This is a hack, see https://github.com/rust-lang/rust/issues/99515
        //   The `include_bytes!` is in the generated code; there is no guarantee that the compiler then
        //   re-runs the macro, instead of just the generated code
        const _: &[u8] = include_bytes!(#file_path_str);
    }
    .into()
}

mod integration_test;
