#![allow(dead_code)]

use std::fs::File;

use ast::AstParser;
use clap::{App, Arg};
use tokenize::Tokenizer;

mod ast;
mod reader;
mod tokenize;

fn main() {
    let matches = App::new("Raytracer SDL Interpreter")
        .version("1.0")
        .author("Zander F. <zander@zanderf.net>")
        .about("A SDL runtime that uses a proprietary SDL language to describe a scene to the raytracer")
        .arg(
            Arg::with_name("SOURCE")
                .help("The source file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let ast = AstParser::new(
        Tokenizer::new(File::open(matches.value_of("SOURCE").unwrap()).unwrap())
            .tokenize()
            .unwrap(),
    )
    .parse_root()
    .unwrap();

    println!("{:?}", ast);
}
