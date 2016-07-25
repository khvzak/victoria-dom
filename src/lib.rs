// lib.rs
// This crate is a library
#![crate_type = "lib"]
// The library is named "victoria_dom"
#![crate_name = "victoria_dom"]

//! Minimalistic HTML parser with CSS selectors
//!
//! The project has been inspired by [Mojo::DOM](https://metacpan.org/pod/Mojo::DOM).
//!
//! It will even try to interpret broken HTML, so you should not use it for validation.
//!
//! # Examples
//!
//! ```
//! extern crate victoria_dom;
//!
//! use victoria_dom::DOM;
//!
//! fn main() {
//!     let html = r#"<html><div id="main">Hello, <a href="http://rust-lang.org" alt="The Rust Programing Language">Rust</a></div></html>"#;
//!     let dom = DOM::new(html);
//!
//!     assert_eq!(dom.at("html").unwrap().text_all(), "Hello, Rust");
//!     assert_eq!(dom.at("div#main > a").unwrap().attr("alt").unwrap(), "The Rust Programing Language");
//! }
//! ```
//!
//! # Supported CSS selectors
//!
//! * `*` Any element.
//! * `E` An element of type `E`.
//! * `E[foo]` An `E` element with a `foo` attribute.
//! * `E[foo="bar"]` An `E` element whose `foo` attribute value is exactly equal to `bar`.
//! * `E[foo~="bar"]` An `E` element whose `foo` attribute value is a list of whitespace-separated values, one of which is exactly equal to `bar`.
//! * `E[foo^="bar"]` An `E` element whose `foo` attribute value begins exactly with the string `bar`.
//! * `E[foo$="bar"]` An `E` element whose `foo` attribute value ends exactly with the string `bar`.
//! * `E[foo*="bar"]` An `E` element whose `foo` attribute value contains the substring `bar`.
//! * `E:root` An `E` element, root of the document.
//! * `E:nth-child(n)` An `E` element, the `n-th` child of its parent.
//! * `E:nth-last-child(n)` An `E` element, the `n-th` child of its parent, counting from the last one.
//! * `E:nth-of-type(n)` An `E` element, the `n-th` sibling of its type.
//! * `E:nth-last-of-type(n)` An `E` element, the `n-th` sibling of its type, counting from the last one.
//! * `E:first-child` An `E` element, first child of its parent.
//! * `E:last-child` An `E` element, last child of its parent.
//! * `E:first-of-type` An `E` element, first sibling of its type.
//! * `E:last-of-type` An `E` element, last sibling of its type.
//! * `E:only-child` An `E` element, only child of its parent.
//! * `E:only-of-type` An `E` element, only sibling of its type.
//! * `E:empty` An `E` element that has no children (including text nodes).
//! * `E:checked` A user interface element `E` which is checked (for instance a radio-button or checkbox).
//! * `E.warning` An `E` element whose class is `warning`.
//! * `E#myid` An `E` element with ID equal to `myid`.
//! * `E:not(s)` An `E` element that does not match simple selector `s`.
//! * `E F` An `F` element descendant of an `E` element.
//! * `E > F` An `F` element child of an `E` element.
//! * `E + F` An `F` element immediately preceded by an `E` element.
//! * `E ~ F` An `F` element preceded by an `E` element.
//! * `E, F, G` Elements of type `E`, `F` and `G`.
//! * `E[foo=bar][bar=baz]` An `E` element whose attributes match all following attribute selectors.

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
extern crate regex;
extern crate uuid;

pub use dom::DOM;

mod dom;
mod util;
