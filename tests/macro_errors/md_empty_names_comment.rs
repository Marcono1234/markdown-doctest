markdown_doctest::md_doctest!(
    "md_empty_names_comment.md",
    transforms = {
        *: {
            ^ => "first",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
