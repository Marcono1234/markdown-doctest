Tests complex Markdown handling such as code blocks in quotes, lists, non-Rust code blocks ...

- ```rust
    first
        second
    ```

- outer
  1. inner
    - inner inner
        1. ```rust
            nested
            inside lists
            ```

> ```rust
> This one is
> in a quote
> ```

Nested:

> > > - ```rust
> > >   nested
> > >   quote
> > >   ```

With trailing space after `rust`:

```rust  
trailing
space
```

With trailing space after closing backticks:

```rust
trailing space
after
```  

```rust
  bad
  indented
   ```

````rust
more than ```
```nested
this is still inside rust code block
```
````

```
plain
text
```

```java
other
language
```

``` inline code ``` even with ```` ```rust ````
and another line
and these ``` are not closing backticks ```
