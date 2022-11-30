#[macro_use]
extern crate lalrpop_util;
extern crate ptree;

use ptree::output::print_tree_with;
use ptree::print_config::UTF_CHARS_BOLD;
use ptree::{Color, PrintConfig, Style};

use std::env;

use crate::espresso::espresso_minimizer;

lalrpop_mod!(pub verilog);
pub mod ast;
pub mod bdd;
pub mod espresso;

fn parser_exp(expr: &str) -> bool {
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
                print!("{}", i);
            }
            println!();

            let mut expression: String = String::from("f =");

            for i in espresso_output.iter() {
                println!("{}", i);
                for j in 0..item_name.len() {
                    match i.as_bytes()[j] as char {
                        '0' => {
                            expression.push_str(item_name[j].as_str());
                            expression.push('\'');
                        }
                        '1' => expression.push_str(item_name[j].as_str()),
                        _ => (),
                    }
                }
                if i != espresso_output.iter().last().unwrap() {
                    expression.push_str(" + ");
                }
            }

            println!("{}", expression);

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
    println!("Format: parser [type] [expr]");
    println!("    [type]: expr, module, test");
    println!("    [expr]: \"~a\"");
    println!("example:");
    println!("parser expr (1'b1&v)|(~u&(&m| |start)&t)");
    println!("parser test");
}

fn parser_test() {
    assert_eq!(parser_exp("(1'b1&v)|(~u&(&m| |start)&t)"), true);
    assert_eq!(parser_exp("001"), false);
    assert_eq!(parser_exp("100"), true);
    assert_eq!(parser_exp("1'b01"), true);
    assert_eq!(parser_exp("1'b2"), false);
    assert_eq!(parser_exp("2'hff"), true);
    assert_eq!(parser_exp("2'hf"), true);
    assert_eq!(parser_exp("1'h2"), true);
    assert_eq!(parser_exp("1'o7"), true);
    assert_eq!(parser_exp("1'o8"), false);
    assert_eq!(parser_exp("2'b"), false);
    assert_eq!(parser_exp("'b101"), true);
    assert_eq!(parser_exp("a|||b"), true);
    assert_eq!(parser_exp("a|| |b"), true);
    assert_eq!(parser_exp("||a || |b"), false);
    assert_eq!(
        parser_module("module mod(input [1:0] in, output out) { assign out = a[0]; }"),
        true
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            let type_here = &args[1];
            let expr = &args[2];
            match type_here.trim().to_lowercase().as_str() {
                "expr" => parser_exp(expr),
                "module" => parser_module(expr),
                _ => {
                    parser_help();
                    false
                }
            };
        }
        2 => {
            let type_here = &args[1];
            if type_here.trim().to_lowercase().as_str() == "test" {
                parser_test();
            } else {
                parser_help();
            }
        }
        _ => parser_help(),
    };
}
