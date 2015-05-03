use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::BTreeMap;

use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

use regex::Regex;

use util::{xml_escape, xml_unescape};

lazy_static! {
    static ref ATTR_RE_STR: String = String::new() +
        r"([^<>=\s/]+|/)" +         // Key
        r"(?:" +
            r"\s*=\s*" +
            r"(?:"+
                r#""([^"]*?)""# +   // Quotation marks
            r"|" +
                r"'([^']*?)'" +     // Apostrophes
            r"|" +
                r"([^>\s]*)" +      // Unquoted
            r")" +
        r")?\s*";

    static ref TOKEN_RE_STR: String = String::new() +
        r"(?is)" +
        r"([^<]+)?" +                                               // Text
        r"(?:" +
            r"<(?:" +
                r"!(?:" +
                    r"DOCTYPE(?:\s+(.+?)\s*)" +                     // Doctype
                r"|" +
                    r"--(.*?)--\s*" +                               // Comment
                r"|" +
                    r"\[CDATA\[(.*?)\]\]" +                         // CDATA
                r")" +
            r"|" +
                r"\?(.*?)\?" +                                      // Processing Instruction
            r"|" +
                // it would be nice to write:
                // r"\s*([^<>\s]+\s*(?:(?:" + &_attr_re_str + r"){0,32766})*+)" +
                r"\s*([^<>\s]+\s*(?:" + &*ATTR_RE_STR + r")*)" +    // Tag
            r")>" +
        r"|" +
            r"(<)" +                                                // Runaway "<"
        r")?";

    // HTML elements that only contain raw text
    static ref RAW: HashSet<&'static str> = hashset!["script", "style"];

    // HTML elements that only contain raw text and entities
    static ref RCDATA: HashSet<&'static str> = hashset!["title", "textarea"];

    static ref END: HashMap<&'static str, &'static str> = {
        // HTML elements with optional end tags
        let mut _end = hashmap!["body" => "head", "optgroup" => "optgroup", "option" => "option"];

        // HTML elements that break paragraphs
        for x in vec![
            "address", "article", "aside", "blockquote", "dir", "div", "dl", "fieldset", "footer", "form",
            "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "main", "menu", "nav", "ol",
            "p", "pre", "section", "table", "ul"
        ] {
            _end.insert(x, "p");
        }

        _end
    };

    // HTML elements with optional end tags and scoping rules
    static ref CLOSE: HashMap<&'static str, (HashSet<&'static str>, HashSet<&'static str>)> = {
        // HTML table elements with optional end tags
        let _table = hashset!["colgroup", "tbody", "td", "tfoot", "th", "thead", "tr"];

        let _close = hashmap![
            "li" => (hashset!["li"], hashset!["ul", "ol"]),

            "colgroup" => (_table.clone(), hashset!["table"]),
            "tbody" => (_table.clone(), hashset!["table"]),
            "tfoot" => (_table.clone(), hashset!["table"]),
            "thead" => (_table.clone(), hashset!["table"]),

            "tr" => (hashset!["tr"], hashset!["table"]),
            "th" => (hashset!["th", "td"], hashset!["table"]),
            "td" => (hashset!["th", "td"], hashset!["table"]),

            "dd" => (hashset!["dd", "dt"], hashset!["dl"]),
            "dt" => (hashset!["dd", "dt"], hashset!["dl"]),

            "rp" => (hashset!["rp", "rt"], hashset!["ruby"]),
            "rt" => (hashset!["rp", "rt"], hashset!["ruby"])
        ];

        _close
    };

    // HTML elements without end tags
    static ref EMPTY: HashSet<&'static str> = hashset![
        "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link",
        "menuitem", "meta", "param", "source", "track", "wbr"
    ];

    // HTML elements categorized as phrasing content (and obsolete inline elements)
    static ref PHRASING: HashSet<&'static str> = hashset![
        "a", "abbr", "area", "audio", "b", "bdi", "bdo", "br", "button", "canvas", "cite", "code", "data",
        "datalist", "del", "dfn", "em", "embed", "i", "iframe", "img", "input", "ins", "kbd", "keygen",
        "label", "link", "map", "mark", "math", "meta", "meter", "noscript", "object", "output", "picture",
        "progress", "q", "ruby", "s", "samp", "script", "select", "small", "span", "strong", "sub", "sup",
        "svg", "template", "textarea", "time", "u", "var", "video", "wbr",
        "acronym", "applet", "basefont", "big", "font", "strike", "tt" // Obsolete
    ];

    // HTML elements that don't get their self-closing flag acknowledged
    static ref BLOCK: HashSet<&'static str> = hashset![
        "a", "address", "applet", "article", "aside", "b", "big", "blockquote", "body", "button",
        "caption", "center", "code", "col", "colgroup", "dd", "details", "dialog", "dir", "div",
        "dl", "dt", "em", "fieldset", "figcaption", "figure", "font", "footer", "form", "frameset",
        "h1", "h2", "h3", "h4", "h5", "h6", "head", "header", "hgroup", "html", "i", "iframe", "li",
        "listing", "main", "marquee", "menu", "nav", "nobr", "noembed", "noframes", "noscript",
        "object", "ol", "optgroup", "option", "p", "plaintext", "pre", "rp", "rt", "s", "script",
        "section", "select", "small", "strike", "strong", "style", "summary", "table", "tbody", "td",
        "template", "textarea", "tfoot", "th", "thead", "title", "tr", "tt", "u", "ul", "xmp"
    ];
}

#[derive(Debug)]
pub struct TreeNode {
    parent: Option<Weak<TreeNode>>,
    elem: NodeElem,
}

#[derive(Debug)]
pub enum NodeElem {
    Tag {
        name: String,
        attrs: Option<BTreeMap<String, Option<String>>>,
        childs: RefCell<Vec<Rc<TreeNode>>>,
    },

    Text {
        elem_type: String,
        content: String,
    },
}

impl TreeNode {
    fn get_tag_name(&self) -> Option<String> {
        match self.elem {
            NodeElem::Tag { ref name, .. } => Some(name.clone()),
            _ => None,
        }
    }
}

fn _process_text_node(current: &Rc<TreeNode>, elem_type: &String, content: &String) {
    let new_node = Rc::new(
        TreeNode {
            parent: Some(current.downgrade()),
            elem: NodeElem::Text { elem_type: elem_type.clone(), content: content.clone() }
        }
    );

    match current.elem {
        NodeElem::Tag { name: _, attrs: _, ref childs } => childs.borrow_mut().push(new_node),
        _ => panic!("Can use only `Tag` node as parent"),
    };
}

fn _process_start_tag(current: &Rc<TreeNode>, start_tag: &String, attrs: BTreeMap<String, Option<String>>) -> Rc<TreeNode> {
    let mut working_node = current.clone();

    // Autoclose optional HTML elements
    if working_node.parent.is_some() {
        if END.contains_key(start_tag.as_str()) {
            working_node = _process_end_tag(&working_node, &END.get(start_tag.as_str()).unwrap().to_string());
        }
        else if CLOSE.contains_key(start_tag.as_str()) {
            let ref allowed = CLOSE.get(start_tag.as_str()).unwrap().0;
            let ref scope = CLOSE.get(start_tag.as_str()).unwrap().1;

            // Close allowed parent elements in scope
            let mut next = working_node.clone();
            while next.parent.is_some() && !scope.contains(next.get_tag_name().unwrap().as_str()) {
                let this_tag_name = next.get_tag_name().unwrap();

                if allowed.contains(this_tag_name.as_str()) {
                    working_node = _process_end_tag(&working_node, &this_tag_name);
                }

                next = next.parent.clone().unwrap().upgrade().unwrap();
            }
        }
    }

    // New tag
    let new_node = Rc::new(
        TreeNode {
            parent: Some(working_node.downgrade()),
            elem: NodeElem::Tag { name: start_tag.clone(), attrs: Some(attrs), childs: RefCell::new(Vec::new()) },
        }
    );

    match working_node.elem {
        NodeElem::Tag { name: _, attrs: _, ref childs } => childs.borrow_mut().push(new_node.clone()),
        _ => panic!("Can use only `Tag` node as parent"),
    }

    new_node
}

fn _process_end_tag(current: &Rc<TreeNode>, end_tag: &String) -> Rc<TreeNode> {
    // Search stack for start tag
    let mut next = current.clone();
    while next.parent.is_some() {
        let this_tag_name = &next.get_tag_name().unwrap();

        // Right tag
        if this_tag_name == end_tag {
            return next;
        }

        // Phrasing content can only cross phrasing content
        if PHRASING.contains(end_tag.as_str()) && !PHRASING.contains(this_tag_name.as_str()) {
            return current.clone();
        }

        next = next.parent.clone().unwrap().upgrade().unwrap();
    }

    // Ignore useless end tag
    current.clone()
}

pub fn parse(html: &String) -> Rc<TreeNode> {
    let root = Rc::new(
        TreeNode {
            parent: None,
            elem: NodeElem::Tag { name: "root".to_string(), attrs: None, childs: RefCell::new(Vec::new()) },
        }
    );

    let mut current = root.clone();

    let re = Regex::new(&*TOKEN_RE_STR).unwrap();
    for caps in re.captures_iter(html) {
        let text = caps.at(1);
        let doctype = caps.at(2);
        let comment = caps.at(3);
        let cdata = caps.at(4);
        let pi = caps.at(5);
        let tag = caps.at(6);
        let runaway = caps.at(11);

        // Text (and runaway "<")
        if text.is_some() {
            let mut text_ok = text.unwrap().to_string();
            if runaway.is_some() {
                text_ok.push_str("<");
            };
            _process_text_node(&current, &"text".to_string(), &xml_unescape(&text_ok)); // TODO: html_unescape
        }

        // Tag
        if tag.is_some() {
            let tag_ok = tag.unwrap();

            // End: /tag
            if tag_ok.starts_with("/") {
                let tag_end = tag_ok.trim_left_matches('/').trim().to_lowercase();
                current = _process_end_tag(&current, &tag_end);
            }
            // Start: tag
            else {
                let tag_plus_attrs: Vec<&str> = tag_ok.splitn(2, ' ').collect();
                let mut start_tag = tag_plus_attrs.get(0).unwrap().to_string();
                let attrs_str = tag_plus_attrs.get(1);

                // Attributes
                let mut attrs: BTreeMap<String, Option<String>> = BTreeMap::new();
                let mut is_closing = false;
                if attrs_str.is_some() {
                    for caps in Regex::new(&*ATTR_RE_STR).unwrap().captures_iter(attrs_str.unwrap()) {
                        let key = caps.at(1).unwrap().to_string().to_lowercase();
                        let value = if caps.at(2).is_some() { caps.at(2) } else if caps.at(3).is_some() { caps.at(3) } else { caps.at(4) };

                        // Empty tag
                        if key == "/" {
                            is_closing = true;
                            continue;
                        }

                        attrs.insert(key, match value {
                            Some(ref x) => Some(xml_unescape(&x.to_string())), // TODO: html_unescape
                            None => None,
                        });
                    }
                }

                // "image" is an alias for "img"
                if start_tag == "image" { start_tag = "img".to_string() }

                current = _process_start_tag(&current, &start_tag, attrs);

                // Element without end tag (self-closing)
                if EMPTY.contains(start_tag.as_str()) || (!BLOCK.contains(start_tag.as_str()) && is_closing) {
                    current = _process_end_tag(&current, &start_tag);
                }

                // Raw text elements
                if !RAW.contains(start_tag.as_str()) && !RCDATA.contains(start_tag.as_str()) {
                    continue;
                }

                _process_text_node(&current, &"raw".to_string(),
                    &(if RCDATA.contains(start_tag.as_str()) { xml_unescape(&start_tag) } else { start_tag.clone() })
                );

                current = _process_end_tag(&current, &start_tag);
            }
        }

        // DOCTYPE
        else if doctype.is_some() {
            _process_text_node(&current, &"doctype".to_string(), &doctype.unwrap().to_string());
        }

        // Comment
        else if comment.is_some() {
            _process_text_node(&current, &"comment".to_string(), &comment.unwrap().to_string());
        }

        // CDATA
        else if cdata.is_some() {
            _process_text_node(&current, &"cdata".to_string(), &cdata.unwrap().to_string());
        }

        // Processing instruction (? try to detect XML)
        else if pi.is_some() {
            _process_text_node(&current, &"pi".to_string(), &pi.unwrap().to_string());
        }
    }

    root
}

pub fn render (root: &Rc<TreeNode>) -> String {

    match root.elem {
        // Text (escaped)
        NodeElem::Text { ref elem_type, ref content } if elem_type == "text" => {
            return xml_escape(content)
        },

        // Raw text
        NodeElem::Text { ref elem_type, ref content } if elem_type == "raw" => {
            return content.clone()
        },

        // DOCTYPE
        NodeElem::Text { ref elem_type, ref content } if elem_type == "doctype" => {
            return "<!DOCTYPE".to_string() + content + ">"
        },

        // Comment
        NodeElem::Text { ref elem_type, ref content } if elem_type == "comment" => {
            return "<!--".to_string() + content + "-->"
        },

        // CDATA
        NodeElem::Text { ref elem_type, ref content } if elem_type == "cdata" => {
            return "<![CDATA[".to_string() + content + "]]>"
        },

        // Processing instruction
        NodeElem::Text { ref elem_type, ref content } if elem_type == "pi" => {
            return "<?".to_string() + content + "?>"
        },

        // Root
        NodeElem::Tag { name: _, attrs: _, ref childs } if root.parent.is_none() => {
            return childs.borrow().iter().map(|ref x| { render(x) }).collect::<Vec<String>>().concat();
        },

        NodeElem::Tag { ref name, ref attrs, ref childs } => {
            let mut result = "<".to_string() + name;

            if attrs.is_some() {
                for (key, value) in attrs.clone().unwrap().iter() {
                    match *value {
                        Some(ref x) => { result = result + " " + key + "=\"" + &xml_escape(x) + "\"" },
                        None        => { result = result + " " + key },
                    }
                }
            }

            // No children
            if childs.borrow().is_empty() {
                return if EMPTY.contains(name.as_str()) { result + ">" } else { result + "></" + name + ">" };
            }

            // Children
            return
                result + ">" +
                &childs.borrow().iter().map(|ref x| { render(x) }).collect::<Vec<String>>().concat() +
                "</" + name + ">";
        },

        _ => { return "".to_string() },
    }
}
