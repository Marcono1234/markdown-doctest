<!-- markdown-doctest-attributes: should_panic -->
```rust
single attribute
```

<!-- markdown-doctest-attributes: edition2018,compile_fail -->
```rust
multiple attributes
```

<!-- markdown-doctest-attributes: compile_fail -->
```rust
compile_fail should still appear in output
```

<!-- markdown-doctest-attributes: no_run -->
```rust
no_run should still appear in output
```

<!-- markdown-doctest-attributes: ignore -->
```rust
this should not appear in output
```

<!-- markdown-doctest-attributes: ignore-x86_64 -->
```rust
target ignore should appear in output
```

<!-- markdown-doctest-attributes: ignore-x86_64,ignore-windows -->
```rust
multiple target ignores
```

- <!-- markdown-doctest-attributes: no_run -->
    ```rust
    list
    ```

<!-- markdown-doctest-names: list -->
<!-- markdown-doctest-attributes: no_run -->
```rust
names and attributes
```

<!-- markdown-doctest-names: list --> <!-- markdown-doctest-attributes: no_run -->
```rust
names and attributes same line
```

- <!-- markdown-doctest-names: list -->
    <!-- markdown-doctest-attributes: no_run -->
    ```rust
    list with names and attributes
    ```

> <!-- markdown-doctest-names: quote -->
> <!-- markdown-doctest-attributes: no_run -->
> ```rust
> block quote with names and attributes
> ```

```rust
ensure that previous attributes were cleared; this should have no attributes
```
