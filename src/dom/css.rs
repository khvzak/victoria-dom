use std::rc::Rc;

use std::char;

use regex;
use regex::{Regex, Captures};

use dom::html::{TreeNode, NodeElem};

lazy_static! {
    static ref ESCAPE_RE_STR: String = r"\\[^0-9a-fA-F]|\\[0-9a-fA-F]{1,6}".to_owned();

    static ref ATTR_RE_STR: String = String::new() +
        r"\[" +
        r"((?:" + &*ESCAPE_RE_STR + r"|[\w-])+)" +                     // Key
        r"(?:" +
            r"(\W)?=" +                                                 // Operator
            r#"(?:"((?:\\"|[^"])*)"|'((?:\\'|[^'])*)'|([^\]]+?))"# +    // Value
            r"(?:\s+(i))?" +                                            // Case-sensitivity
        r")?" +
        r"\]";
}

#[derive(Debug)]
pub enum SelectorItem {
    Combinator { op: String },
    Conditions { items: Vec<ConditionItem> },
}

#[derive(Debug)]
pub enum ConditionItem {
    Tag { name: Regex },
    Attribute { name: Regex, value: Option<Regex> },
    PseudoClass {
        class: String,
        group: Option<GroupOfSelectors>,
        equation: Option<(i32, i32)>,
    },
}

pub type Selectors = Vec<Rc<SelectorItem>>;
pub type GroupOfSelectors = Vec<Rc<Selectors>>;

pub fn matches(tree: &Rc<TreeNode>, css: &str) -> bool {
    if css.is_empty() { return true; }
    match tree.elem {
        NodeElem::Tag { .. } => _match(&parse(css), tree, tree),
        _ => false
    }
}

pub fn select(tree: &Rc<TreeNode>, css: &str, limit: usize) -> Vec<Rc<TreeNode>> {
    let group = parse(css);

    let mut result = Vec::new();

    let mut queue = tree.get_childs().unwrap();
    while queue.len() > 0 {
        let current = queue.remove(0);
        if let NodeElem::Tag { .. } = current.elem {} else { continue; }

        queue = { let mut x = current.get_childs().unwrap(); x.append(&mut queue); x };
        if (group.is_empty() && css == "*") || _match(&group, &current, tree) { result.push(current); }

        if limit > 0 && result.len() == limit { break; }
    }

    result
}

pub fn select_one(tree: &Rc<TreeNode>, css: &str) -> Option<Rc<TreeNode>> {
    select(tree, css, 1).pop()
}

fn _match(group: &GroupOfSelectors, current: &Rc<TreeNode>, tree: &Rc<TreeNode>) -> bool {
    for _selectors in group {
        let selectors = Rc::new(_selectors.iter().rev().cloned().collect::<Selectors>());
        if _combinator(&selectors, current, tree, 0) { return true; }
    }
    return false;
}

fn _combinator(selectors: &Rc<Selectors>, current: &Rc<TreeNode>, tree: &Rc<TreeNode>, mut idx: usize) -> bool {
    if idx >= selectors.len() { return false; }

    match *selectors[idx] {
        SelectorItem::Conditions { ref items } => {
            if !_match_selector_conditions(items, current) { return false; }

            idx = idx + 1;
            if idx >= selectors.len() { return true; }
            return _combinator(selectors, current, tree, idx);
        },

        SelectorItem::Combinator { ref op } => {
            idx = idx + 1;

            // ">" (parent only)
            if op == ">" {
                if current.parent.is_none() { return false; }
                let parent = current.get_parent().unwrap();

                // no suitable parent
                if let NodeElem::Root { .. } = parent.elem { return false; }
                if parent.id == tree.id { return false; }

                return _combinator(selectors, &parent, tree, idx);
            }

            // "~" (preceding siblings)
            if op == "~" {
                for sibling in _siblings(current, None) {
                    if sibling.id == current.id { return false; }
                    if _combinator(selectors, &sibling, tree, idx) { return true; }
                }
                return false;
            }

            // "+" (immediately preceding siblings)
            if op == "+" {
                let mut found = false;
                for sibling in _siblings(current, None) {
                    if sibling.id == current.id { return found; }
                    found = _combinator(selectors, &sibling, tree, idx);
                }
                return false;
            }

            // " " (ancestor)
            let mut parent = current.get_parent();
            while parent.is_some() {
                let current_next = parent.clone().unwrap();

                if let NodeElem::Root { .. } = current_next.elem { return false; }
                if current_next.id == tree.id { return false; }

                if _combinator(selectors, &current_next, tree, idx) { return true; }

                parent = current_next.get_parent();
            }
            return false;
        },
    }
}

fn _match_selector_conditions(conditions: &Vec<ConditionItem>, current: &Rc<TreeNode>) -> bool {
    'conditem: for ci in conditions {
        match ci {
            &ConditionItem::Tag { name: ref name_re } => {
                if !name_re.is_match(current.get_tag_name().unwrap()) { return false; }
            },

            &ConditionItem::Attribute { name: ref name_re, value: ref value_re } => {
                let attrs = current.get_tag_attrs().unwrap();
                let value_re = value_re.as_ref();

                for (name, value) in attrs.iter() {
                    let value = value.as_ref();

                    if name_re.is_match(name) && (value.is_none() || value_re.is_none() || value_re.unwrap().is_match(value.unwrap())) {
                        continue 'conditem; // go to a next condition item
                    }
                }
                return false;
            },

            &ConditionItem::PseudoClass { ref class, ref group, ref equation } => {
                // ":empty"
                if class == "empty" {
                    let _is_empty = |x: &TreeNode| match x.elem {
                        NodeElem::Text { ref elem_type, .. } => elem_type == "comment" || elem_type == "pi",
                        _ => false,
                    };

                    let _matched = current.get_childs().unwrap().iter().filter(|&x| !_is_empty(x)).count() == 0;
                    if _matched { continue 'conditem; }
                }

                // ":root"
                else if class == "root" {
                    let parent = current.get_parent();
                    let _matched = parent.is_some() && match parent.unwrap().elem {
                        NodeElem::Root { .. } => true,
                        _ => false
                    };
                    if _matched { continue 'conditem; }
                }

                // ":not"
                else if class == "not" {
                    let _matched = !_match(&group.clone().unwrap(), current, current);
                    if _matched { continue 'conditem; }
                }

                // ":checked"
                else if class == "checked" {
                    let _matched = match current.elem {
                        NodeElem::Tag { ref attrs, .. } => attrs.contains_key("checked") || attrs.contains_key("selected"),
                        _ => false
                    };
                    if _matched { continue 'conditem; }
                }

                // ":nth-child", ":nth-last-child", ":nth-of-type" or ":nth-last-of-type"
                else if let Some(equation) = *equation {
                    let mut siblings = if class.ends_with("of-type") {
                        _siblings(current, Some(current.get_tag_name().unwrap()))
                    } else {
                        _siblings(current, None)
                    };

                    if class.starts_with("nth-last") { siblings.reverse() }

                    for i in 0..siblings.len() {
                        let result = equation.0 * (i as i32) + equation.1;

                        if result < 1 { continue; }
                        if (result - 1) as usize >= siblings.len() { break; }

                        if siblings[(result - 1) as usize].id == current.id { continue 'conditem; }
                    }
                }

                // ":only-child" or ":only-of-type"
                else if class == "only-child" || class == "only-of-type" {
                    let siblings = if class == "only-of-type" {
                        _siblings(current, Some(current.get_tag_name().unwrap()))
                    } else {
                        _siblings(current, None)
                    };
                    for sibling in siblings {
                        if sibling.id != current.id { return false; }
                    }

                    continue 'conditem;
                }

                return false;
            },
        }
    }

    return true;
}

fn _siblings(current: &Rc<TreeNode>, _name: Option<&str>) -> Vec<Rc<TreeNode>> {
    let parent = current.get_parent().unwrap();
    let childs = parent.get_childs().unwrap();

    childs.iter().filter(|&x| match x.elem {
        NodeElem::Tag { ref name, .. } => if _name.is_some() { name == _name.unwrap() } else { true },
        _ => false
    }).cloned().collect()
}

fn _unescape(_val: &str) -> String {
    let mut val = _val.to_owned();

    lazy_static! {
        static ref _RE: Regex = Regex::new(r"\\([0-9a-fA-F]{1,6})\s?").unwrap();
    }

    // Remove escaped newlines
    val = val.replace("\\\n", "");

    // Unescape Unicode characters
    val = _RE.replace_all(&val, |caps: &Captures| {
        let hex_char = caps.at(1).unwrap();
        format!("{}", char::from_u32(u32::from_str_radix(hex_char, 16).unwrap()).unwrap())
    });

    // Remove backslash
    val = val.replace(r"\", "");

    val
}

fn _name_re(_val: &str) -> Regex {
    Regex::new(&(r"(?:^|:)".to_owned() + &regex::quote(&_unescape(_val)) + "$")).unwrap()
}

fn _value_re(op: &str, _val: Option<&str>, insensitive: bool) -> Option<Regex> {
    if _val.is_none() { return None };
    let mut value = regex::quote(&_unescape(_val.unwrap()));

    if insensitive {
        value = "(?i)".to_owned() + &value;
    }

    Some(Regex::new(&(
        // "~=" (word)
        if op == "~" {
            r"(?:^|\s+)".to_owned() + &value + r"(?:\s+|$)"
        }

        // "*=" (contains)
        else if op == "*" {
            value
        }

        // "^=" (begins with)
        else if op == "^" {
            r"^".to_owned() + &value
        }

        // "$=" (ends with)
        else if op == "$" {
            value + r"$"
        }

        // Everything else
        else {
            r"^".to_owned() + &value + "$"
        }
    )).unwrap())
}

pub fn parse(css: &str) -> GroupOfSelectors {
    let mut css = css.trim();

    // Group separator re
    lazy_static! {
        static ref _SEPARATOR_RE: Regex = Regex::new(r"^(?s)\s*,\s*(.*)$").unwrap();
    }

    let mut group: GroupOfSelectors = Vec::new();
    loop {
        let (selectors, css_rest) = _parse_selectors(css);
        if !selectors.is_empty() {
            group.push(Rc::new(selectors));
            css = css_rest;
        } else {
            break;
        }

        // Separator
        if let Some(caps) = _SEPARATOR_RE.captures(css) {
            css = caps.at(1).unwrap();
        } else {
            break;
        }
    }

    group
}

fn _parse_selectors(css: &str) -> (Selectors, &str) {
    let mut css = css;

    // Selector combinator re
    lazy_static! {
        static ref _COMBINATOR_RE: Regex = Regex::new(r"^(?s)\s*([ >+~])\s*(.*)$").unwrap();
    }

    let mut selectors: Selectors = Vec::new();
    loop {
        let (conditions, css_rest) = _parse_selector_conditions(css);
        if !conditions.is_empty() {
            selectors.push(Rc::new(SelectorItem::Conditions { items: conditions }));
            css = css_rest;
        } else {
            break;
        }

        // Combinator
        if let Some(caps) = _COMBINATOR_RE.captures(css) {
            selectors.push(Rc::new(SelectorItem::Combinator { op: caps.at(1).unwrap().to_owned() }));
            css = caps.at(2).unwrap();
        } else {
            break;
        }
    }

    return (selectors, css);
}

fn _parse_selector_conditions(css: &str) -> (Vec<ConditionItem>, &str) {
    let mut css = css;

    lazy_static! {
        static ref _CLASS_OR_ID_RE: Regex = Regex::new(&(r"^(?s)([.#])((?:".to_owned() + &*ESCAPE_RE_STR + r"\s|\\.|[^,.#:[ >~+])+)" + r"(.*)$")).unwrap();
        static ref _ATTRIBUTES_RE: Regex = Regex::new(&(r"^(?s)".to_owned() + &*ATTR_RE_STR + r"(.*)$")).unwrap();
        static ref _PSEUDO_CLASS_RE: Regex = Regex::new(&(r"^(?s):([\w-]+)(?:\(((?:\([^)]+\)|[^)])+)\))?".to_owned() + r"(.*)$")).unwrap();
        static ref _TAG_RE: Regex = Regex::new(&(r"^(?s)((?:".to_owned() + &*ESCAPE_RE_STR + r"\s|\\.|[^,.#:[ >~+])+)" + r"(.*)$")).unwrap();
    }

    let mut conditions: Vec<ConditionItem> = Vec::new();
    loop {
        // Class or ID
        if let Some(caps) = _CLASS_OR_ID_RE.captures(css) {
            let prefix = caps.at(1).unwrap();
            let (name, op) = if prefix == "." { ("class", "~") } else { ("id", "") };
            let op_val = caps.at(2);
            conditions.push(ConditionItem::Attribute { name: _name_re(name), value: _value_re(op, op_val, false) });
            css = caps.at(3).unwrap_or("");
        }

        // Attributes
        else if let Some(caps) = _ATTRIBUTES_RE.captures(css) {
            let name = caps.at(1).unwrap();
            let op = caps.at(2).unwrap_or("");
            let op_val = caps.at(3).or(caps.at(4)).or(caps.at(5));
            let op_insensitive = caps.at(6).is_some();
            conditions.push(ConditionItem::Attribute { name: _name_re(name), value: _value_re(op, op_val, op_insensitive) });
            css = caps.at(7).unwrap_or("");
        }

        // Pseudo-class
        else if let Some(caps) = _PSEUDO_CLASS_RE.captures(css) {
            let name = caps.at(1).unwrap().to_owned().to_lowercase();
            let args = caps.at(2);

            // ":not" (contains more selectors)
            if name == "not" {
                conditions.push(ConditionItem::PseudoClass { class: name, group: args.map(parse), equation: None });
            }
            // ":nth-*" (with An+B notation)
            else if name.starts_with("nth-") {
                conditions.push(ConditionItem::PseudoClass { class: name, group: None, equation: args.map(_equation) });
            }
            // ":first-*" (rewrite to ":nth-*")
            else if name.starts_with("first-") {
                let name = "nth-".to_owned() + &name[6..];
                conditions.push(ConditionItem::PseudoClass { class: name, group: None, equation: Some((0, 1)) });
            }
            // ":last-*" (rewrite to ":nth-*")
            else if name.starts_with("last-") {
                let name = "nth-".to_owned() + &name;
                conditions.push(ConditionItem::PseudoClass { class: name, group: None, equation: Some((-1, 1)) });
            }
            else {
                // No args
                conditions.push(ConditionItem::PseudoClass { class: name, group: None, equation: None });
            }

            css = caps.at(3).unwrap_or("");
        }

        // Tag
        else if let Some(caps) = _TAG_RE.captures(css) {
            let name = caps.at(1).unwrap();
            if name != "*" {
                conditions.push(ConditionItem::Tag { name: _name_re(name) });
            }
            css = caps.at(2).unwrap_or("");
        }

        else { break; }
    }

    return (conditions, css);
}

fn _equation(equation_str: &str) -> (i32, i32) {
    lazy_static! {
        static ref _RE1: Regex = Regex::new(r"^\s*((?:\+|-)?\d+)\s*$").unwrap();
        static ref _RE2: Regex = Regex::new(r"^(?i)\s*((?:\+|-)?(?:\d+)?)?n\s*((?:\+|-)\s*\d+)?\s*$").unwrap();
    }

    if equation_str.is_empty() { return (0, 0); }

    // "even"
    if equation_str.trim().to_lowercase() == "even" { return (2, 2); }

    // "odd"
    if equation_str.trim().to_lowercase() == "odd" { return (2, 1); }

    // "4", "+4" or "-4"
    if let Some(caps) = _RE1.captures(equation_str) {
        let num = caps.at(1).unwrap().parse::<i32>().unwrap();
        return (0, num);
    }

    // "n", "4n", "+4n", "-4n", "n+1", "4n-1", "+4n-1" (and other variations)
    if let Some(caps) = _RE2.captures(equation_str) {
        let mut result = (0, 0);
        let num1 = caps.at(1).unwrap();
        result.0 = if num1 == "-" { -1 } else if num1.is_empty() { 1 } else { num1.parse::<i32>().unwrap() };
        if let Some(num2) = caps.at(2) {
            result.1 = num2.split_whitespace().collect::<Vec<&str>>().concat().parse::<i32>().unwrap();
        }
        return result;
    }

    return (0, 0);
}
