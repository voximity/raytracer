use std::{fs::File, time::Instant};

use clap::{App, Arg};

use crate::interpret::Interpreter;

mod ast;
mod interpret;
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

    let now = Instant::now();
    let scene = Interpreter::new(File::open(matches.value_of("SOURCE").unwrap()).unwrap())
        .unwrap()
        .run()
        .unwrap();

    println!("Scene constructed in {}s", now.elapsed().as_secs_f32());
    
    scene.render_to(matches.value_of("output").unwrap(), image::ImageFormat::Png);
    println!("Operation complete in in {}s", now.elapsed().as_secs_f32());
}
