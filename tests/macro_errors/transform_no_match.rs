markdown_doctest::md_doctest!(
    "transform_no_match.md",
    transforms = {
        *: {
            "does-not-exist"| => "insert",
            // ensure that all errors are reported, not just the first
            "does-not-exist-either"| => "insert",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
