markdown_doctest::md_doctest!(
    "md_no_code_block.md",
    transforms = {
        *: {
            ^ => "first",
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
