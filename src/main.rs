#![feature(is_some_and)]

#[macro_use]
extern crate lalrpop_util;
extern crate ptree;

use ptree::output::print_tree_with;
use ptree::print_config::UTF_CHARS_BOLD;
use ptree::{Color, PrintConfig, Style};

use std::env;

use crate::espresso::espresso_minimizer;
use crate::technology_map::technology_map_by_nand_nor;

lalrpop_mod!(pub verilog);
pub mod ast;
pub mod bdd;
pub mod espresso;
pub mod technology_map;

fn parser_exp(expr: &str, path: Option<&str>) -> bool {
    let config = {
        let mut config = PrintConfig::from_env();
        config.leaf = Style {
            bold: true,
            foreground: Some(Color::Green),
            ..Style::default()
        };
        config.characters = UTF_CHARS_BOLD.into();
        config.indent = 4;
        config
    };
    println!("Expr: {}", expr);
    let root = verilog::ExprParser::new().parse(expr);
    match root {
        Ok(t) => {
            println!("AST Tree:");
            print_tree_with(&t, &config).unwrap();
            let bnode = bdd::BNode::from(&t);
            let (truthtable, item_name) = bnode.create_truthtable();
            println!("{:?}", truthtable);
            let espresso_output: Vec<String> = espresso_minimizer(truthtable);
            println!("Espresso result: ");
            for i in item_name.iter() {
                print!("{}|", i);
            }
            println!();

            let mut expression: String = String::from("f = ");

            for i in espresso_output.iter() {
                println!("{}", i);
                for (j, _) in item_name.iter().enumerate() {
                    match i.as_bytes()[j] as char {
                        '0' => {
                            expression.push('<');
                            expression.push_str(item_name[j].as_str());
                            expression.push('\'');
                            expression.push('>');
                        }
                        '1' => {
                            expression.push('<');
                            expression.push_str(item_name[j].as_str());
                            expression.push('>');
                        }
                        _ => (),
                    }
                }
                if i != espresso_output.iter().last().unwrap() {
                    expression.push_str(" + ");
                }
            }

            println!("----------------------------------------------");
            println!("Optimized Boolean Algebra:");
            println!("{}", expression);
            println!("----------------------------------------------");
            println!("Technology Mapping:");

            if item_name.len() == 1 && expression.find('\'').is_none() {
                println!("module test(input {}, output out);", item_name[0]);
                println!("assign out = {};", item_name[0]);
                println!("endmodule")
            } else {
                println!("{:?}", item_name);
                println!(
                    "\n\n{}",
                    technology_map_by_nand_nor(expression, path.unwrap_or("./library.json"))
                );
            }

            println!("----------------------------------------------");
            true
        }
        Err(e) => {
            println!("Error: {:?}", e);
            println!("----------------------------------------------");
            false
        }
    }
}

fn parser_module(expr: &str) -> bool {
    let config = {
        let mut config = PrintConfig::from_env();
        config.leaf = Style {
            bold: true,
            foreground: Some(Color::Green),
            ..Style::default()
        };
        config.characters = UTF_CHARS_BOLD.into();
        config.indent = 4;
        config
    };
    println!("Module: {}", expr);
    let root = verilog::Module_scopeParser::new().parse(expr);
    match root {
        Ok(t) => {
            println!("AST Tree:");
            print_tree_with(&t, &config).unwrap();
            println!("----------------------------------------------");
            true
        }
        Err(e) => {
            println!("Error: {:?}", e);
            println!("----------------------------------------------");
            false
        }
    }
}

fn parser_help() {
    println!("Format: parser [type] [expr] [path-to-lib file]");
    println!("    [type]: expr, module, test");
    println!("    [expr]: \"~a\"");
    println!("example:");
    println!("parser expr (1'b1&v)|(~u&(&m| |start)&t) ./library.json");
    println!("parser test ./library.json");
}

fn parser_test(path: &str) {
    assert!(parser_exp("(1'b1&v)|(~u&(&m| |start)&t)", Some(path)));
    assert!(!parser_exp("001", Some(path)));
    assert!(parser_exp("100", Some(path)));
    assert!(parser_exp("1'b01", Some(path)));
    assert!(!parser_exp("1'b2", Some(path)));
    assert!(parser_exp("2'hff", Some(path)));
    assert!(parser_exp("2'hf", Some(path)));
    assert!(parser_exp("1'h2", Some(path)));
    assert!(parser_exp("1'o7", Some(path)));
    assert!(!parser_exp("1'o8", Some(path)));
    assert!(!parser_exp("2'b", Some(path)));
    assert!(parser_exp("'b101", Some(path)));
    assert!(parser_exp("a|||b", Some(path)));
    assert!(parser_exp("a|| |b", Some(path)));
    assert!(!parser_exp("||a || |b", Some(path)));
    assert!(parser_module(
        "module mod(input [1:0] in, output out) { assign out = a[0]; }"
    ));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        4 => {
            let type_here = &args[1];
            let expr = &args[2];
            match type_here.trim().to_lowercase().as_str() {
                "expr" => parser_exp(expr, Some(args[3].as_str())),
                "module" => parser_module(expr),
                _ => {
                    parser_help();
                    false
                }
            };
        }
        3 => {
            let type_here = &args[1];
            let expr = &args[2];
            match type_here.trim().to_lowercase().as_str() {
                "expr" => parser_exp(expr, Some("./library.json")),
                "module" => parser_module(expr),
                "test" => {
                    parser_test(expr.as_str());
                    true
                }
                _ => {
                    parser_help();
                    false
                }
            };
        }
        2 => {
            let type_here = &args[1];
            if type_here.trim().to_lowercase().as_str() == "test" {
                parser_test("./library.json");
            } else {
                parser_help();
            }
        }
        _ => parser_help(),
    };
}
