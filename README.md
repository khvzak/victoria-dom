# victoria-dom
Minimalistic HTML parser with CSS selectors

[![crates.io](https://img.shields.io/crates/v/victoria-dom.svg)](https://crates.io/crates/victoria-dom)
[![Build Status](https://travis-ci.org/khvzak/victoria-dom.svg?branch=master)](https://travis-ci.org/khvzak/victoria-dom)
[![Coverage Status](https://coveralls.io/repos/github/khvzak/victoria-dom/badge.svg?branch=master)](https://coveralls.io/github/khvzak/victoria-dom?branch=master)
[![Released API docs](https://docs.rs/victoria-dom/badge.svg)](https://docs.rs/victoria-dom)
[![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://khvzak.github.io/victoria-dom/)

The project has been inspired by [Mojo::DOM](https://metacpan.org/pod/Mojo::DOM).

### Installing
Add the following lines to your `Cargo.toml` file:

```toml
[dependencies]
victoria-dom = "0.1"
```

and this to your crate root:
```rust
extern crate victoria_dom;
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
https://docs.rs/victoria-dom
