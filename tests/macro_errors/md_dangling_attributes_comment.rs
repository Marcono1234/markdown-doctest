markdown_doctest::md_doctest!(
    "md_dangling_attributes_comment.md",
    transforms = {
        *: {
            ^ => "first",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
