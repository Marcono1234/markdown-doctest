markdown_doctest::md_doctest!(
    "md_empty_names_comment_name.md",
    transforms = {
        *: {
            ^ => "first",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
