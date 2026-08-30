//! Tests macro syntax and usage errors, independent of used Markdown file

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        // no transforms
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            ^ => "a",
        },
        "other": {
            ^ => "b",
        },
        // duplicate wildcard
        *: {
            ^ => "c",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        "a": {
            ^ => "a",
        },
        "b": {
            ^ => "b",
        },
        // duplicate name
        "a": {
            ^ => "c",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        "a": {
            // no transforms
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // search string containing \n
            "a\nb"| => "a",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // search string containing \r
            "a\rb"| => "a",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // both match and replace prefix
            (*<*"a">) => "b",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // both match and replace suffix
            (<"a"*>*) => "b",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // insert \n
            "a"| => "b\n",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // insert \r
            "a"| => "b\r",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // insert in-line \n
            ("a"|) => "b\n",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // insert in-line \r
            ("a"|) => "b\r",
        },
    },
);

markdown_doctest::md_doctest!(
    "dummy.md",
    transforms = {
        *: {
            // insert empty
            "a"| => [],
        },
    },
);

// Suppress error about missing `main` function
fn main() {}
