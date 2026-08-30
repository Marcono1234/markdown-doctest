Some small code snippets.

```rust
// Explicit return
return;
```

<!-- markdown-doctest-names: ok-return -->
```rust
let i: u32 = "123".parse()?;
assert_eq!(i, 123);
```

<!-- markdown-doctest-attributes: should_panic -->
```rust
panic!("test");
```
