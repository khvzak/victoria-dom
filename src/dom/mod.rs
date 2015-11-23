pub mod css;
pub mod html;

use std::rc::Rc;

use self::html::TreeNode;

// base 6.34

struct DOM {
    tree: Rc<TreeNode>,
}

impl DOM {
    pub fn find(&self, css: &str) {

    }

    pub fn matches(&self, css: &str) -> bool {
        css::matches(&self.tree, css)
    }
}

fn new(html: &str) -> DOM {
    DOM { tree: html::parse(html) }
}
