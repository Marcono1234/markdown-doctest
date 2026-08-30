// Note: Currently contains no test for being unable to read the Markdown file, e.g. when it
// does not exist, because the error message includes the IO error which is OS dependent

#[test]
fn macro_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro_errors/*.rs");
}
