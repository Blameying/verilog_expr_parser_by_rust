use crate::ast::TreeNode;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::From;
use std::sync::Mutex;

lazy_static! {
    static ref MAP: Mutex<HashMap<String, bool>> = {
        #![allow(unused_mut)]
        let mut map = HashMap::new();
        Mutex::new(map)
    };
}

#[allow(dead_code)]
struct BNode {
    value: bool,
    is_leave: bool,
    operator: String,
    method: Option<Box<dyn Fn(bool, bool) -> bool>>,
    left: Option<Box<BNode>>,
    right: Option<Box<BNode>>,
}

impl From<&TreeNode> for BNode {
    fn from(value: &TreeNode) -> Self {
        let mut bnode = BNode {
            value: false,
            is_leave: false,
            operator: value.tag.clone(),
            method: None,
            left: None,
            right: None,
        };

        let func: Option<Box<dyn Fn(bool, bool) -> bool>> = match value.tag.to_lowercase().as_str()
        {
            "s~" | "s!" => Some(Box::new(|a: bool, _b: bool| !a)),
            "d+" => Some(Box::new(|a: bool, b: bool| a | b)),
            "d-" => Some(Box::new(|a: bool, b: bool| a != b)),
            "d&" => Some(Box::new(|a: bool, b: bool| a & b)),
            "d|" => Some(Box::new(|a: bool, b: bool| a | b)),
            "d&&" => Some(Box::new(|a: bool, b: bool| a & b)),
            "d||" => Some(Box::new(|a: bool, b: bool| a | b)),
            "Identifier" => {
                bnode.is_leave = true;
                bnode.operator = value.val.clone();
                MAP.lock().unwrap().insert(value.val.clone(), false);
                Some(Box::new(|a: bool, _b: bool| a))
            }
            _ => None,
        };

        if let None = &func {
            panic!("Unsupported operator in expression!");
        }

        bnode.method = func;
        match value.subs.len() {
            1 => {
                bnode.left = Some(Box::new(BNode::from(&value.subs[0])));
                bnode.right = Some(Box::new(BNode::from(&value.subs[0])));
            }
            2 => {
                bnode.left = Some(Box::new(BNode::from(&value.subs[0])));
                bnode.right = Some(Box::new(BNode::from(&value.subs[1])));
            }
            _ => (),
        }

        bnode
    }
}

impl BNode {
    fn eval(&self) -> Option<bool> {
        if self.is_leave {
            Some(*MAP.lock().unwrap().get(&self.operator).unwrap())
        } else {
            if let Some(m) = &self.method {
                Some(m(
                    self.left.as_ref().unwrap().eval().unwrap(),
                    self.right.as_ref().unwrap().eval().unwrap(),
                ))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bdd::BNode;
    use crate::bdd::MAP;
    #[test]
    fn test_bnode() {
        let bnode_left = BNode {
            value: false,
            is_leave: true,
            operator: String::from("a"),
            method: None,
            left: None,
            right: None,
        };
        let bnode_right = BNode {
            value: false,
            is_leave: true,
            operator: String::from("b"),
            method: None,
            left: None,
            right: None,
        };

        let bnode_root = BNode {
            value: false,
            is_leave: true,
            operator: String::from("2&"),
            method: Some(Box::new(|a: bool, b: bool| a & b)),
            left: Some(Box::new(bnode_left)),
            right: Some(Box::new(bnode_right)),
        };

        MAP.lock().unwrap().insert(String::from("a"), true);
        MAP.lock().unwrap().insert(String::from("b"), false);

        assert_eq!(bnode_root.eval(), Some(false));
    }
}
