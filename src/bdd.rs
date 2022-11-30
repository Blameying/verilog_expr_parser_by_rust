use crate::ast::TreeNode;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::From;
use std::sync::Mutex;

lazy_static! {
    pub static ref MAP: Mutex<HashMap<String, bool>> = {
        #![allow(unused_mut)]
        let mut map = HashMap::new();
        Mutex::new(map)
    };
}

#[allow(dead_code)]
pub struct BNode {
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
            "identifier" => {
                bnode.is_leave = true;
                bnode.operator = value.val.clone();
                MAP.lock().unwrap().insert(value.val.clone(), false);
                Some(Box::new(|a: bool, _b: bool| a))
            }
            _ => None,
        };

        if func.is_none() {
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
    pub fn eval(&self) -> Option<bool> {
        if self.is_leave {
            println!(
                "leave: {}, operator: {}",
                *MAP.lock().unwrap().get(&self.operator).unwrap(),
                self.operator
            );
            Some(*MAP.lock().unwrap().get(&self.operator).unwrap())
        } else {
            println!("operator: {}", self.operator);
            self.method.as_ref().map(|m| {
                m(
                    self.left.as_ref().unwrap().eval().unwrap(),
                    self.right.as_ref().unwrap().eval().unwrap(),
                )
            })
        }
    }

    pub fn create_truthtable(&self) -> (Vec<String>, Vec<String>) {
        let len: usize = MAP.lock().unwrap().len();
        let stop_mask: usize = (1 << len) - 1;
        let mut ret: Vec<String> = Vec::new();

        let list: Vec<String> = MAP.lock().unwrap().keys().cloned().collect();
        println!("{:?}", list);

        ret.push(format!(".i {}", list.len()));
        ret.push(String::from(".o 1"));

        for mut i in 0..=stop_mask {
            for (j, item) in list.iter().enumerate() {
                let mask: usize = 1 << (list.len() - 1 - j);
                MAP.lock()
                    .unwrap()
                    // repair here, to reduce the memory usage
                    .entry(item.clone())
                    .and_modify(|v| *v = (mask & i) > 0);
            }
            let result: bool = self.eval().unwrap();

            i <<= 1;

            if result {
                i |= 1;
            }

            ret.push(format!(
                "{:0width$b} {:1b}",
                i >> 1,
                i & 1,
                width = list.len()
            ));
        }
        ret.push(String::from(".e"));
        (ret, list)
    }

    pub fn get_inputs(&self) -> &MAP {
        &MAP
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
            is_leave: false,
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
