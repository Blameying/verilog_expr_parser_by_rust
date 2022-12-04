use daggy::petgraph::visit::IntoNodeReferences;
use daggy::{Dag, NodeIndex, Walker};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};

#[derive(Serialize, Deserialize)]
struct Node {
    id: u32,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct PatternGraph {
    nodes: Vec<Node>,
    edges: Vec<Vec<u32>>,
}

#[derive(Serialize, Deserialize)]
struct GateLibrary {
    and: Vec<PatternGraph>,
    or: Vec<PatternGraph>,
    not: Vec<PatternGraph>,
}

#[derive(Debug)]
struct DAGWithInfo {
    input: u32,
    output: u32,
    dag: Dag<Gate, u32>,
    cost: f32,
    input_nodes: Vec<NodeIndex>,
    output_nodes: Vec<NodeIndex>,
}

impl DAGWithInfo {
    pub fn new(dag: Dag<Gate, u32>) -> DAGWithInfo {
        let mut input_nodes: Vec<NodeIndex> = Vec::new();
        let mut output_nodes: Vec<NodeIndex> = Vec::new();
        let mut cost: u32 = 0;

        for (index, n) in dag.node_references() {
            if dag.children(index).iter(&dag).count() == 0 {
                output_nodes.push(index);
            } else if dag.parents(index).iter(&dag).count() == 0 {
                input_nodes.push(index);
            }

            match n {
                Gate::Input(_) => (),
                _ => cost += 1,
            };
        }

        DAGWithInfo {
            input: input_nodes.len() as u32,
            output: output_nodes.len() as u32,
            dag,
            cost: cost as f32 / input_nodes.len() as f32,
            input_nodes,
            output_nodes,
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
enum Gate {
    And,
    Or,
    Not,
    Input(String),
    Nand,
    Nor,
}

fn replace_node_by_graph(src: &[DAGWithInfo], target: &mut Dag<Gate, u32>, target_node: NodeIndex) {
    let input_list: Vec<daggy::NodeIndex> = target
        .parents(target_node)
        .iter(target)
        .map(|(_, n)| n)
        .collect();
    let output = target.children(target_node).iter(target).next();
    let mut stack: Vec<NodeIndex> = Vec::new();
    stack.clone_from(&input_list);

    while !stack.is_empty() {
        let sub_pattern_option = src.iter().find(|f| f.input as usize <= stack.len());

        if sub_pattern_option.is_none() {
            break;
        }

        let sub_pattern = sub_pattern_option.unwrap();

        let root = sub_pattern.output_nodes.first().unwrap();
        let new_root = target.add_node(sub_pattern.dag.node_weight(*root).unwrap().clone());

        /*(index in sub_pattern, index in target) */
        let mut parent_stack: Vec<(NodeIndex, NodeIndex)> = Vec::new();

        for (_, n) in sub_pattern.dag.parents(*root).iter(&sub_pattern.dag) {
            let idx = target.add_node(sub_pattern.dag.node_weight(n).unwrap().clone());
            target.add_edge(idx, new_root, 1).unwrap();
            parent_stack.push((n, idx));
        }

        let mut new_input_list: Vec<NodeIndex> = Vec::new();
        while !parent_stack.is_empty() {
            let mut repeat_node: HashMap<NodeIndex, NodeIndex> = HashMap::new();
            for (idx_sub, idx_new) in &parent_stack {
                if sub_pattern
                    .dag
                    .parents(*idx_sub)
                    .iter(&sub_pattern.dag)
                    .count()
                    != 0
                {
                    for (_, n) in sub_pattern.dag.parents(*idx_sub).iter(&sub_pattern.dag) {
                        if repeat_node.contains_key(&n) {
                            target
                                .add_edge(*repeat_node.get(&n).unwrap(), *idx_new, 1)
                                .unwrap();
                        } else {
                            let idx =
                                target.add_node(sub_pattern.dag.node_weight(n).unwrap().clone());
                            target.add_edge(idx, *idx_new, 1).unwrap();
                            repeat_node.insert(n, idx);
                        }
                    }
                } else {
                    new_input_list.push(*idx_new);
                }
            }
            parent_stack.clear();
            for (k, v) in repeat_node {
                parent_stack.push((k, v));
            }
        }

        for i in new_input_list {
            let idx = stack.pop().unwrap();
            target.add_edge(idx, i, 1).unwrap();
        }

        stack.push(new_root);
        if input_list.len() == 1 {
            break;
        }
    }

    if let Some((_, n)) = output {
        target.add_edge(stack.pop().unwrap(), n, 1).unwrap();
    }
}

fn straightforward_map(path: &'static str, mut origin: Dag<Gate, u32>) -> Dag<Gate, u32> {
    let file = File::open(path).unwrap();
    let lib: GateLibrary = serde_json::from_reader(file).unwrap();

    let get_dag = |v: &Vec<PatternGraph>| -> Vec<DAGWithInfo> {
        let mut dag_list: Vec<DAGWithInfo> = Vec::new();
        for g in v.iter() {
            let mut dag: Dag<Gate, u32> = Dag::new();
            let mut hashmap: HashMap<u32, NodeIndex> = HashMap::new();
            for n in g.nodes.iter() {
                let gate: Gate = match n.name.as_str() {
                    "NOR" => Gate::Nor,
                    "NAND" => Gate::Nand,
                    "INPUT" => Gate::Input(n.id.to_string()),
                    _ => panic!("Error gate type in json"),
                };
                hashmap.insert(n.id, dag.add_node(gate));
            }

            for e in g.edges.iter() {
                dag.add_edge(
                    *hashmap.get(&e[0]).unwrap(),
                    *hashmap.get(&e[1]).unwrap(),
                    1,
                )
                .unwrap();
            }

            dag_list.push(DAGWithInfo::new(dag));
        }
        dag_list
    };

    let mut and_lib = get_dag(&lib.and);
    let mut not_lib = get_dag(&lib.not);
    let mut or_lib = get_dag(&lib.or);

    and_lib.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap());
    not_lib.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap());
    or_lib.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap());

    let target = origin.clone();

    for (index, n) in target.node_references() {
        match n {
            Gate::And => replace_node_by_graph(&and_lib, &mut origin, index),
            Gate::Not => replace_node_by_graph(&not_lib, &mut origin, index),
            Gate::Or => replace_node_by_graph(&or_lib, &mut origin, index),
            _ => (),
        }
    }

    origin.filter_map(
        |_, n| match n {
            Gate::And => None,
            Gate::Not => None,
            Gate::Or => None,
            _ => Some(n.clone()),
        },
        |_, e| Some(*e),
    )
}

fn transform_boolean_algebra_to_dag(boolean_function: String) -> Dag<Gate, u32> {
    let (_, last) = boolean_function.split_at(boolean_function.find('=').unwrap() + 1);
    let or_level: Vec<&str> = last.split(&['+'][..]).map(|f| f.trim()).collect();
    let mut and_level: Vec<Vec<String>> = Vec::new();
    let mut dag: Dag<Gate, u32> = Dag::new();

    for v in or_level.iter() {
        let list: Vec<String> = v
            .split(&['<', '>'][..])
            .filter(|f| !f.trim().is_empty())
            .map(|f| String::from(f.trim()))
            .collect();
        and_level.push(list);
    }

    let or_gate = dag.add_node(Gate::Or);

    for v in and_level.iter() {
        let and_gate = dag.add_node(Gate::And);
        dag.add_edge(and_gate, or_gate, 1).unwrap();
        for i in v.iter() {
            if i.find('\'').is_some() {
                let not_gate = dag.add_node(Gate::Not);
                let input = dag.add_node(Gate::Input(i.clone()));
                dag.add_edge(not_gate, and_gate, 1).unwrap();
                dag.add_edge(input, not_gate, 1).unwrap();
            } else {
                let input = dag.add_node(Gate::Input(i.clone()));
                dag.add_edge(input, and_gate, 1).unwrap();
            }
        }
    }

    dag
}

fn generate_netlist(dag: Dag<Gate, u32>) -> String {
    let dag_info = DAGWithInfo::new(dag);
    let out_name = "out";
    let mut result = String::from("module test(");
    let mut name_pool: HashMap<NodeIndex, String> = HashMap::new();

    /* module interface definition */
    for i in dag_info.input_nodes.iter() {
        if let Gate::Input(s) = dag_info.dag.node_weight(*i).unwrap() {
            result.push_str("input ");
            result.push_str(s.trim_matches('\''));
            result.push_str(", ");
            name_pool.insert(*i, s.trim_matches('\'').to_string());
        }
    }

    result.push_str(format!("output {out_name});\n").as_str());

    /* wire name generator */
    let namer = || unsafe {
        static mut ID: usize = 0;
        ID += 1;
        ID
    };

    let gate_namer = || unsafe {
        static mut ID: usize = 0;
        ID += 1;
        ID
    };

    let mut parent_stack: Vec<NodeIndex> = Vec::new();
    parent_stack.push(*dag_info.output_nodes.first().unwrap());
    name_pool.insert(
        *dag_info.output_nodes.first().unwrap(),
        out_name.to_string(),
    );

    let mut gates_list: String = String::new();
    while !parent_stack.is_empty() {
        let mut all_child: Vec<NodeIndex> = Vec::new();
        all_child.clear();
        for &n in parent_stack.iter() {
            let mut gate_name = match dag_info.dag.node_weight(n).unwrap() {
                Gate::Nor => format!("NOR g{}(", gate_namer()),
                Gate::Nand => format!("NAND g{}(", gate_namer()),
                _ => panic!("It should not be here"),
            };

            for (_, n) in dag_info.dag.parents(n).iter(&dag_info.dag) {
                let mut parent_recursion = dag_info
                    .dag
                    .recursive_walk(n, |g, idx| g.parents(idx).iter(g).next());

                while let Some((_, node)) = parent_recursion.walk_next(&dag_info.dag) {
                    if dag_info.dag[node] == Gate::Nor
                        || dag_info.dag[node] == Gate::Nand
                        || name_pool.contains_key(&node)
                    {
                        if let Gate::Input(input_var) = dag_info.dag.node_weight(node).unwrap() {
                            gate_name.push_str(input_var.trim_matches('\''));
                            gate_name.push_str(", ");
                        } else {
                            let value = name_pool.entry(node).or_insert(format!("t{}", namer()));
                            gate_name.push_str(value.as_str());
                            gate_name.push_str(", ");
                            if !all_child.contains(&node) {
                                all_child.push(node);
                            }
                        }
                        break;
                    }
                }
            }
            gate_name.push_str(name_pool.get(&n).unwrap().as_str());
            gate_name.push_str(");\n");

            gates_list += &gate_name;
        }
        parent_stack.clear();
        parent_stack.clone_from(&all_child);
    }

    for (k, v) in name_pool {
        if (dag_info.dag[k] == Gate::Nand || dag_info.dag[k] == Gate::Nor)
            && (v.as_bytes()[0] as char == 't')
        {
            result += &format!("wire {};\n", v);
        }
    }

    result += &gates_list;
    result.push_str("endmodule");

    result
}

pub fn technology_map_by_nand_nor(boolean_function: String) -> String {
    let dag = transform_boolean_algebra_to_dag(boolean_function);

    let lib = straightforward_map(
        "/home/blame/Workspace/verilog_expr_parser_by_rust/input/library.json",
        dag,
    );

    println!("lib: {:?}", lib);
    generate_netlist(lib)
}
