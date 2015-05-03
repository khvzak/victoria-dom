extern crate dom_parser;

use dom_parser::dom::html::{parse, render};

fn main() {
    println!("{}", "hello");

    let tree = parse(&"<div><div FOO=\"0\" id=\"a\">A</div><div id=\"b\">B</div></div>".to_string());
    let text = render(&tree);
    println!("{:?}", tree);
    println!("{}", text);
}
