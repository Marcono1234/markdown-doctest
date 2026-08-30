markdown_doctest::md_doctest!(
    "transform_no_code_with_name.md",
    transforms = {
        "does-not-exist": {
            ^ => "first",
        },
        // ensure that all errors are reported, not just the first
        "does-not-exist-either": {
            ^ => "first",
        }
    },
);

// Suppress error about missing `main` function
fn main() {}
