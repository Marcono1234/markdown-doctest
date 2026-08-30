use std::collections::HashSet;

use proc_macro2::Span;
use syn::{
    Error, LitStr, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
};

pub(super) struct ParsedMdDoctestConfig {
    pub file_path: LitStr,
    // `Vec<(key, value)>` instead of `HashMap` to preserve order
    pub transforms: Vec<(ParsedCodeBlockName, Vec<ParsedTransform>)>,
    pub debug: bool,
}

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(transforms);
    custom_keyword!(debug);
}

impl Parse for ParsedMdDoctestConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: Maybe alternatively support multiple file paths (as `[file1, file2, ...]`) to allow using the same transforms
        //   for multiple files
        //   - should then check if transform is used by *any* file instead of by every file when reporting error for
        //     unused transforms?
        //   - should include sanitized file name in struct name then (and emit multiple structs?; but only when specifying
        //     multiple files); otherwise cannot tell apart tests in `cargo test` output?
        let file_path = input.parse()?;
        input.parse::<Token![,]>()?;
        input.parse::<keyword::transforms>()?;
        input.parse::<Token![=]>()?;

        let transforms_map_content;
        let braces = braced!(transforms_map_content in input);
        let transforms: Vec<(ParsedCodeBlockName, Vec<ParsedTransform>)> = transforms_map_content
            .parse_terminated(Self::parse_transform_entry, Token![,])?
            .into_iter()
            .collect();

        if transforms.is_empty() {
            return Err(Error::new(
                braces.span.join(),
                "should specify at least one transform",
            ));
        }

        let mut debug = false;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.peek(keyword::debug) {
                input.parse::<keyword::debug>()?;
                debug = true;

                // Allow trailing comma
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }
        }

        let mut has_wildcard = false;
        let mut names = HashSet::new();
        for transform in &transforms {
            match &transform.0 {
                ParsedCodeBlockName::Wildcard(token) => {
                    if has_wildcard {
                        return Err(Error::new(token.span, "duplicate wildcard"));
                    }
                    has_wildcard = true;
                }
                ParsedCodeBlockName::Name { name, name_s } => {
                    if !names.insert(name_s) {
                        return Err(Error::new(name.span(), "duplicate name"));
                    }
                }
            }
        }

        Ok(Self {
            file_path,
            transforms,
            debug,
        })
    }
}

impl ParsedMdDoctestConfig {
    fn parse_transform_entry(
        input: ParseStream,
    ) -> syn::Result<(ParsedCodeBlockName, Vec<ParsedTransform>)> {
        let code_block_name = input.parse()?;
        input.parse::<Token![:]>()?;

        let transforms_content;
        let braces = braced!(transforms_content in input);
        let transforms: Vec<ParsedTransform> = transforms_content
            .parse_terminated(ParsedTransform::parse, Token![,])?
            .into_iter()
            .collect();

        if transforms.is_empty() {
            return Err(Error::new(
                braces.span.join(),
                "should specify at least one transform",
            ));
        }

        Ok((code_block_name, transforms))
    }
}

pub(crate) enum ParsedCodeBlockName {
    Wildcard(Token![*]),
    // TODO: Maybe support also non-empty list of strings to apply transforms to multiple names?
    Name {
        name: LitStr,
        // Cache value to avoid parsing it repeatedly
        name_s: String,
    },
}
impl Parse for ParsedCodeBlockName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![*]) {
            let token = input.parse::<Token![*]>()?;
            Ok(Self::Wildcard(token))
        } else if lookahead.peek(LitStr) {
            let name = input.parse::<LitStr>()?;
            let name_s = name.value();
            Ok(Self::Name { name, name_s })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Where an 'insert' transform should insert the new value
pub(crate) enum InsertKind {
    Before,
    After,
}

/// How to match prefix / suffix for a 'replace' in-line transform
pub(crate) enum ReplaceMatchKind {
    /// No prefix / suffix
    None,
    /// Allow prefix / suffix (but don't replace)
    Match,
    /// Allow prefix / suffix and replace it
    Replace,
}

pub(crate) enum ParsedTransform {
    // Note: For now don't support `(^)` (InsertStartInsideLine) and `($)` (InsertEndInsideLine)
    //   they would mainly be useful for (named) single line code blocks, because they would be
    //   applied to all lines; that feels like a niche use case

    // `^`
    InsertStart {
        insert: Vec<String>,
    },
    // `$`
    InsertEnd {
        insert: Vec<String>,
    },
    // `|...` or `...|`
    InsertLine {
        kind: InsertKind,
        /// Whether the search string must match from the start of the line (`true`), or can have any prefix (`false`)
        match_from_start: bool,
        /// Whether the search string must match to the end of the line (`true`), or can have any suffix (`false`)
        match_to_end: bool,
        search: LitStr,
        // Cache value to avoid parsing it repeatedly
        search_s: String,
        insert: Vec<String>,
    },
    // `(|...)` or `(...|)`
    InsertInsideLine {
        kind: InsertKind,
        // see InsertLine doc
        match_from_start: bool,
        match_to_end: bool,
        search: LitStr,
        // Cache value to avoid parsing it repeatedly
        search_s: String,
        insert: String,
    },
    // `<...>`
    ReplaceLine {
        // see InsertLine doc
        match_from_start: bool,
        match_to_end: bool,
        search: LitStr,
        // Cache value to avoid parsing it repeatedly
        search_s: String,
        insert: Vec<String>,
    },
    // `(<...>)`
    ReplaceInsideLine {
        prefix: ReplaceMatchKind,
        suffix: ReplaceMatchKind,
        search: LitStr,
        // Cache value to avoid parsing it repeatedly
        search_s: String,
        insert: String,
    },
}

impl Parse for ParsedTransform {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        // InsertStart
        if lookahead.peek(Token![^]) {
            input.parse::<Token![^]>()?;
            let insert = Self::parse_insert_lines(input, false)?;
            Ok(Self::InsertStart { insert })
        }
        // InsertEnd
        else if lookahead.peek(Token![$]) {
            input.parse::<Token![$]>()?;
            let insert = Self::parse_insert_lines(input, false)?;
            Ok(Self::InsertEnd { insert })
        }
        // InsertLine(Before)
        else if lookahead.peek(Token![|]) {
            input.parse::<Token![|]>()?;

            let match_from_start = Self::try_parse_star(input)?.is_none();
            let search = Self::parse_search_string(input)?;
            let match_to_end = Self::try_parse_star(input)?.is_none();

            let insert = Self::parse_insert_lines(input, false)?;
            let search_s = search.value();
            Ok(Self::InsertLine {
                kind: InsertKind::Before,
                match_from_start,
                match_to_end,
                search,
                search_s,
                insert,
            })
        }
        // InsertLine (leading *)
        else if lookahead.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Self::parse_insert_line_after(input, false)
        }
        // InsertLine(After)
        else if lookahead.peek(LitStr) {
            Self::parse_insert_line_after(input, true)
        }
        // ReplaceLine
        else if lookahead.peek(Token![<]) {
            input.parse::<Token![<]>()?;

            let match_from_start = Self::try_parse_star(input)?.is_none();
            let search = Self::parse_search_string(input)?;
            let match_to_end = Self::try_parse_star(input)?.is_none();

            input.parse::<Token![>]>()?;

            let insert = Self::parse_insert_lines(
                input, true, // allow empty to remove line
            )?;
            let search_s = search.value();
            Ok(Self::ReplaceLine {
                match_from_start,
                match_to_end,
                search,
                search_s,
                insert,
            })
        }
        // in-line transform
        else if lookahead.peek(syn::token::Paren) {
            Self::parse_in_line_transform(input)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ParsedTransform {
    fn try_parse_star(input: ParseStream) -> syn::Result<Option<Token![*]>> {
        if input.peek(Token![*]) {
            input.parse().map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse_search_string(input: ParseStream) -> syn::Result<LitStr> {
        let value_lit = input.parse::<LitStr>()?;
        if value_lit.value().contains(['\n', '\r']) {
            return Err(Error::new(
                value_lit.span(),
                "search string is matched per line; `\\n` and `\\r` will not match",
            ));
        }

        Ok(value_lit)
    }

    /// Parses `InsertLine(After)`
    fn parse_insert_line_after(input: ParseStream, match_from_start: bool) -> syn::Result<Self> {
        let search = Self::parse_search_string(input)?;
        let match_to_end = Self::try_parse_star(input)?.is_none();

        input.parse::<Token![|]>()?;

        let insert = Self::parse_insert_lines(input, false)?;
        let search_s = search.value();
        Ok(Self::InsertLine {
            kind: InsertKind::After,
            match_from_start,
            match_to_end,
            search,
            search_s,
            insert,
        })
    }

    // Parses an in-line transform `(...)`
    fn parse_in_line_transform(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        // Reassign to prevent accidental usage of original `input`
        let original_input = input;
        let input = content;

        let match_from_start = Self::try_parse_star(&input)?.is_none();

        let lookahead = input.lookahead1();

        // InsertInsideLine(Before)
        if lookahead.peek(Token![|]) {
            input.parse::<Token![|]>()?;

            let search = Self::parse_search_string(&input)?;
            let match_to_end = Self::try_parse_star(&input)?.is_none();

            let insert = Self::parse_insert_inside_line(original_input)?;
            let search_s = search.value();
            Ok(Self::InsertInsideLine {
                kind: InsertKind::Before,
                match_from_start,
                match_to_end,
                search,
                search_s,
                insert,
            })
        }
        // InsertInsideLine(After)
        else if lookahead.peek(LitStr) {
            let search = Self::parse_search_string(&input)?;

            input.parse::<Token![|]>()?;

            let match_to_end = Self::try_parse_star(&input)?.is_none();

            let insert = Self::parse_insert_inside_line(original_input)?;
            let search_s = search.value();
            Ok(Self::InsertInsideLine {
                kind: InsertKind::After,
                match_from_start,
                match_to_end,
                search,
                search_s,
                insert,
            })
        }
        // ReplaceInsideLine
        else if lookahead.peek(Token![<]) {
            input.parse::<Token![<]>()?;

            let mut prefix_match = if match_from_start {
                ReplaceMatchKind::None
            } else {
                ReplaceMatchKind::Match
            };
            if let Some(star) = Self::try_parse_star(&input)? {
                if matches!(prefix_match, ReplaceMatchKind::Match) {
                    return Err(Error::new(
                        star.span,
                        "cannot both match and replace any prefix",
                    ));
                }
                prefix_match = ReplaceMatchKind::Replace;
            }

            let search = Self::parse_search_string(&input)?;

            let mut suffix_match = ReplaceMatchKind::None;
            if Self::try_parse_star(&input)?.is_some() {
                suffix_match = ReplaceMatchKind::Replace;
            }

            input.parse::<Token![>]>()?;

            if let Some(star) = Self::try_parse_star(&input)? {
                if matches!(suffix_match, ReplaceMatchKind::Replace) {
                    return Err(Error::new(
                        star.span,
                        "cannot both match and replace any suffix",
                    ));
                }
                suffix_match = ReplaceMatchKind::Match;
            }

            let insert = Self::parse_insert_inside_line(original_input)?;
            let search_s = search.value();
            Ok(ParsedTransform::ReplaceInsideLine {
                prefix: prefix_match,
                suffix: suffix_match,
                search,
                search_s,
                insert,
            })
        } else {
            Err(lookahead.error())
        }
    }

    /// Parses `=> insert`, for full line transforms
    fn parse_insert_lines(input: ParseStream, allow_empty: bool) -> syn::Result<Vec<String>> {
        input.parse::<Token![=>]>()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            let value_lit = input.parse::<LitStr>()?;
            let value = value_lit.value();

            if value.contains(['\n', '\r']) {
                return Err(Error::new(
                    value_lit.span(),
                    "inserted line must not contain `\\n` or `\\r`; use `[...]` to specify multiple lines to insert",
                ));
            }
            Ok(vec![value])
        } else if lookahead.peek(syn::token::Bracket) {
            fn parse_inserted_line(input: ParseStream) -> syn::Result<String> {
                let value_lit = input.parse::<LitStr>()?;
                let value = value_lit.value();
                if value.contains(['\n', '\r']) {
                    return Err(Error::new(
                        value_lit.span(),
                        "inserted line must not contain `\\n` or `\\r`",
                    ));
                }

                Ok(value)
            }

            let content;
            let brackets = bracketed!(content in input);

            let values: Vec<String> = content
                .parse_terminated(parse_inserted_line, Token![,])?
                .into_iter()
                .collect();

            if !allow_empty && values.is_empty() {
                return Err(Error::new(brackets.span.join(), "must not be empty"));
            }

            Ok(values)
        } else {
            Err(lookahead.error())
        }
    }

    /// Parses `=> insert`, for in-line transforms
    fn parse_insert_inside_line(input: ParseStream) -> syn::Result<String> {
        input.parse::<Token![=>]>()?;
        let value_lit = input.parse::<LitStr>()?;
        let value = value_lit.value();

        if value.contains(['\n', '\r']) {
            return Err(Error::new(
                value_lit.span(),
                "inserted value must not contain `\\n` or `\\r`",
            ));
        }

        Ok(value)
    }
}

impl ParsedTransform {
    pub(crate) fn span_for_no_match(&self) -> Span {
        match self {
            ParsedTransform::InsertStart { .. } | ParsedTransform::InsertEnd { .. } => {
                unreachable!("should always match")
            }
            // TODO: Should instead return complete pattern (including `|`, `<`, ...)?
            ParsedTransform::InsertLine { search, .. } => search.span(),
            ParsedTransform::InsertInsideLine { search, .. } => search.span(),
            ParsedTransform::ReplaceLine { search, .. } => search.span(),
            ParsedTransform::ReplaceInsideLine { search, .. } => search.span(),
        }
    }
}
