use std::{borrow::Cow, fmt::Debug, io};
extern crate ptree;

use ptree::{Style, TreeItem};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeNode {
    pub tag: String,
    pub val: String,
    pub subs: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(tag: &'static str, val: String, subs: Vec<TreeNode>) -> Self {
        TreeNode {
            tag: tag.to_string(),
            val,
            subs,
        }
    }
}

impl TreeItem for TreeNode {
    type Child = Self;
    fn write_self<W: io::Write>(&self, f: &mut W, style: &Style) -> io::Result<()> {
        write!(f, "{}", style.paint(&self.val))
    }

    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(&self.subs)
    }
}
