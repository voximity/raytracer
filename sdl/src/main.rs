#![allow(dead_code)]

use std::fs::File;

use ast::AstParser;
use clap::{App, Arg};
use tokenize::Tokenizer;

use crate::interpret::Interpreter;

mod ast;
mod interpret;
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
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .help("The output file")
                .default_value("render.png")
                .required(false),
        )
        .get_matches();

    let scene = Interpreter::new(File::open(matches.value_of("SOURCE").unwrap()).unwrap()).unwrap().run().unwrap();
    scene.render_to(matches.value_of("output").unwrap(), image::ImageFormat::Png);
}
