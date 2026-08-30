# Usage

This library consists of the `markdown_doctest::md_doctest!` macro, which should be called anywhere from the main source (for example from `src/lib.rs`)
and be guarded with `#[cfg(doctest)]`. This makes sure that this part of the code is only built during dev builds (e.g. `cargo test`) and allows declaring
the dependency on this library under `dev-dependencies`.

<!-- TODO: Is this really a good tip, or does it have any undesired side-effects? -->
> [!TIP]
> Instead of `#[cfg(doctest)]` you can use `#[cfg(any(test, doctest))]` for better IDE support, because by default the IDE might not build
> and validate code guarded with `#[cfg(doctest)]`, making it difficult to use the macro and react to compiler errors.

```rust
#[cfg(doctest)]
markdown_doctest::md_doctest!(
    "../README.md",
    transforms = {
        ...
    }
);
```

When you then run `cargo test` (which includes running doctests), the code blocks from the Markdown file are run as doctests as well.

> [!IMPORTANT]
>
> - This library performs transformations at Markdown text level; be careful when inserting text containing ```` ``` ```` since that
>   might accidentally terminate code blocks.
> - Some IDEs do not support all macro functionality needed by this library, so automatic recompilation might not happen and "Expand macro" might not work (related [rust-analyzer issue](https://github.com/rust-lang/rust-analyzer/issues/15950)).
> - The output of `cargo test` does not directly point to the location of the (transformed) doctests included by this library.
>   However, the library influences the output to make it a bit easier to understand:
>   - The doctest is placed on a struct with name `MarkdownDoctest_Line_<line>`, where `<line>` is the line number in which `md_doctest!` was called.
>   - Line numbers within the doctest are offset by 1000, e.g. 1001 means line 1 in the _transformed_ source. See also the section below for enabling `debug` output for the macro.

## Macro arguments

The macro takes two main arguments:

- Markdown file path, relative to the parent directory of the file which is invoking the macro (e.g. `src/lib.rs`)
- `transforms = { ... }`
- optional: `debug`

  Appended after the transforms: `transforms = { ... }, debug`. This creates a sibling file next to the original Markdown file with file name suffix `.md_doctest_debug.md`,
  containing the transformed Markdown source which will be used as doc comment and from which the doctests will be extracted by `cargo test`.\
  The behavior and output of this `debug` argument may change in the future; it should only be used for temporary troubleshooting.

The transforms are grouped by code block name (see also the ["HTML comment directives" section](#html-comment-directives) below).
The wildcard `*` applies to all code blocks, named or unnamed. The same code block can be matched by multiple names (or the wildcard),
causing all of their transforms to be applied.

```rust
markdown_doctest::md_doctest!(
    "../README.md",
    transforms = {
        // wildcard transforms; apply to all code blocks
        *: {
            "some line"| => "inserted line",
            ...
        },
        // transforms for code blocks with name "custom-name-a"
        "custom-name-a": {
            ...
        },
        // transforms for code blocks with name "custom-name-b"
        "custom-name-b": {
            ...
        },
    }
);
```

## Transforms

Transforms have this syntax:

<pre>
<i>&lt;pattern&gt;</i> =&gt; <i>&lt;new-value&gt;</i>
</pre>

For every line in a code block (respectively substring in the line) matching the pattern, the new value is applied. See the sections below for details
about the patterns and their behavior.

Transforms are applied in the order they are declared, and operate on the results of previously executed patterns.

### Full line transforms

These transforms operate on a full line, either inserting a new line before / after or replacing the line.

Patterns:

- `^`: insert as first line
- `$`: insert as last line
- `|"..."`: insert before line matching string
- `"..."|`: insert after line matching string
- `<"...">`: replace line matching string

New value:

- a string literal `"..."`
- multiple string literals written as `[ ... ]`\
  For "replace line" it can be empty (`[]`) to remove matching lines instead of replacing them.

Syntax:
<pre>
<i>&lt;pattern&gt;</i> =&gt; "<i>&lt;new-value&gt;</i>"
<i>&lt;pattern&gt;</i> =&gt; [<i>"&lt;new-value&gt;</i>", ...]
</pre>

For the transforms with a `"..."` pattern the full line content must match. Alternatively `*` can be used before and / or after the string to permit any prefix / suffix in the line. For example `*"world"` matches the line `hello world`.

<table>
<thead>
<tr>
<th scope="col" width="33%">Markdown (source)</th>
<th scope="col" width="33%"><code>md_doctest!</code></th>
<th scope="col" width="33%">Markdown (result)</th>
</tr>
</thead>
<tbody>

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
println!("hello {name}");
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // ^ = insert at start
      ^ => "let name = \"Bob\";",
    },
  }
);
```

</td>
<td>

```rust
let name = "Bob";
println!("hello {name}");
```

</td>
</tr>

<!-- new row -->
<tr>
<td>

```rust
let value = get_value();
let result = fs::write(file, value);
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // insert multiple lines at start
      ^ => [
        "use std::fs;",
        "use my_crate::get_value;",
      ],
    },
  }
);
```

</td>
<td>

```rust
use std::fs;
use my_crate::get_value;
let value = get_value();
let result = fs::write(file, value);
```

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
std::fs::write(file, content)?;
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // $ = insert at end
      $ => "Ok::<(), Box<dyn std::error::Error>>(())",
    },
  }
);
```

</td>
<td>

```rust
std::fs::write(file, content)?;
Ok::<(), Box<dyn std::error::Error>>(())
```

</td>
</tr>

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
let mut s = String::new();
// ...
println!("message: {s}");
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // "..."| = insert after line
      // surrounding '*' allows any prefix / suffix
      *"let mut s"*| => "s.push_str(\"hello\");",
    },
  }
);
```

</td>
<td>

```rust
let mut s = String::new();
s.push_str("hello");
// ...
println!("message: {s}");
```

</td>
</tr>

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
let mut s = String::new();
// ... set content
println!("message: {s}");
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // <"..."> = replace line
      // trailing '*' allows any suffix
      <"// ..."*> => "s.push_str(\"hello\");",
    },
  }
);
```

</td>
<td>

```rust
let mut s = String::new();
s.push_str("hello");
println!("message: {s}");
```

</td>
</tr>

</tbody>
</table>

### In-line transforms

These transforms operate within a line, changing its value. They are similar to the "full line transforms", except that their patterns are enclosed in `(...)`.

Patterns:

- `(|"...")`: insert in line before matching string
- `("..."|)`: insert in line after matching string
- `(<"...">)`: replace matching string in line

By default the `"..."` part of the pattern is expected to start at the start of the line and end at the end of the line. This can be changed by using `(*` and / or `*)` to allow any prefix / suffix.

Additionally for replacing content in a line with `(<"...">)`, `<*` and / or `*>` can be used to match any prefix / suffix _and_ to replace it. This cannot be combined with matching any prefix / suffix, that is, `(*<*...` or `...*>*)` is not allowed.

A pattern can match multiple substrings in a line, for example applying `(*|"do"*) => "to "` to the line `do or not do` changes it to `to do or not to do`.

<table>
<thead>
<tr>
<th scope="col" width="33%">Markdown (source)</th>
<th scope="col" width="33%"><code>md_doctest!</code></th>
<th scope="col" width="33%">Markdown (result)</th>
</tr>
</thead>
<tbody>

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
println!("hello new world");
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // insert in front of substring " new "
      (*|" new "*) => "beautiful ",
    },
  }
);
```

</td>
<td>

```rust
println!("hello beautiful new world");
```

</td>
</tr>

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
std::fs::write(file, ...)?;
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // replace "..."
      (*<"...">*) => "\"test content\"",
    },
  }
);
```

</td>
<td>

```rust
std::fs::write(file, "test content")?;
```

</td>
</tr>

<!-- new row -->
</td>
</tr>

<tr>
<td>

```rust
fn pi() -> f64 {
    3.1415
}
```

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    *: {
      // replace "3.14" including any suffix
      (*<"3.14"*>) => "3.1415926535",
    },
  }
);
```

</td>
<td>

```rust
fn pi() -> f64 {
    3.1415926535
}
```

</td>
</tr>

</tbody>
</table>

## HTML comment directives

Currently two HTML comment directives are supported, which must appear immediately in front of the opening ```` ```rust ```` marker of a code block.

- names: `<!-- markdown-doctest-names: <names> -->`

    Specifies a comma-separated list of names for the code block. Names identify code blocks and allow applying transformations only to code blocks with a certain name.
    They can also be used as tag by applying the same name to multiple code blocks, for example to all code blocks which should have a `Ok` result as last line.
    By default a code block has no name, and only the wildcard `*` matches it.

- attributes: `<!-- markdown-doctest-attributes: <attributes> -->`

    Specifies [doctest attributes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#attributes) which should be used instead of `rust`. The value is applied as is, no splitting of values is performed.

If both 'names' and 'attributes' are provided then 'names' must come first. Neither duplicate directives for the same code block nor dangling directives without any code block are allowed.

<table>
<thead>
<tr>
<th scope="col" width="33%">Markdown (source)</th>
<th scope="col" width="33%"><code>md_doctest!</code></th>
<th scope="col" width="33%">Markdown (result)</th>
</tr>
</thead>
<tbody>

<!-- new row -->
<tr>
<td>

No name, only the wildcard matches this block:

````markdown
```rust
let value = get_value()?;
println!("value: {value}");
```
````

Block with custom name:

````markdown
<!-- markdown-doctest-names: fs-usage -->
```rust
let value = get_value()?;
fs::write(file, value)?;
```
````

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    // applied to all code blocks
    *: {
      // return Ok
      $ => "Ok::<(), Box<dyn std::error::Error>>(())",
    },
    // applied only to code blocks with name "fs-usage"
    "fs-usage": {
      // add import
      ^ => "use std::fs;",
    },
  }
);
```

</td>
<td>

````markdown
```rust
let value = get_value()?;
println!("value: {value}");
Ok::<(), Box<dyn std::error::Error>>(())
```

```rust
use std::fs;
let value = get_value()?;
fs::write(file, value)?;
Ok::<(), Box<dyn std::error::Error>>(())
```
````

<!-- new row -->
<tr>
<td>

Using names as 'tags'. This example uses `unwrap()` instead of `?` so it only has the tag "fs-usage":

````markdown
<!-- markdown-doctest-names: fs-usage -->
```rust
let value = get_value().unwrap();
fs::write(file, value).unwrap();
```
````

This example uses `?` and therefore additionally has the tag "ok-return":

````markdown
<!-- markdown-doctest-names: fs-usage,ok-return -->
```rust
let value = get_value()?;
fs::write(file, value)?;
```
````

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    "fs-usage": {
      // add import
      ^ => "use std::fs;",
    },
    "ok-return": {
      // return Ok
      $ => "Ok::<(), Box<dyn std::error::Error>>(())",
    },
  }
);
```

</td>
<td>

````markdown
```rust
use std::fs;
let value = get_value().unwrap();
fs::write(file, value).unwrap();
```

```rust
use std::fs;
let value = get_value()?;
fs::write(file, value)?;
Ok::<(), Box<dyn std::error::Error>>(())
```
````

<!-- new row -->
<tr>
<td>

Specifying both a name and doctest attributes:

````markdown
<!-- markdown-doctest-names: ok-return -->
<!-- markdown-doctest-attributes: should_panic -->
```rust
let value = get_value()?;
assert_eq!(value, 1);
```
````

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  transforms = {
    "ok-return": {
      // return Ok
      $ => "Ok::<(), Box<dyn std::error::Error>>(())",
    },
  }
);
```

</td>
<td>

````markdown
```should_panic
let value = get_value()?;
assert_eq!(value, 1);
Ok::<(), Box<dyn std::error::Error>>(())
```
````

<!-- new row -->
<tr>
<td>

Attribute `ignore` completely ignores the code block and does not apply transforms. Prefer the attribute `no_run` if the code should still be compiled but not be run.

````markdown
<!-- markdown-doctest-attributes: ignore -->
```rust
error.into()
```
````

</td>
<td>

```rust
md_doctest!(
  "MyFile.md",
  ...
);
```

</td>
<td>

_no Rust code block_

</td>
</tr>

</tbody>
</table>
