markdown_doctest::md_doctest!(
    "transform_no_match_ignore.md",
    transforms = {
        *: {
            ^ => "insert",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
