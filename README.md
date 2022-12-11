## Introduction

This project is a simple verilog expression parser which implemented several functions as below:
1. Parse verilog functions into a pretty printable AST
2. Evaluate the expression to get the truth table of it
3. Port the Espresso Heuristic Logic Minimizer as a C static library which can also be used by Rust
4. Transform the result of Espresso stage into literature boolean algebra function
5. Construct a DAG structure for boolean algebra function
6. According to the library json file given by user to do a straightforward technology map
7. Print a valid netlist

## Compiling
The code was uploaded at this address: [https://github.com/Blameying/verilog_expr_parser_by_rust/tree/main/src](https://github.com/Blameying/verilog_expr_parser_by_rust/tree/main/src). The project was organized by the dependencies manager tool of rust: Cargo, if you have the rust-nightly environment, you only need to run this command in root directory(which contains the Cargo.toml file) of the project:

```shell
cargo build
#or
cargo run
```

Considering you may not have the Rust environment, I will provide the executable binary file for the popular platform as much as I can. You can find them in the **binary** directory and run them in your terminal directly.

You can pass some args to the tool as below:
```shell
Format: parser [type] [expr] [path-to-lib file]
    [type]: expr, module, test
    [expr]: "~a"
example:
parser expr (1'b1&v)|(~u&(&m| |start)&t) ./library.json
parser test ./library.json
```

## Drawbacks
1. If the given expression contains non-bitwise variables, the ast can be got but the program will be panicked after this stage.
2. Originally, the test command is designed for the AST stage, and after adding the new features, it did not work because of the Drawback 1.

![Screenshot 2022-12-11 at 11.20.15](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2011.20.15.png)

## Straightforward Technology Mapping

In the library.json file, the **AND**, **OR** and **NOT** gates are represented by **NAND** and **NOR** gates, and every gate can represented in different forms which has different cost.

Here's a simple example to show how it works, and the **AND** gate has two representations here:
```json
{
  "and": [
    {
      "nodes": [
        { "id": 0, "name": "NAND" },
        { "id": 1, "name": "INPUT" },
        { "id": 2, "name": "INPUT" },
        { "id": 3, "name": "NAND" },
        { "id": 4, "name": "INPUT" },
        { "id": 5, "name": "INPUT" }
      ],
      "edges": [
        [1, 0],
        [2, 0],
        [3, 1],
        [3, 2],
        [4, 3],
        [5, 3]
      ]
    },
    {
      "nodes": [
        { "id": 0, "name": "NOR" },
        { "id": 1, "name": "INPUT" },
        { "id": 2, "name": "INPUT" },
        { "id": 3, "name": "NOR" },
        { "id": 4, "name": "INPUT" },
        { "id": 5, "name": "INPUT" },
        { "id": 6, "name": "NOR" },
        { "id": 7, "name": "INPUT" },
        { "id": 8, "name": "INPUT" },
        { "id": 9, "name": "INPUT" },
        { "id": 10, "name": "INPUT" }
      ],
      "edges": [
        [1, 0],
        [2, 0],
        [3, 1],
        [4, 3],
        [5, 3],
        [6, 2],
        [7, 6],
        [8, 6],
        [9, 4],
        [9, 5],
        [10, 7],
        [10, 8]
      ]
    }
  ]
}
```
The I will use the figure below to show what is straightforward map:

![Screenshot 2022-12-11 at 11.32.26](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2011.32.26.png)

## The method to reducing the numbers of gates
After transforming the expression into truth table, the **Espresso** library will do the technology independent optimization. Here are the results of the given test case at this stage.

| expr | result |
| :--: | :----: |
| ~(!a \| b) | f = \<a\>\<b'\> |
| b\|(b&c)| f = \<b\> |
|a \| !a & b | f = \<b\> + \<a\> |
| ~(a&b) | f = \<b'\> + \<a'\> |
| !(c \|\| d)| f = \<c'\>\<d'\> |
| a&b&c \| (a&b&!d) \| (a&b&~e) | f = \<a\>\<c\>\<b\> + \<a\>\<d'\>\<b\> + \<a\>\<e'\>\<b\> |

## Algorithm Library Used
1. Lalrpop (parser generater)
2. Espressor Heuristic Logic Minimizer (technology independent optimizer)

## Test Result
![Screenshot 2022-12-11 at 13.04.08](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.04.08.png)

![Screenshot 2022-12-11 at 13.05.41](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.05.41.png)

![Screenshot 2022-12-11 at 13.06.14](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.06.14.png)

![Screenshot 2022-12-11 at 13.07.19](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.07.19.png)

![Screenshot 2022-12-11 at 13.07.57](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.07.57.png)

![Screenshot 2022-12-11 at 13.08.52](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.08.52.png)

![Screenshot 2022-12-11 at 13.09.09](https://blog-img-1310827095.cos.ap-beijing.myqcloud.com/Screenshot%202022-12-11%20at%2013.09.09.png)
