# victoria-dom
Minimalistic HTML parser with CSS selectors

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

The project has been inspired by [Mojo::DOM](https://metacpan.org/pod/Mojo::DOM).

### Installing
Add the following lines to the `Cargo.toml` file:

```toml
[dependencies]
victoria-dom = "0.1.*"
```

### Examples
```rust
extern crate victoria_dom;

use victoria_dom::DOM;

fn main() {
    let html = r#"<html><div id="main">Hello, <a href="http://rust-lang.org" alt="The Rust Programing Language">Rust</a></div></html>"#;
    let dom = DOM::new(html);

    assert_eq!(dom.at("html").unwrap().text_all(), "Hello, Rust");
    assert_eq!(dom.at("div#main > a").unwrap().attr("alt").unwrap(), "The Rust Programing Language");
}
```

### Documentation
https://khvzak.github.io/victoria-dom/
