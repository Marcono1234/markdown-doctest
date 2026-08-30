use std::collections::HashSet;
use std::fs;
use std::io::Result;
use std::path::PathBuf;

/// Looks for `.expected` files and compares them with the actual debug output of the macro,
/// checking the transformed Markdown content.
#[test]
fn transformed_md() -> Result<()> {
    let mut source_files = Vec::new();
    let source_file_ext = ".md";

    let mut expected_files = Vec::new();
    let expected_file_ext = ".expected";

    let mut actual_files = Vec::new();
    let actual_file_ext = ".md_doctest_debug.md";

    // Get path of test file, see https://stackoverflow.com/a/30004252
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.push("tests/transformed_md");

    for file in fs::read_dir(test_dir)? {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }

        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap();
        let file_path = file.path();

        if file_name.ends_with(expected_file_ext) {
            expected_files.push(file_path);
        } else if file_name.ends_with(actual_file_ext) {
            actual_files.push(file_path);
        }
        // Check this last because file extension `.md` would also match 'expected' files
        else if file_name.ends_with(source_file_ext) {
            source_files.push(file_path);
        } else if file_name != ".gitignore" {
            panic!("unrecognized file type: {file_name}")
        }
    }

    if expected_files.is_empty() {
        panic!("no expected files found");
    }

    for source_file in source_files {
        let file_name = source_file.file_name().unwrap().to_str().unwrap();

        let mut expected_file = source_file.clone();
        if !expected_file.pop() {
            panic!("file has no parent dir");
        }
        expected_file.push(file_name.to_owned() + expected_file_ext);

        if !expected_files.contains(&expected_file) {
            panic!("missing expected file: {expected_file:?}");
        }
    }

    // Sort to get consistent order; directory iteration order is OS dependent
    expected_files.sort();

    let mut leftover_actual_files = HashSet::<PathBuf>::from_iter(actual_files);

    for expected_file in expected_files {
        let base_file_name = expected_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(expected_file_ext)
            .unwrap();

        let mut actual_file = expected_file.clone();
        if !actual_file.pop() {
            panic!("file has no parent dir");
        }
        actual_file.push(base_file_name.to_owned() + actual_file_ext);

        if !leftover_actual_files.remove(&actual_file) {
            panic!("missing actual file: {actual_file:?}");
        }

        let expected = fs::read_to_string(&expected_file)?.replace("\r\n", "\n");
        let actual = fs::read_to_string(&actual_file)?;
        assert_eq!(
            expected,
            actual,
            "unexpected content for: {:?}",
            actual_file.file_name().unwrap()
        )
    }

    if !leftover_actual_files.is_empty() {
        panic!("missing expected files for actual files: {leftover_actual_files:?}");
    }

    Ok(())
}

// Macro calls which create the debug files

markdown_doctest::md_doctest!(
    "./transformed_md/Markdown.md",
    transforms = {
        *: {
            ^ => "dummy",
        },
    },
    debug,
);

markdown_doctest::md_doctest!(
    "./transformed_md/Transforms.md",
    transforms = {
        "insert-start": {
            ^ => "start",
            ^ => ["1-1", "1-2"],
        },
        "insert-end": {
            $ => "end",
            $ => ["1-1", "1-2"],
        },
        "insert-line-before": {
            |"first" => ["1-1", "1-2"],
            |"second" => "2",
            |*"ird" => "3",
            |"four"* => "4",
            |*"if"* => "5",
        },
        "insert-line-after": {
            "first"| => ["1-1", "1-2"],
            "second"| => "2",
            *"ird"| => "3",
            "four"*| => "4",
            *"if"*| => "5",
        },
        "insert-inside-line-before": {
            (|"first") => "1",
            (*|"cond") => "2",
            (|"thir"*) => "3",
            (*|"our"*) => "4",
            (*|"multiple"*) => "5",
        },
        "insert-inside-line-after": {
            ("first"|) => "1",
            (*"cond"|) => "2",
            ("thir"|*) => "3",
            (*"our"|*) => "4",
            (*"multiple"|*) => "5",
        },
        "replace-line": {
            <"first"> => ["1-1", "1-2"],
            <*"cond"> => "2",
            <"thir"*> => "3",
            <*"our"*> => "4",
            // remove line
            <"fifth"> => [],
        },
        "replace-inside-line": {
            (<"first">) => "1",
            (*<"cond">) => "2",
            (<"thir">*) => "3",
            (*<"our">*) => "4",
            (*<"multiple">*) => "5",
            (<*"ixth">) => "6",
            (<"seven"*>) => "7",
            (<*"igh"*>) => "8",
            (<*"int">*) => "9",
            (*<"enth val"*>) => "10",
        },
        "transform-sequence": {
            ^ => "1",
            ^ => "2",
            $ => "3",
            $ => "4",
            "4"| => "5",
            "5"| => "6",
            <"6"> => "7",
            <"7"> => "8",
        },
    },
    debug,
);

markdown_doctest::md_doctest!(
    "./transformed_md/Names.md",
    transforms = {
        *: {
            ^ => "dummy",
        },
    },
    debug,
);

markdown_doctest::md_doctest!(
    "./transformed_md/Attributes.md",
    transforms = {
        *: {
            ^ => "dummy",
        },
    },
    debug,
);
