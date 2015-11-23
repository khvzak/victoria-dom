use std::rc::Rc;

use std::char;

use regex;
use regex::{Regex, Captures};

use dom::html::{TreeNode, NodeElem};

lazy_static! {
    static ref ESCAPE_RE_STR: String = r"\\[^0-9a-fA-F]|\\[0-9a-fA-F]{1,6}".to_string();

    static ref ATTR_RE_STR: String = String::new() +
        r"\[" +
        r"((?:" + &*ESCAPE_RE_STR + r"|[\w\-])+)" +                     // Key
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
    Tag { name: String },
    Attribute { name: String, value: Option<String> },
    PseudoClass {
        class: String,
        group: Option<GroupOfSelectors>,
        equation: Option<(i32, i32)>,
    },
}

pub type Selectors = Vec<Rc<SelectorItem>>;
pub type GroupOfSelectors = Vec<Rc<Selectors>>;

pub fn matches(tree: &Rc<TreeNode>, css: &str) -> bool {
    match tree.elem {
        NodeElem::Tag { .. } => _match(&_parse(css), tree, tree),
        _ => false
    }
}

pub fn select(tree: &Rc<TreeNode>, css: &str) -> Vec<Rc<TreeNode>> {
    let group = _parse(css);

    let mut result = Vec::new();

    let mut queue = tree.get_childs().unwrap();
    while queue.len() > 0 {
        let current = queue.remove(0);
        if let NodeElem::Tag { .. } = current.elem {} else { continue; }

        queue = { let mut x = current.get_childs().unwrap(); x.append(&mut queue); x };
        if _match(&group, &current, tree) { result.push(current); }
    }

    result
}

pub fn select_one(tree: &Rc<TreeNode>, css: &str) -> Option<Rc<TreeNode>> {
    let group = _parse(css);

    let mut queue = tree.get_childs().unwrap();
    while queue.len() > 0 {
        let current = queue.remove(0);
        if let NodeElem::Tag { .. } = current.elem {} else { continue; }

        queue = { let mut x = current.get_childs().unwrap(); x.append(&mut queue); x };

        //println!("\n[select_one] current: {}", current.dbg_string());

        if _match(&group, &current, tree) { return Some(current); }
    }

    None
}

fn _match(group: &GroupOfSelectors, current: &Rc<TreeNode>, tree: &Rc<TreeNode>) -> bool {
    for _selectors in group {
        let selectors = Rc::new(_selectors.iter().rev().map(|x| x.clone()).collect::<Selectors>());
        //println!("[_match] selectors: {:?}", selectors);
        if _combinator(&selectors, current, tree, 0) { /*println!("[_match] -> true");*/ return true; }
        //println!("[_match] -> false");
    }
    return false;
}

fn _combinator(selectors: &Rc<Selectors>, current: &Rc<TreeNode>, tree: &Rc<TreeNode>, mut idx: usize) -> bool {
    if idx >= selectors.len() { return false; }

    let mut si = selectors[idx].clone();

    //println!("[_combinator] si: {:?}; current: {}", si, current.dbg_string());

    if let SelectorItem::Conditions { ref items } = *si.clone() {
        if !_match_selector_conditions(items, current) { return false; }

        idx = idx + 1;
        if idx >= selectors.len() { return true; }
        si = selectors[idx].clone();
    }

    //println!("[_combinator] _match_selector_conditions -> true; next_si: {:?}", si);

    match *si {
        SelectorItem::Combinator { ref op } => {
            idx = idx + 1;

            // ">" (parent only)
            if op == ">" {
                if current.parent.is_none() { return false; }
                let parent = current.get_parent().unwrap();

                //println!("[_combinator] op: \">\"; parent: {}", parent.dbg_string());

                // no suitable parent
                if let NodeElem::Root { .. } = parent.elem { return false; }
                if parent.id == tree.id { return false; }

                return _combinator(selectors, &parent, tree, idx);
            }

            // "~" (preceding siblings)
            if op == "~" {
                for ref sibling in _siblings(current, None) {
                    if sibling.id == current.id { return false; }
                    if _combinator(selectors, sibling, tree, idx) { return true; }
                }
                return false;
            }

            // "+" (immediately preceding siblings)
            if op == "+" {
                let mut found = false;
                for ref sibling in _siblings(current, None) {
                    if sibling.id == current.id { return found; }
                    found = _combinator(selectors, sibling, tree, idx);
                }
                return false;
            }

            // " " (ancestor)
            let mut parent = current.get_parent();
            while parent.is_some() {
                let parent_ok = parent.clone().unwrap();

                if let NodeElem::Root { .. } = parent_ok.elem { return false; }
                if parent_ok.id == tree.id { return false; }

                if _combinator(selectors, &parent_ok, tree, idx) { return true; }

                parent = parent_ok.get_parent();
            }
            return false;
        },

        _ => return false
    }
}

fn _match_selector_conditions(conditions: &Vec<ConditionItem>, current: &Rc<TreeNode>) -> bool {
    for c in conditions {
        match c {
            &ConditionItem::Tag { name: ref name_re } => {
                if !Regex::new(name_re).unwrap().is_match(current.get_tag_name().unwrap()) { return false; }
            },

            &ConditionItem::Attribute { name: ref name_re, value: ref value_re } => {
                let attrs = current.get_tag_attrs().unwrap();

                let result = (|| -> bool {
                    for (name, value) in attrs.iter() {
                        if Regex::new(name_re).unwrap().is_match(name) {
                            if value_re.is_none() { return true; }
                            if value.is_none() { return false; }

                            let (value_re_str, value_str) = (value_re.clone().unwrap(), value.clone().unwrap());
                            if Regex::new(&value_re_str).unwrap().is_match(&value_str) { return true; }
                        }
                    }
                    return false;
                })();

                if !result { return false; }
            },

            &ConditionItem::PseudoClass { ref class, ref group, ref equation } => {
                // ":empty"
                if class == "empty" {
                    let _is_empty = |x: &TreeNode| match x.elem {
                        NodeElem::Text { ref elem_type, .. } => elem_type == "comment" || elem_type == "pi",
                        _ => false,
                    };

                    return current.get_childs().unwrap().iter().filter(|&x| !_is_empty(x)).count() == 0;
                }

                // ":root"
                if class == "root" {
                    let parent = current.get_parent();
                    return parent.is_some() && match parent.unwrap().elem { NodeElem::Root { .. } => true, _ => false };
                }

                // ":not"
                if class == "not" {
                    return !_match(&group.clone().unwrap(), current, current);
                }

                // ":checked"
                if class == "checked" {
                    return match current.elem {
                        NodeElem::Tag { name: _, ref attrs, .. } => attrs.contains_key("checked") || attrs.contains_key("selected"),
                        _ => false
                    };
                }

                let _nth = |_class: &str, _equation: (i32, i32)| -> bool {
                    if _class.starts_with("nth-") {
                        let mut siblings = if _class.ends_with("of-type") {
                            _siblings(current, Some(current.get_tag_name().unwrap()))
                        } else {
                            _siblings(current, None)
                        };

                        // ":nth-last-*"
                        if _class.starts_with("nth-last") { siblings.reverse() }

                        for i in 0..(siblings.len()-1) {
                            let result = _equation.0 * (i as i32) + _equation.1;

                            if result < 1 { continue; }
                            if (result - 1) as usize >= siblings.len() { break; }

                            if siblings[(result - 1) as usize].id == current.id { return true; }
                        }
                    }
                    return false;
                };

                // ":first-*" (rewrite with nth)
                if class.starts_with("first-") {
                    return _nth(&("nth-".to_string() + class.trim_left_matches("first-")), (0, 1));
                }

                // ":last-*" (rewrite with nth)
                if class.starts_with("last-") {
                    return _nth(&("nth-last-".to_string() + class.trim_left_matches("last-")), (-1, 1));
                }

                // ":nth-*" or ":nth-last-*"
                if class.starts_with("nth-") {
                    return _nth(class, equation.unwrap());
                }

                // ":only-*"
                if class == "only-child" || class == "only-of-type" {
                    let siblings = if class == "only-of-type" {
                        _siblings(current, Some(current.get_tag_name().unwrap()))
                    } else {
                        _siblings(current, None)
                    };
                    for sibling in siblings {
                        if sibling.id != current.id { return false; }
                    }
                    return true;
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
    }).map(|x| x.clone()).collect()
}

fn _unescape(_val: &str) -> String {
    let mut val = _val.to_string();

    // Remove escaped newlines
    val = val.replace("\\\n", "");

    // Unescape Unicode characters
    let re = Regex::new(r"\\([0-9a-fA-F]{1,6})\s?").unwrap();
    val = re.replace_all(&val, |caps: &Captures| {
        let hex_char = caps.at(1).unwrap();
        format!("{}", char::from_u32(u32::from_str_radix(hex_char, 16).unwrap()).unwrap())
    });

    // Remove backslash
    val = val.replace(r"\", "");

    val
}

fn _name_re_str(_val: &str) -> String {
    r"(?:^|:)".to_string() + &regex::quote(&_unescape(_val)) + "$"
}

fn _value_re_str(op: &str, _val: Option<&str>, insensitive: bool) -> Option<String> {
    if _val.is_none() { return None };
    let mut value = regex::quote(_val.unwrap());

    if insensitive {
        value = "(?i)".to_string() + &value;
    }

    Some(
        // "~=" (word)
        if op == "~" {
            r"(?:^|\s+)".to_string() + &value + r"(?:\s+|$)"
        }

        // "*=" (contains)
        else if op == "*" {
            value
        }

        // "^=" (begins with)
        else if op == "^" {
            r"^".to_string() + &value
        }

        // "$=" (ends with)
        else if op == "$" {
            value + r"$"
        }

        // Everything else
        else {
            r"^".to_string() + &value + "$"
        }
    )
}

pub fn _parse(_css: &str) -> GroupOfSelectors {
    let mut css = _css;

    // Group separator re
    let re = Regex::new(r"^\s*,\s*(.*)$").unwrap();

    let mut group: GroupOfSelectors = Vec::new();
    loop {
        let (selectors, css_rest) = _parse_selector(css);
        if selectors.is_empty() { break; } else { group.push(Rc::new(selectors)); }
        css = css_rest;

        let caps_re = re.captures(css);
        if caps_re.is_some() {
            let caps = caps_re.unwrap();
            css = caps.at(1).unwrap();
        } else {
            break;
        }
    }

    group
}

fn _parse_selector(_css: &str) -> (Selectors, &str) {
    let mut css = _css;

    // Selector combinator re
    let re = Regex::new(r"^\s*([ >+~])\s*(.*)$").unwrap();

    let mut selectors: Selectors = Vec::new();
    loop {
        let mut conditions: Vec<ConditionItem> = Vec::new();
        loop {
            let (condition, css_rest) = _parse_selector_condition(css);
            if condition.is_some() {
                css = css_rest;
                conditions.push(condition.unwrap());
            } else {
                break;
            }
        }
        if conditions.is_empty() { break; } else { selectors.push(Rc::new(SelectorItem::Conditions {items: conditions})); }

        let caps_re = re.captures(css);
        if caps_re.is_some() {
            let caps = caps_re.unwrap();

            let op = caps.at(1).unwrap();
            css = caps.at(2).unwrap();

            selectors.push(Rc::new(SelectorItem::Combinator {op: op.to_string()}));
        } else {
            break;
        }
    }

    return (selectors, css);
}

fn _parse_selector_condition(css: &str) -> (Option<ConditionItem>, &str) {
    if css.is_empty() { return (None, css) }

    // Class or ID
    let re1_str = r"^([.#])((?:".to_string() + &*ESCAPE_RE_STR + r"\s|\\.|[^,.#:[ >~+])+)" + r"(.*)$";
    let caps_re1 = Regex::new(&re1_str).unwrap().captures(css);
    if caps_re1.is_some() {
        let caps = caps_re1.unwrap();

        let prefix = caps.at(1).unwrap();
        let op_val = caps.at(2);
        let css_rest = caps.at(3).unwrap_or("");
        let (name, op) = if prefix == "." { ("class", "~") } else { ("id", "") };

        return (Some(ConditionItem::Attribute {name: _name_re_str(name), value: _value_re_str(op, op_val, false)}), css_rest);
    }

    // Attributes
    let re2_str = r"^".to_string() + &*ATTR_RE_STR + r"(.*)$";
    let caps_re2 = Regex::new(&re2_str).unwrap().captures(css);
    if caps_re2.is_some() {
        let caps = caps_re2.unwrap();

        let name = caps.at(1).unwrap();
        let op = caps.at(2).unwrap_or("");
        let op_val = 
            if caps.at(3).is_some() {
                caps.at(3)
            } else if caps.at(4).is_some() {
                caps.at(4)
            } else if caps.at(5).is_some() {
                caps.at(5)
            } else {
                caps.at(6)
            };
        let css_rest = caps.at(7).unwrap_or("");

        return (Some(ConditionItem::Attribute {name: _name_re_str(name), value: _value_re_str(op, op_val, false)}), css_rest)
    }

    // Pseudo-class
    let re3_str = r"^:([\w-]+)(?:\(((?:\([^)]+\)|[^)])+)\))?".to_string() + r"(.*)$";
    let caps_re3 = Regex::new(&re3_str).unwrap().captures(css);
    if caps_re3.is_some() {
        let caps = caps_re3.unwrap();

        let pc_class = caps.at(1).unwrap().to_string().to_lowercase();
        let pc_css = caps.at(2).unwrap();
        let css_rest = caps.at(3).unwrap_or("");

        // ":not" contains more selectors
        if pc_class == "not" {
            return (
                Some(ConditionItem::PseudoClass { class: pc_class, group: Some(_parse(pc_css)), equation: None }),
                css_rest
            );
        }

        return (Some(ConditionItem::PseudoClass { class: pc_class, group: None, equation: _equation(pc_css) }), css_rest);
    }

    // Tag
    let re4_str = r"^((?:".to_string() + &*ESCAPE_RE_STR + r"\s|\\.|[^,.#:[ >~+])+)" + r"(.*)$";
    let caps_re4 = Regex::new(&re4_str).unwrap().captures(css);
    if caps_re4.is_some() {
        let caps = caps_re4.unwrap();

        let name = caps.at(1).unwrap();
        let css_rest = caps.at(2).unwrap_or("");

        if name != "*" {
            return (Some(ConditionItem::Tag {name: _name_re_str(name)}), css_rest);
        }
    }

    (None, css)
}

fn _equation(equation_str: &str) -> Option<(i32, i32)> {

    if equation_str.is_empty() { return None; }

    // even
    if equation_str.trim().to_lowercase() == "even" { return Some((2, 2)) }

    // odd
    if equation_str.trim().to_lowercase() == "odd" { return Some((2, 1)) }

    // Equation
    let mut num = (1, 1);

    let re = Regex::new(r"(?i)(?:(-?(?:\d+)?)?(n))?\s*\+?\s*(-?\s*\d+)?\s*$").unwrap();
    let caps_re = re.captures(equation_str);
    if caps_re.is_none() {
        return Some(num);
    } else {
        let caps = caps_re.unwrap();
        num.0 =
            if caps.at(1).is_some() && !caps.at(1).unwrap().is_empty() {
                if caps.at(1).unwrap() == "-" { -1 } else { caps.at(1).unwrap().parse::<i32>().unwrap() }
            }
            else if caps.at(2).is_some() { 1 } else { 0 };
        num.1 =
            if caps.at(3).is_some() {
                caps.at(3).unwrap().split_whitespace().collect::<Vec<&str>>().concat().parse::<i32>().unwrap()
            } else { 0 } 
    }

    Some(num)
}
