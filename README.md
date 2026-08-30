# markdown-doctest

Rust library for including and transforming Rust code blocks of Markdown files as doctests.

## Why?

Currently Rust's support for running doctests in Markdown files is limited (see [cargo issue #383](https://github.com/rust-lang/cargo/issues/383)).
While it is possible to achieve this by using `#[doc = include_str!("../README.md")]` (see also [documentation](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#include-items-only-when-collecting-doctests)), this has the following problems:

- hiding lines with the doctest prefix `#` won't hide them from the rendered Markdown content
- specifying custom [doctest attributes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#attributes) might confuse the Markdown renderer

This means the code blocks in the Markdown files need redundant boilerplate code, otherwise they do not compile and are not testable.

> [!NOTE]
> If the code snippets in your Markdown files are actually compilable and runnable as is, then simply use the `#[doc = include_str!("...")]`
> approach instead of this library.
>
> Also have a look at the ["Similar projects" section](#similar-projects) below for crates which serve a similar purpose and
> might be a better alternative to this library.

> [!NOTE]
> This library is still experimental. Feedback and suggestions for improvements are welcome!

## How does it work?

1. The user calls the macro of this crate in their code
2. The macro does the following:
    1. Read the Markdown file
    2. Extract ```` ```rust ```` code blocks
    3. Transform code blocks according to macro configuration
    4. Emit a `struct` which has the transformed Markdown code as doc comment
3. The user runs `cargo test`, which includes the generated doctests

**Q:** If the macro extracts and transforms the code anyway, why not emit the code as regular unit test?\
**A:** Because [doctest attributes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#attributes) support functionality which
regular unit tests don't support, and because parsing and emitting the code (without risking hygiene issues?) might be more difficult. Have a look at crates
like [skeptic](https://crates.io/crates/skeptic) for this functionality.

## Usage

> [!NOTE]
> This crate is currently not published on crates.io; however because it is only needed as dev-dependency you
> can declare it as Git dependency:
>
> ```toml
> [dev-dependencies]
> markdown_doctest = { git = "https://github.com/Marcono1234/markdown-doctest.git", rev = "<git-commit>" }
> ```

Consider this example README content:

````markdown
How to read a file:

```rust
let content = fs::read_to_string("my-file.txt")?;
println!("content: {s}");
```
````

To run it as part of the doctests, you can use markdown-doctest in your `src/lib.rs` like this:

```rust
#[cfg(doctest)]
markdown_doctest::md_doctest!(
    "../README.md",
    transforms = {
        *: {
            // insert the import as first line
            ^ => "use std::fs;",
            // replace the file path
            (*<"my-file.txt">*) => "test-resource.txt",
            // return Ok to allow using `?`
            $ => "Ok::<(), Box<dyn std::error::Error>>(())",
        },
    }
);
```

See the [Usage guide](./Usage.md) for more details.

## Building

This project uses [cargo-make](https://github.com/sagiegurari/cargo-make) for building:

```sh
cargo make
```

## Similar projects

- <https://crates.io/crates/skeptic> (feature-rich, but requires using a build script)
  > Test your Rust Markdown via Cargo
- <https://crates.io/crates/doc-comment> (no support for transforming tests)
  > Write doc comments from macros
- <https://crates.io/crates/doubter> (slightly outdated)
  > Test Rust code blocks in your Markdown files
- <https://crates.io/crates/pretty-readme> (limited support for transforming tests)
  > Macro to make using a README.md file as the root module documentation easy, seamless, and testable
- <https://crates.io/crates/mce>
  > Rust macros to extract part(s) of README.md (or a similar file) and to use them in tests/doctests/elsewhere
- <https://crates.io/crates/include-file>
  > Include sections of files into Rust source code
- ... multiple crates which take the opposite approach: generate the README with examples from the code

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

All contributions you make to this project are licensed implicitly under both licenses mentioned above, without any additional terms or conditions.

Note: This dual-licensing is the same you see for the majority of Rust projects, see also the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/necessities.html#crate-and-its-dependencies-have-a-permissive-license-c-permissive).
