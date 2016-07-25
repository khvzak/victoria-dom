mod css;
mod html;

use std::collections::BTreeMap;
use std::rc::Rc;

use regex::Regex;

use self::html::TreeNode;

/// The HTML `DOM` type
pub struct DOM {
    tree: Rc<TreeNode>,
}

impl DOM {
    /// Construct a new `DOM` object and parse HTML.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div id=\"title\">Hello</div>");
    /// ```
    pub fn new(html: &str) -> DOM {
        DOM { tree: html::parse(html) }
    }

    /// Find all ancestor elements of the current element matching the optional CSS selector
    /// and return a Vector of DOM objects of these elements.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<html><body><div id=\"title\">Hello</div></body></html>");
    /// let ancestors: Vec<_> = dom.at("div").unwrap().ancestors(None).iter().map(|x| x.tag().unwrap().to_string()).collect();
    /// assert_eq!(ancestors, ["body", "html"]);
    /// ```
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

    /// Find first descendant element of the current element matching the CSS selector and return it as a DOM object,
    /// or `None` if none could be found.
    pub fn at(&self, selector: &str) -> Option<DOM> {
        if let Some(node) = css::select_one(&self.tree, selector) {
            return Some(DOM { tree: node })
        }
        None
    }

    /// The current element tag name.
    pub fn tag(&self) -> Option<&str> {
        self.tree.get_tag_name()
    }

    /// The current element attribute2value map.
    pub fn attrs(&self) -> BTreeMap<String, Option<String>> {
        self.tree.get_tag_attrs().map_or_else(|| BTreeMap::new(), |x| x.clone())
    }

    /// The current element attribute value, or `None` if there are no attribute with the name or value.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.tree.get_tag_attrs().and_then(|x| x.get(name)).and_then(|x| x.as_ref()).map(|x| x.as_str())
    }

    /// Find all child elements of the current element matching the CSS selector and return a Vector of DOM objects of these elements.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div><div id=\"a\">A <span id=\"c\">C</span></div><div id=\"b\">B</div></div>");
    /// let childs: Vec<_> = dom.at("div").unwrap().childs(None).iter().map(|x| x.attr("id").unwrap().to_string()).collect();
    /// assert_eq!(childs, ["a", "b"]);
    /// ```
    pub fn childs(&self, selector: Option<&str>) -> Vec<DOM> {
        self.tree.get_childs().unwrap_or(Vec::new()).into_iter().filter_map(|x|
            if x.is_tag() && (selector.is_none() || css::matches(&x, selector.unwrap())) {
                Some(DOM { tree: x })
            } else {
                None
            }
        ).collect()
    }

    /// Find all descendant elements of the current element matching the CSS selector and return a Vector of DOM objects of these elements.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div><div id=\"a\"><div id=\"c\">C</div></div><div id=\"b\">B</div></div>");
    /// let elems: Vec<_> = dom.find("div[id]").iter().map(|x| x.attr("id").unwrap().to_string()).collect();
    /// assert_eq!(elems, ["a", "c", "b"]);
    /// ```
    pub fn find(&self, selector: &str) -> Vec<DOM> {
        css::select(&self.tree, selector, 0).into_iter().map(|x| DOM { tree: x }).collect()
    }

    /// Check if the current element matches the CSS selector.
    pub fn matches(&self, selector: &str) -> bool {
        css::matches(&self.tree, selector)
    }

    /// Find all sibling elements after the current element matching the CSS selector and return a Vector of DOM objects of these elements.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div><div id=\"a\"><div id=\"c\">C</div></div><div id=\"b\">B</div></div>");
    /// let elems: Vec<_> = dom.at("div#a").unwrap().following(None).iter().map(|x| x.attr("id").unwrap().to_string()).collect();
    /// assert_eq!(elems, ["b"]);
    /// ```
    pub fn following(&self, selector: Option<&str>) -> Vec<DOM> {
        self._siblings().into_iter().skip_while(|x| x.id != self.tree.id).skip(1)
            .filter(|x| selector.is_none() || css::matches(x, selector.unwrap()))
            .map(|x| DOM { tree: x }).collect()
    }

    /// Return a DOM object for next sibling element, or `None` if there are no more siblings.
    pub fn next(&self) -> Option<DOM> {
        self._siblings().into_iter().skip_while(|x| x.id != self.tree.id).skip(1).next().map(|x| DOM { tree: x })
    }

    /// Find all sibling elements before the current element matching the CSS selector and return a Vector of DOM objects of these elements.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div><div id=\"a\"><div id=\"c\">C</div></div><div id=\"b\">B</div></div>");
    /// let elems: Vec<_> = dom.at("div#b").unwrap().preceding(None).iter().map(|x| x.attr("id").unwrap().to_string()).collect();
    /// assert_eq!(elems, ["a"]);
    /// ```
    pub fn preceding(&self, selector: Option<&str>) -> Vec<DOM> {
        self._siblings().into_iter().take_while(|x| x.id != self.tree.id)
            .filter(|x| selector.is_none() || css::matches(x, selector.unwrap()))
            .map(|x| DOM { tree: x }).collect()
    }

    /// Return a DOM object for the previous sibling element, or `None` if there are no more siblings.
    pub fn prev(&self) -> Option<DOM> {
        self._siblings().into_iter().take_while(|x| x.id != self.tree.id).last().map(|x| DOM { tree: x })
    }

    fn _siblings(&self) -> Vec<Rc<TreeNode>> {
        self.tree.get_parent()
            .and_then(|x| x.get_childs())
            .map(|x| x.into_iter().filter(|v| v.is_tag()).collect::<Vec<_>>())
            .unwrap_or(Vec::new())
    }

    /// Return a DOM object for the parent of the current element, or `None` if this element has no parent.
    pub fn parent(&self) -> Option<DOM> {
        self.tree.get_parent().map(|x| DOM { tree: x })
    }

    /// Render the current element and its content to HTML.
    pub fn to_string(&self) -> String {
        html::render(&self.tree)
    }

    /// Extract text content from the current element only (not including child elements) with smart whitespace trimming.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div>foo\n<p>bar</p>baz\n</div>");
    /// assert_eq!(dom.at("div").unwrap().text(), "foo baz");
    /// ```
    pub fn text(&self) -> String {
        self._text(false, true) // non-recursive trimmed
    }

    /// Extract text content from the current element only (not including child elements) without smart whitespace trimming.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div>foo\n<p>bar</p>baz\n</div>");
    /// assert_eq!(dom.at("div").unwrap().rtext(), "foo\nbaz\n");
    /// ```
    pub fn rtext(&self) -> String {
        self._text(false, false) // non-recursive raw
    }

    /// Extract text content from all descendant nodes of the current element with smart whitespace trimming.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div>foo\n<p>bar</p>baz\n</div>");
    /// assert_eq!(dom.at("div").unwrap().text_all(), "foo bar baz");
    /// ```
    pub fn text_all(&self) -> String {
        self._text(true, true) // recursive trimmed
    }

    /// Extract text content from all descendant nodes of the current element with smart whitespace trimming.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div>foo\n<p>bar</p>baz\n</div>");
    /// assert_eq!(dom.at("div").unwrap().rtext_all(), "foo\nbarbaz\n");
    /// ```
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

    /// Return content of the current element.
    ///
    /// ```
    /// use victoria_dom::DOM;
    /// let dom = DOM::new("<div><b>Test</b></div>");
    /// assert_eq!(dom.at("div").unwrap().content(), "<b>Test</b>");
    /// ```
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
        if trim && _RE2.is_match(&text) && _RE3.is_match(&chunk) {
            chunk = " ".to_owned() + &chunk
        }

        // Trim whitespace blocks
        if _RE4.is_match(&chunk) || !trim {
            text.push_str(&chunk);
        }
    }
    text
}
