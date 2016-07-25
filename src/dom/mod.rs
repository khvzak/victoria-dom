mod css;
mod html;

use std::collections::BTreeMap;
use std::rc::Rc;

use regex::Regex;

use self::html::TreeNode;

pub struct DOM {
    tree: Rc<TreeNode>,
}

impl DOM {
    pub fn new(html: &str) -> DOM {
        DOM { tree: html::parse(html) }
    }

    pub fn ancestors(&self, selector: Option<&str>) -> Vec<DOM> {
        let mut ancestors = Vec::new();
        let mut node = self.tree.clone();
        while let Some(parent) = node.get_parent() {
            if parent.is_tag() && (selector.is_none() || css::matches(&parent, selector.unwrap())) {
                ancestors.push(DOM { tree: parent.clone() });
            }
            node = parent;
        }
        ancestors
    }

    pub fn at(&self, selector: &str) -> Option<DOM> {
        if let Some(node) = css::select_one(&self.tree, selector) {
            return Some(DOM { tree: node })
        }
        None
    }

    pub fn tag(&self) -> Option<&str> {
        self.tree.get_tag_name()
    }

    pub fn attrs(&self) -> BTreeMap<String, Option<String>> {
        self.tree.get_tag_attrs().map_or_else(|| BTreeMap::new(), |x| x.clone())
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.tree.get_tag_attrs().and_then(|x| x.get(name)).and_then(|x| x.as_ref()).map(|x| x.as_str())
    }

    pub fn childs(&self, selector: Option<&str>) -> Vec<DOM> {
        self.tree.get_childs().unwrap_or(Vec::new()).into_iter().filter_map(|x|
            if x.is_tag() && (selector.is_none() || css::matches(&x, selector.unwrap())) {
                Some(DOM { tree: x })
            } else {
                None
            }
        ).collect()
    }

    pub fn find(&self, selector: &str) -> Vec<DOM> {
        css::select(&self.tree, selector, 0).into_iter().map(|x| DOM { tree: x }).collect()
    }

    pub fn matches(&self, selector: &str) -> bool {
        css::matches(&self.tree, selector)
    }

    pub fn following(&self, selector: Option<&str>) -> Vec<DOM> {
        self._siblings().into_iter().skip_while(|x| x.id != self.tree.id).skip(1)
            .filter(|x| selector.is_none() || css::matches(x, selector.unwrap()))
            .map(|x| DOM { tree: x }).collect()
    }

    pub fn next(&self) -> Option<DOM> {
        self._siblings().into_iter().skip_while(|x| x.id != self.tree.id).skip(1).next().map(|x| DOM { tree: x })
    }

    pub fn preceding(&self, selector: Option<&str>) -> Vec<DOM> {
        self._siblings().into_iter().take_while(|x| x.id != self.tree.id)
            .filter(|x| selector.is_none() || css::matches(x, selector.unwrap()))
            .map(|x| DOM { tree: x }).collect()
    }

    pub fn prev(&self) -> Option<DOM> {
        self._siblings().into_iter().take_while(|x| x.id != self.tree.id).last().map(|x| DOM { tree: x })
    }

    fn _siblings(&self) -> Vec<Rc<TreeNode>> {
        self.tree.get_parent()
            .and_then(|x| x.get_childs())
            .map(|x| x.into_iter().filter(|v| v.is_tag()).collect::<Vec<_>>())
            .unwrap_or(Vec::new())
    }

    pub fn parent(&self) -> Option<DOM> {
        self.tree.get_parent().map(|x| DOM { tree: x })
    }

    pub fn to_string(&self) -> String {
        html::render(&self.tree)
    }

    pub fn text(&self) -> String {
        self._text(false, true) // non-recursive trimmed
    }

    pub fn rtext(&self) -> String {
        self._text(false, false) // non-recursive raw
    }

    pub fn text_all(&self) -> String {
        self._text(true, true) // recursive trimmed
    }

    pub fn rtext_all(&self) -> String {
        self._text(true, false) // recursive raw
    }

    fn _text(&self, recursive: bool, trim: bool) -> String {
        // Try to detect "pre" tag
        let mut under_pre_tag = false;
        if trim {
            let mut node = self.tree.clone();
            loop {
                if let html::NodeElem::Tag { ref name, .. } = node.elem {
                    if name == "pre" {
                        under_pre_tag = true;
                        break;
                    }
                }
                if node.get_parent().is_some() { node = node.get_parent().unwrap(); } else { break; }
            }
        }

        match self.tree.get_childs() {
            Some(nodes) => _nodes_text(&nodes, recursive, trim && !under_pre_tag),
            _ => String::new(),
        }
    }

    pub fn content(&self) -> String {
        self.tree.get_childs().unwrap().into_iter().map(|x| html::render(&x)).collect::<Vec<_>>().join("")
    }
}

fn _nodes_text(nodes: &Vec<Rc<TreeNode>>, recursive: bool, trim: bool) -> String {
    lazy_static! {
        static ref _RE1: Regex = Regex::new(r"\s+").unwrap();
        static ref _RE2: Regex = Regex::new(r"\S\z").unwrap();
        static ref _RE3: Regex = Regex::new(r"^[^.!?,;:\s]+").unwrap();
        static ref _RE4: Regex = Regex::new(r"\S+").unwrap();
    }

    let mut text = String::new();
    for node in nodes {
        let mut chunk = match node.elem {
            html::NodeElem::Text { ref elem_type, ref content } => {
                match elem_type.as_ref() {
                    "text" if trim => _RE1.replace_all(content.trim(), " "),
                    "text" | "raw" | "cdata" => content.to_owned(),
                    _ => String::new(),
                }
            },
            html::NodeElem::Tag { ref name, ref childs, .. } if recursive => {
                _nodes_text(&childs.borrow(), true, trim && name != "pre")
            }
            _ => String::new(),
        };

        // Add leading whitespace if punctuation allows it
        if _RE2.is_match(&text) && _RE3.is_match(&chunk) {
            chunk = " ".to_owned() + &chunk
        }

        // Trim whitespace blocks
        if _RE4.is_match(&chunk) || !trim {
            text.push_str(&chunk);
        }
    }
    text
}
