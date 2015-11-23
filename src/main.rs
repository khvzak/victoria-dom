extern crate dom_parser;
//extern crate regex;

use dom_parser::dom::html;
//use dom_parser::dom::css;

//use std::char;
//use regex::Regex;

fn main() {
    println!("{}", "hello");

    let html = r#"<div><p id="a">Test</p><p id="b">123</p></div>"#;
    let tree = dom_parser::dom::html::parse(html);

    //let node_selected = dom_parser::dom::css::select_one(&tree, "div > p[id=a]").unwrap();

    //println!("{:?}", node_selected);
    //println!("{}", dom_parser::dom::html::render(&node_selected));

    /*
    let html = r#"<div id = test foo ="bar" class=tset bar=/baz/ baz=//>works<p>ok!</p></div>"#.to_string();
    println!("{}", html);

    let tree = parse(&html);
    let html_rendered = render(&tree);
    println!("{}", html_rendered);
    */

    //let x = dom_parser::dom::css::_parse("body > div > div[foo=bar]");
    //println!("{:?}", x);

    //println!("{:?}", _test());
    //println!("{:?}", _test());
}
