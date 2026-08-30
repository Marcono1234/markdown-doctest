// Logic here roughly follows https://spec.commonmark.org/0.31.2/#fenced-code-blocks
// Full CommonMark parsing might be exaggerated for now?
// -> TODO: ? Actually use CommonMark and mirror logic in https://github.com/rust-lang/rust/blob/e457a7b0d326d67b4322ef0d11bd715cfaeda48f/src/librustdoc/html/markdown.rs#L737
//    (mention it as comment as well)

use std::{collections::HashSet, ops::Range};

// TODO Add unit tests? currently only covered by integration tests

// TODO(rust): For errors report Span within the Markdown file, once supported, see https://github.com/rust-lang/rfcs/issues/2869
pub(super) fn parse_md(content: &str) -> Result<Vec<CodeBlock>, String> {
    let mut code_blocks = Vec::new();

    let mut line_number = 0;

    // structure: `(line_number, value)`
    let mut comment_names: Option<(u32, String)> = None;
    let mut comment_attributes: Option<(u32, String)> = None;

    // These vars are for the currently processed code block
    let mut start_line: Option<u32> = None;
    let mut backtick_count = 0;
    let mut code_lines = Vec::<String>::new();

    for line in content.lines() {
        // Start line numbering at 1, to match how rustc reports errors
        line_number += 1;

        if start_line.is_none() {
            let trimmed_line = line.trim_ascii_end();

            let LineMarkerComments { names, attributes } =
                parse_marker_comments(trimmed_line, line_number)?;
            if let Some(names) = names {
                if let Some((existing_line, _)) = comment_names {
                    return Err(format!(
                        "names comment already exists at line {existing_line}"
                    ));
                }
                if let Some((existing_attributes_line, _)) = comment_attributes {
                    return Err(format!(
                        "names comment should come before attributes at line {existing_attributes_line}"
                    ));
                }

                comment_names = Some((line_number, names));
            }
            if let Some(attributes) = attributes {
                if let Some((existing_line, _)) = comment_attributes {
                    return Err(format!(
                        "attributes comment already exists at line {existing_line}"
                    ));
                }
                comment_attributes = Some((line_number, attributes));
            }

            // Note: Intentionally do not support Markdown `~~~` for now; in case support for it is added,
            //   need to preserve it also in output because code block content may contain ```;
            //   or alternatively after transformation check for ``` (and more consecutive backticks) and then
            //   use that + 1 as number of backticks for the enclosing ```rust; update warning in Usage guide then
            let start_text = "```rust";
            if trimmed_line.ends_with(start_text) {
                if comment_names.as_ref().is_some_and(|c| c.0 == line_number)
                    || comment_attributes
                        .as_ref()
                        .is_some_and(|c| c.0 == line_number)
                {
                    // If comment is in front of ```rust, then CommonMark does not seem to recognize
                    // it as code block
                    return Err(format!(
                        "comment cannot be on same line as code block start at line {line_number}"
                    ));
                }

                start_line = Some(line_number);

                let mut backticks = 3;
                // `unwrap()` should be safe due to enclosing `ends_with` check
                let mut search_index = trimmed_line.rfind(start_text).unwrap();
                while search_index > 0 {
                    if trimmed_line.as_bytes()[search_index - 1] == b'`' {
                        backticks += 1;
                    } else {
                        break;
                    }
                    search_index -= 1;
                }
                backtick_count = backticks;
            }
            // check dangling names comment
            else if let Some((comment_line, _)) = comment_names
                && comment_line != line_number
                // And there is no immediate attributes comment afterwards
                && comment_attributes
                    .as_ref()
                    .is_none_or(|c| c.0 != comment_line + 1)
            {
                return Err(format!("dangling names comment at line {comment_line}"));
            }
            // check dangling attributes comment
            else if let Some((comment_line, _)) = comment_attributes
                && comment_line != line_number
            {
                return Err(format!(
                    "dangling attributes comment at line {comment_line}"
                ));
            }
        } else {
            let trimmed_line = line.trim_ascii_end();

            let mut backticks = 0;
            let mut search_index = trimmed_line.len();
            while search_index > 0 {
                if trimmed_line.as_bytes()[search_index - 1] == b'`' {
                    backticks += 1;
                } else {
                    break;
                }
                search_index -= 1;
            }

            // Check if end marker was found
            // Also permit if there are more backticks than opening one
            if backticks < backtick_count {
                code_lines.push(line.to_owned());
            } else {
                // Prefix to trim is determined based on prefix of closing ```
                let prefix = &trimmed_line[0..search_index];
                let trimmed_prefix = prefix.trim_ascii_end();

                // Trim prefix from code lines
                for l in code_lines.iter_mut() {
                    if l.starts_with(prefix) {
                        l.drain(0..prefix.len());
                    }
                    // Also account for bogus code block inside block quote, where individual lines have less indentation
                    // than closing ```
                    else if l.starts_with(trimmed_prefix) {
                        l.drain(0..trimmed_prefix.len());
                    } else {
                        // To be safe, fail here instead of causing potentially malformed code block
                        return Err(format!("failed trimming prefix for line: {l}"));
                    }
                }

                fn split_names(names: &str, line_number: u32) -> Result<Vec<String>, String> {
                    let names: Vec<String> = names.split(",").map(|s| s.to_owned()).collect();
                    if names.is_empty() {
                        return Err(format!("names must not be empty at line {line_number}"));
                    }

                    let mut names_set = HashSet::new();

                    for n in &names {
                        if n.is_empty() {
                            return Err(format!("contains empty name at line {line_number}"));
                        }
                        if !names_set.insert(n) {
                            return Err(format!(
                                "contains duplicate name '{n}' at line {line_number}"
                            ));
                        }
                    }
                    Ok(names)
                }

                code_blocks.push(CodeBlock {
                    start_line: start_line.unwrap(),
                    names: comment_names
                        .map_or_else(|| Ok(Vec::new()), |c| split_names(&c.1, c.0))?,
                    rustdoc_attributes: comment_attributes.map(|c| c.1),
                    backtick_count,
                    lines: code_lines,
                });

                start_line = None;
                backtick_count = 0;
                code_lines = Vec::new();

                comment_names = None;
                comment_attributes = None;
            }
        }
    }

    if let Some(start_line) = start_line {
        Err(format!(
            "failed finding closing ``` for code block started at line {start_line}"
        ))
    } else {
        Ok(code_blocks)
    }
}

struct LineMarkerComments {
    names: Option<String>,
    attributes: Option<String>,
}

fn parse_marker_comments(line: &str, line_number: u32) -> Result<LineMarkerComments, String> {
    fn parse_comment(
        line: &str,
        line_number: u32,
        kind: &str,
    ) -> Result<Option<(Range<usize>, String)>, String> {
        let comment_start = format!("<!-- markdown-doctest-{kind}:");
        let comment_end = "-->";
        if let Some(name_index) = line.find(&comment_start) {
            let search_start_index = name_index + comment_start.len();
            if line[search_start_index..].find(&comment_start).is_some() {
                // Duplicate comment in same line
                return Err(format!("duplicate {kind} comment at line {line_number}"));
            }

            if let Some(end_index) = line[search_start_index..].find(comment_end) {
                let range = search_start_index..search_start_index + end_index;
                let value = line[range.clone()].trim_ascii().to_owned();

                if value.is_empty() {
                    return Err(format!(
                        "empty value for {kind} comment at line {line_number}"
                    ));
                }

                Ok(Some((range, value)))
            } else {
                Err(format!("missing comment end at line {line_number}"))
            }
        } else {
            Ok(None)
        }
    }

    let names_comment = parse_comment(line, line_number, "names")?;
    let attributes_comment = parse_comment(line, line_number, "attributes")?;

    if let Some((
        Range {
            start: names_start,
            end: names_end,
        },
        _,
    )) = names_comment
        && let Some((
            Range {
                end: attributes_end,
                ..
            },
            _,
        )) = attributes_comment
    {
        if names_start > attributes_end {
            return Err(format!(
                "names comment should come before attributes comment in line {line_number}"
            ));
        }
        if names_end == attributes_end {
            // If both share the same "-->" then one is missing
            return Err(format!(
                "missing comment end for names comment at line {line_number}"
            ));
        }
    }

    Ok(LineMarkerComments {
        names: names_comment.map(|c| c.1),
        attributes: attributes_comment.map(|c| c.1),
    })
}

pub(crate) struct CodeBlock {
    /// Line number (starting at 1) of the opening ```` ```rust ````
    pub start_line: u32,
    /// Names of the code block; can be empty
    pub names: Vec<String>,
    pub rustdoc_attributes: Option<String>,
    pub backtick_count: u32,
    pub lines: Vec<String>,
}
