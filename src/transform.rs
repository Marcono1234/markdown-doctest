use std::{collections::HashSet, ops::Range};

use proc_macro2::Span;
use syn::Error;

use crate::{
    parse_macro::{InsertKind, ParsedCodeBlockName, ParsedTransform, ReplaceMatchKind},
    parse_md::CodeBlock,
};

const LINE_BREAK: &str = "\n";

// TODO Add unit tests? currently only covered by integration tests

/// Applies transforms and creates a Markdown doc string containing all code blocks
pub(crate) fn transform(
    md_file_span: Span,
    code_blocks: &[CodeBlock],
    transforms: &[(ParsedCodeBlockName, Vec<ParsedTransform>)],
) -> syn::Result<String> {
    // Caller should have verified this; otherwise no code blocks lead to confusing errors
    // about no transform matching
    assert!(!code_blocks.is_empty());
    assert!(!transforms.is_empty());

    let all_code_block_names: HashSet<_> = code_blocks.iter().flat_map(|c| &c.names).collect();
    let error = transforms
        .iter()
        .filter_map(|t| match &t.0 {
            ParsedCodeBlockName::Wildcard(_) => None,
            ParsedCodeBlockName::Name { name, name_s } => {
                if all_code_block_names.contains(name_s) {
                    None
                } else {
                    Some(name)
                }
            }
        })
        .map(|name| Error::new(name.span(), "no code block with this name exists"))
        .reduce(|mut existing, e| {
            existing.combine(e);
            existing
        });

    if let Some(error) = error {
        return Err(error);
    }

    let mut transforms: Vec<(_, Vec<TrackedTransform>)> = transforms
        .iter()
        .map(|(name, transforms)| {
            (
                name,
                transforms
                    .iter()
                    .map(|transform| TrackedTransform {
                        transform,
                        matched_something: false,
                    })
                    .collect(),
            )
        })
        .collect();

    let mut only_ignored_code_blocks = true;
    let mut md_result = String::new();

    for code_block in code_blocks {
        md_result.push_str(&format!("line: {}", code_block.start_line));

        let names = &code_block.names;
        if names.is_empty() {
            md_result.push_str(LINE_BREAK);
        } else {
            md_result.push('\\'); // hard line break after line number text
            md_result.push_str(LINE_BREAK);
            md_result.push_str(&format!("names: {}{LINE_BREAK}", names.join(", ")));
        }

        if code_block.rustdoc_attributes == Some("ignore".to_owned()) {
            // For unconditional ignore (unlike `ignore-<target>`) completely skip the block and
            // don't apply transforms so that they are not spuriously marked as used
            // This assumes user used `ignore` for permanently ignored snippet and not as temporary solution

            md_result.push_str(LINE_BREAK);
            md_result.push_str("_ignored_");
        } else {
            only_ignored_code_blocks = false;

            let transforms = transforms
                .iter_mut()
                .filter_map(|t| match &t.0 {
                    ParsedCodeBlockName::Wildcard(_) => Some(&mut t.1),
                    ParsedCodeBlockName::Name { name_s, .. } => {
                        if names.contains(name_s) {
                            Some(&mut t.1)
                        } else {
                            None
                        }
                    }
                })
                .flatten()
                .collect();
            let transformed = apply_transforms(code_block, transforms)?;
            md_result.push_str(&transformed);
        }

        md_result.push_str(LINE_BREAK);
        md_result.push_str(LINE_BREAK);
        md_result.push_str("---"); // Markdown horizontal rule
        md_result.push_str(LINE_BREAK);
        md_result.push_str(LINE_BREAK);
    }

    // Check this here to avoid spurious errors below about no transform matching
    if only_ignored_code_blocks {
        return Err(Error::new(
            md_file_span,
            "all code blocks have `ignore` attribute",
        ));
    }

    let error = transforms
        .iter()
        .flat_map(|t| &t.1)
        .filter(|t| !t.matched_something)
        .map(|t| {
            // TODO(rust): Report as warning instead, once supported by Rust, see https://github.com/rust-lang/rust/issues/54140
            Error::new(
                t.transform.span_for_no_match(),
                "does not match any line in any code block",
            )
        })
        .reduce(|mut existing, e| {
            existing.combine(e);
            existing
        });
    if let Some(error) = error {
        return Err(error);
    }

    Ok(md_result)
}

struct TrackedTransform<'a> {
    transform: &'a ParsedTransform,
    matched_something: bool,
}

fn insert_multiple<T: Clone>(vec: &mut Vec<T>, index: usize, insert: &[T], replace: bool) {
    vec.splice(
        index..(if replace { index + 1 } else { index }),
        insert.iter().cloned(),
    );
}

// TODO: Decouple this from syn, to make it testable on its own
fn apply_transforms<'a>(
    code_block: &CodeBlock,
    transforms: Vec<&mut TrackedTransform<'a>>,
) -> syn::Result<String> {
    let backticks = "`".repeat(code_block.backtick_count as usize);
    let mut md_result = String::new();
    md_result.push_str(&backticks);
    md_result.push_str(code_block.rustdoc_attributes.as_ref().map_or("rust", |s| s));
    md_result.push_str(LINE_BREAK);

    let mut lines = code_block.lines.clone();
    for tracked_transform in transforms {
        let transform = tracked_transform.transform;

        fn find_lines(
            lines: &[String],
            search: &str,
            match_from_start: bool,
            match_to_end: bool,
            find_all_in_line: bool,
        ) -> Vec<(usize, Vec<Range<usize>>)> {
            let search_len = search.len();
            let mut result = Vec::new();

            for (line_index, line) in lines.iter().enumerate() {
                match (match_from_start, match_to_end) {
                    (true, true) => {
                        if line == search {
                            #[expect(
                                clippy::single_range_in_vec_init,
                                reason = "intentionally creates a Vec containing a single Range"
                            )]
                            result.push((line_index, vec![0..search_len]));
                        }
                    }
                    (true, false) => {
                        if line.starts_with(search) {
                            #[expect(
                                clippy::single_range_in_vec_init,
                                reason = "intentionally creates a Vec containing a single Range"
                            )]
                            result.push((line_index, vec![0..search_len]));
                        }
                    }
                    (false, true) => {
                        if line.ends_with(search) {
                            #[expect(
                                clippy::single_range_in_vec_init,
                                reason = "intentionally creates a Vec containing a single Range"
                            )]
                            result.push((line_index, vec![line.len() - search_len..line.len()]));
                        }
                    }
                    (false, false) => {
                        let ranges: Vec<Range<usize>> = if find_all_in_line {
                            line.match_indices(search)
                                .map(|m| m.0..m.0 + search_len)
                                .collect()
                        } else {
                            #[expect(
                                clippy::single_range_in_vec_init,
                                reason = "intentionally creates a Vec containing a single Range"
                            )]
                            line.find(search)
                                .map_or_else(Vec::new, |i| vec![i..i + search_len])
                        };
                        if !ranges.is_empty() {
                            result.push((line_index, ranges));
                        }
                    }
                }
            }

            result
        }

        match &transform {
            ParsedTransform::InsertStart { insert } => {
                tracked_transform.matched_something = true;
                insert_multiple(&mut lines, 0, insert, false);
            }
            ParsedTransform::InsertEnd { insert } => {
                tracked_transform.matched_something = true;
                lines.extend_from_slice(insert);
            }
            ParsedTransform::InsertLine {
                kind,
                match_from_start,
                match_to_end,
                search: _,
                search_s,
                insert,
            } => {
                let matches = find_lines(
                    &lines,
                    search_s,
                    *match_from_start,
                    *match_to_end,
                    // just care if there is any match in the line
                    false,
                );
                if !matches.is_empty() {
                    tracked_transform.matched_something = true;
                }

                // Process in reverse order so that the insertion indices remain correct
                for (mut line_index, _) in matches.into_iter().rev() {
                    if matches!(kind, InsertKind::After) {
                        line_index += 1;
                    }

                    insert_multiple(&mut lines, line_index, insert, false);
                }
            }
            ParsedTransform::InsertInsideLine {
                kind,
                match_from_start,
                match_to_end,
                search: _,
                search_s,
                insert,
            } => {
                let matches = find_lines(&lines, search_s, *match_from_start, *match_to_end, true);
                if !matches.is_empty() {
                    tracked_transform.matched_something = true;
                }

                for (line_index, ranges) in matches {
                    let line = &mut lines[line_index];
                    // Process in reverse order so that the range indices remain correct
                    for Range { start, end } in ranges.into_iter().rev() {
                        let insert_index = if matches!(kind, InsertKind::Before) {
                            start
                        } else {
                            end
                        };
                        line.insert_str(insert_index, insert);
                    }
                }
            }
            ParsedTransform::ReplaceLine {
                match_from_start,
                match_to_end,
                search: _,
                search_s,
                insert,
            } => {
                let matches = find_lines(
                    &lines,
                    search_s,
                    *match_from_start,
                    *match_to_end,
                    // just care if there is any match in the line
                    false,
                );
                if !matches.is_empty() {
                    tracked_transform.matched_something = true;
                }

                // Process in reverse order so that the insertion indices remain correct
                for (line_index, _) in matches.into_iter().rev() {
                    if insert.is_empty() {
                        lines.remove(line_index);
                    } else if insert.len() == 1 {
                        lines[line_index].clone_from(&insert[0]);
                    } else {
                        insert_multiple(&mut lines, line_index, insert, true);
                    }
                }
            }
            ParsedTransform::ReplaceInsideLine {
                prefix,
                suffix,
                search: _,
                search_s,
                insert,
            } => {
                let matches = find_lines(
                    &lines,
                    search_s,
                    matches!(prefix, ReplaceMatchKind::None),
                    matches!(suffix, ReplaceMatchKind::None),
                    true,
                );
                if !matches.is_empty() {
                    tracked_transform.matched_something = true;
                }

                for (line_index, ranges) in matches {
                    let line = &mut lines[line_index];
                    // Process in reverse order so that the range indices remain correct
                    for Range { mut start, mut end } in ranges.into_iter().rev() {
                        if matches!(prefix, ReplaceMatchKind::Replace) {
                            start = 0;
                        }
                        if matches!(suffix, ReplaceMatchKind::Replace) {
                            end = line.len();
                        }

                        line.replace_range(start..end, insert);
                    }
                }
            }
        }
    }

    for line in lines {
        md_result.push_str(&line);
        md_result.push_str(LINE_BREAK);
    }

    md_result.push_str(&backticks);
    Ok(md_result)
}
