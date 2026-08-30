markdown_doctest::md_doctest!(
    "md_attributes_before_names_comment.md",
    transforms = {
        *: {
            ^ => "first",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
