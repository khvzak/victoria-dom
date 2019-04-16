use std::collections::{HashSet, HashMap, BTreeMap};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

use regex::{self, Regex};

use util::{xml_escape, html_unescape, html_attr_unescape};

lazy_static! {
    static ref ATTR_RE_STR: String = String::new() +
        r"([^<>=\s/]+|/)" +         // Key
        r"(?:" +
            r"\s*=\s*" +
            r"(?s:" +
                r#""(.*?)""# +      // Quotation marks
            r"|" +
                r"'(.*?)'" +        // Apostrophes
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
                    r"DOCTYPE(\s+\w+.*?)" +                         // Doctype
                r"|" +
                    r"--(.*?)--\s*" +                               // Comment
                r"|" +
                    r"\[CDATA\[(.*?)\]\]" +                         // CDATA
                r")" +
            r"|" +
                r"\?(.*?)\?" +                                      // Processing Instruction
            r"|" +
                r"\s*([^<>\s]+\s*(?:" + &*ATTR_RE_STR + r")*)" +    // Tag
            r")>" +
        r"|" +
            r"(<)" +                                                // Runaway "<"
        r")?" +
        r"(.*)$";                                                   // Rest of html

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
        "progress", "q", "ruby", "s", "samp", "script", "select", "slot", "small", "span", "strong", "sub", "sup",
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

static NODE_ID_NEXT: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct TreeNode {
    pub id: usize,
    pub parent: Option<Weak<TreeNode>>,
    pub elem: NodeElem,
}

#[derive(Debug)]
pub enum NodeElem {
    Root {
        childs: RefCell<Vec<Rc<TreeNode>>>,
    },

    Tag {
        name: String,
        attrs: BTreeMap<String, Option<String>>,
        childs: RefCell<Vec<Rc<TreeNode>>>,
    },

    Text {
        elem_type: String,
        content: String,
    },
}

impl TreeNode {
    pub fn is_tag(&self) -> bool {
        match self.elem {
            NodeElem::Tag { .. } => true,
            _ => false,
        }
    }

    pub fn get_tag_name(&self) -> Option<&str> {
        match self.elem {
            NodeElem::Tag { ref name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn get_tag_attrs<'a>(&'a self) -> Option<&'a BTreeMap<String, Option<String>>> {
        match self.elem {
            NodeElem::Tag { ref attrs, .. } => Some(attrs),
            _ => None,
        }
    }

    pub fn get_parent(&self) -> Option<Rc<TreeNode>> {
        match self.parent {
            Some(ref x) => Some(x.upgrade().unwrap()),  // strong reference should alive, force unwrap it
            _ => None,
        }
    }

    pub fn get_childs(&self) -> Option<Vec<Rc<TreeNode>>> {
        match self.elem {
            NodeElem::Root { ref childs } => Some(childs.borrow().clone()),
            NodeElem::Tag { ref childs, .. } => Some(childs.borrow().clone()),
            _ => None,
        }
    }

    // pub fn dbg_string(&self) -> String {
    //     let id = self.id;
    //     match self.elem {
    //         NodeElem::Root { .. } => format!("[{}] TreeNode:Root", id),
    //         NodeElem::Tag { ref name, ref attrs, .. } => format!("[{}] TreeNode:Tag(name: {}, attrs: {:?})", id, name, attrs),
    //         NodeElem::Text { ref elem_type, ref content } => format!("[{}] TreeNode:Text(type: {}, content: {})", id, elem_type, content),
    //     }
    // }
}

fn _process_text_node(current: &Rc<TreeNode>, elem_type: &str, content: &str) {
    let new_node = Rc::new(
        TreeNode {
            id: NODE_ID_NEXT.fetch_add(1, Ordering::Relaxed),
            parent: Some(Rc::downgrade(current)),
            elem: NodeElem::Text { elem_type: elem_type.to_owned(), content: content.to_owned() },
        }
    );

    match current.elem {
        NodeElem::Root { ref childs } => childs.borrow_mut().push(new_node),
        NodeElem::Tag { ref childs, .. } => childs.borrow_mut().push(new_node),
        NodeElem::Text { .. } => panic!("Cannot use `Text` node as parent"),
    };
}

fn _process_start_tag(current: &Rc<TreeNode>, start_tag: &str, attrs: BTreeMap<String, Option<String>>) -> Rc<TreeNode> {
    let mut working_node = current.clone();

    // Autoclose optional HTML elements
    if working_node.parent.is_some() {
        if let Some(end_tag) = END.get(start_tag) {
            working_node = _process_end_tag(&working_node, end_tag);
        }
        else if let Some(x) = CLOSE.get(start_tag) {
            let (ref allowed, ref scope) = *x;

            // Close allowed parent elements in scope
            let mut next = working_node.clone();
            while next.parent.is_some() && !scope.contains(next.clone().get_tag_name().unwrap()) {
                let this = next.clone();
                let this_tag_name = this.get_tag_name().unwrap();

                if allowed.contains(this_tag_name) {
                    working_node = _process_end_tag(&working_node, this_tag_name);
                }

                next = next.get_parent().unwrap();
            }
        }
    }

    // New tag
    let new_node = Rc::new(
        TreeNode {
            id: NODE_ID_NEXT.fetch_add(1, Ordering::Relaxed),
            parent: Some(Rc::downgrade(&working_node)),
            elem: NodeElem::Tag { name: start_tag.to_owned(), attrs: attrs, childs: RefCell::new(Vec::new()) },
        }
    );

    match working_node.elem {
        NodeElem::Root { ref childs } => childs.borrow_mut().push(new_node.clone()),
        NodeElem::Tag { ref childs, .. } => childs.borrow_mut().push(new_node.clone()),
        NodeElem::Text { .. } => panic!("Cannot use `Text` node as parent"),
    }

    new_node
}

fn _process_end_tag(current: &Rc<TreeNode>, end_tag: &str) -> Rc<TreeNode> {
    // Search stack for start tag
    let mut next = current.clone();
    while next.parent.is_some() {
        let this = next.clone();
        let this_tag_name = this.get_tag_name().unwrap();

        // Right tag
        if this_tag_name == end_tag {
            return next.get_parent().unwrap();
        }

        // Phrasing content can only cross phrasing content
        if PHRASING.contains(end_tag) && !PHRASING.contains(this_tag_name) {
            return current.clone();
        }

        next = next.get_parent().unwrap();
    }

    // Ignore useless end tag
    current.clone()
}

pub fn parse(html: &str) -> Rc<TreeNode> {
    let mut html = html;

    let root = Rc::new(
        TreeNode {
            id: NODE_ID_NEXT.fetch_add(1, Ordering::Relaxed),
            parent: None,
            elem: NodeElem::Root { childs: RefCell::new(Vec::new()) },
        }
    );

    let mut current = root.clone();

    lazy_static! {
        static ref _TAG_PLUS_ATTRS_RE: Regex = Regex::new(r"^([^\s/]+)([\s\S]*)").unwrap();
    }

    let re = Regex::new(&*TOKEN_RE_STR).unwrap();
    while let Some(caps) = re.captures(html) {
        let text = caps.get(1);
        let doctype = caps.get(2);
        let comment = caps.get(3);
        let cdata = caps.get(4);
        let pi = caps.get(5);
        let tag = caps.get(6);
        let runaway = caps.get(11);

        html = caps.get(12).map(|c| c.as_str()).unwrap_or(""); // html rest

        // Text (and runaway "<")
        if let Some(text) = text {
            if runaway.is_some() {
                _process_text_node(&current, "text", &html_unescape(&(text.as_str().to_owned() + "<")));
            } else {
                _process_text_node(&current, "text", &html_unescape(text.as_str()));
            }
        }

        // Tag
        if let Some(tag) = tag {
            // End: /tag
            if tag.as_str().starts_with("/") {
                let end_tag = tag.as_str().trim_start_matches('/').trim().to_lowercase();
                current = _process_end_tag(&current, &end_tag);
            }
            // Start: tag
            else {
                let caps = _TAG_PLUS_ATTRS_RE.captures(tag.as_str()).unwrap(); // panic is ok
                let mut start_tag = caps.get(1).unwrap().as_str().to_lowercase();
                let attrs_str = caps.get(2).unwrap();

                // Attributes
                let mut attrs: BTreeMap<String, Option<String>> = BTreeMap::new();
                let mut is_closing = false;
                for caps in Regex::new(&*ATTR_RE_STR).unwrap().captures_iter(attrs_str.as_str()) {
                    let key = caps.get(1).unwrap().as_str().to_owned().to_lowercase();
                    let value = if caps.get(2).is_some() { caps.get(2) } else if caps.get(3).is_some() { caps.get(3) } else { caps.get(4) };

                    // Empty tag
                    if key == "/" {
                        is_closing = true;
                        continue;
                    }

                    attrs.insert(key, match value {
                        Some(ref x) => Some(html_attr_unescape(x.as_str())),
                        _ => None,
                    });
                }

                // "image" is an alias for "img"
                if start_tag == "image" { start_tag = "img".to_owned() }

                current = _process_start_tag(&current, &start_tag, attrs);

                // Element without end tag (self-closing)
                if EMPTY.contains(start_tag.as_str()) || (!BLOCK.contains(start_tag.as_str()) && is_closing) {
                    current = _process_end_tag(&current, &start_tag);
                }

                // Raw text elements
                if RAW.contains(start_tag.as_str()) || RCDATA.contains(start_tag.as_str()) {
                    let raw_text_re = Regex::new(&(r"(.+?)<\s*/\s*".to_owned() + &regex::escape(&start_tag) + r"\s*>(.*)$")).unwrap();
                    if let Some(raw_text_caps) = raw_text_re.captures(html) {
                        let raw_text = raw_text_caps.get(1).unwrap();
                        html = raw_text_caps.get(2).map(|c| c.as_str()).unwrap_or("");

                        if RCDATA.contains(&start_tag.as_str()) {
                            _process_text_node(&current, "raw", &html_unescape(raw_text.as_str()))
                        } else {
                            _process_text_node(&current, "raw", raw_text.as_str())
                        }

                        current = _process_end_tag(&current, &start_tag);
                    }
                }
            }
        }

        // DOCTYPE
        else if let Some(doctype) = doctype {
            _process_text_node(&current, "doctype", doctype.as_str());
        }

        // Comment
        else if let Some(comment) = comment {
            _process_text_node(&current, "comment", comment.as_str());
        }

        // CDATA
        else if let Some(cdata) = cdata {
            _process_text_node(&current, "cdata", cdata.as_str());
        }

        // Processing instruction
        else if let Some(pi) = pi {
            _process_text_node(&current, "pi", pi.as_str());
        }

        if html.is_empty() { break; }
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
            return "<!DOCTYPE".to_owned() + content + ">"
        },

        // Comment
        NodeElem::Text { ref elem_type, ref content } if elem_type == "comment" => {
            return "<!--".to_owned() + content + "-->"
        },

        // CDATA
        NodeElem::Text { ref elem_type, ref content } if elem_type == "cdata" => {
            return "<![CDATA[".to_owned() + content + "]]>"
        },

        // Processing instruction
        NodeElem::Text { ref elem_type, ref content } if elem_type == "pi" => {
            return "<?".to_owned() + content + "?>"
        },

        // Root
        NodeElem::Root { ref childs } => {
            return childs.borrow().iter().map(|ref x| { render(x) }).collect::<Vec<String>>().concat();
        },

        NodeElem::Tag { ref name, ref attrs, ref childs } => {
            let mut result = "<".to_owned() + name;

            // Attributes
            for (key, value) in attrs.iter() {
                match *value {
                    Some(ref x) => { result = result + " " + key + "=\"" + &xml_escape(x) + "\"" },
                    None        => { result = result + " " + key },
                }
            }

            // No children
            if childs.borrow().is_empty() {
                return if EMPTY.contains(&name[..]) { result + ">" } else { result + "></" + name + ">" };
            }

            // Children
            return
                result + ">" +
                &childs.borrow().iter().map(|ref x| { render(x) }).collect::<Vec<String>>().concat() +
                "</" + name + ">";
        },

        _ => { return "".to_owned() },
    }
}
