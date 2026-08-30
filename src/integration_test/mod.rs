//! Hacky workaround to perform a full integration test on the macro:
//! - doctests don't run in `tests/`
//! - normally Rust disallows calling the macro from the main code
//!
//! However, by guarding this only with `#[cfg(doctest)]` (and not `any(test, doctest)`)
//! it seems the compiler permits this and the doctests are actually executed by `cargo test`.
//!
//! IMPORTANT:
//! - Use `#[cfg(doctest)]` for all code within here to not actually include irrelevant
//!   code for regular builds.
//! - When editing this file (especially when line numbers change) it might be necessary
//!   to update the checks for the `cargo test` output.

// TODO: The `cargo test` output is currently not checked
//   Could maybe use cargo-make script support (see https://github.com/sagiegurari/cargo-make/tree/0.37.24#script)
//   but it seems that installs an unpinned (?) version of rust-script, which is not ideal (and has multiple transitive dependencies)

#[cfg(doctest)]
markdown_doctest::md_doctest!(
    "./TestFile.md",
    transforms = {
        "ok-return": {
            // return Ok to allow using `?`
            $ => "Ok::<(), Box<dyn std::error::Error>>(())",
        },
    },
);
