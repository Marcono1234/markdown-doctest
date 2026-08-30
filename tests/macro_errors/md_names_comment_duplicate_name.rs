markdown_doctest::md_doctest!(
    "md_names_comment_duplicate_name.md",
    transforms = {
        *: {
            ^ => "first",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
