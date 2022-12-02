use daggy::petgraph::adj::EdgeIndex;
use daggy::petgraph::Direction;
use daggy::{Dag, NodeIndex};
use serde::{Deserialize, Serialize};
use serde_json::Result;
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
    cost: u32,
}

impl DAGWithInfo {
    pub fn new(dag: Dag<Gate, u32>) -> DAGWithInfo {
        let mut input_nodes: u32 = 0;
        let mut output_nodes: u32 = 0;
        let mut cost: u32 = 0;

        for n in dag.raw_nodes() {
            if n.next_edge(Direction::Incoming).index() == u32::MAX as usize {
                input_nodes += 1;
            } else if n.next_edge(Direction::Outgoing).index() == u32::MAX as usize {
                output_nodes += 1;
            }

            match n.weight {
                Gate::Input(_) => (),
                _ => cost += 1,
            };
        }

        DAGWithInfo {
            input: input_nodes,
            output: output_nodes,
            dag,
            cost,
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
enum Gate {
    And,
    Or,
    Not,
    Input(String),
    NAND,
    NOR,
}

fn straightforward_map(path: &'static str) -> HashMap<String, Vec<DAGWithInfo>> {
    let file = File::open(path).unwrap();
    let lib: GateLibrary = serde_json::from_reader(file).unwrap();
    let mut map: HashMap<String, Vec<DAGWithInfo>> = HashMap::new();

    let get_dag = |v: &Vec<PatternGraph>| -> Vec<DAGWithInfo> {
        let mut dag_list: Vec<DAGWithInfo> = Vec::new();
        for g in v.iter() {
            let mut dag: Dag<Gate, u32> = Dag::new();
            let mut hashmap: HashMap<u32, NodeIndex> = HashMap::new();
            for n in g.nodes.iter() {
                let gate: Gate = match n.name.as_str() {
                    "NOR" => Gate::NOR,
                    "NAND" => Gate::NAND,
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

    map.insert("AND".to_string(), get_dag(&lib.and));
    map.insert("OR".to_string(), get_dag(&lib.or));
    map.insert("NOT".to_string(), get_dag(&lib.not));

    map
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

pub fn technology_map_by_nand_nor(boolean_function: String) -> String {
    let dag = transform_boolean_algebra_to_dag(boolean_function);

    for n in dag.raw_nodes() {
        println!("{:?} {:?}", n.weight, n.next_edge(Direction::Incoming));
    }

    let lib =
        straightforward_map("/home/blame/Workspace/verilog_expr_parser_by_rust/input/library.json");
    // for (i, _) in inputs.iter().enumerate() {}
    //
    String::from("Not finished")
}
