use std::{fs::File, sync::mpsc, time::{Duration, Instant}};

use clap::{App, Arg};
use notify::Watcher;

use crate::interpret::{InterpretError, Interpreter};

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
        .arg(
            Arg::with_name("watch")
                .long("watch")
                .short("w")
                .help("Whether or not to watch the file and rerender on save")
                .required(false),
        )
        .get_matches();

    fn render(matches: &clap::ArgMatches) -> Result<(), InterpretError> {
        let now = Instant::now();
        let scene = Interpreter::new(File::open(matches.value_of("SOURCE").unwrap()).unwrap())?.run()?;
    
        println!("Scene constructed in {}s", now.elapsed().as_secs_f32());
    
        scene.render_to(matches.value_of("output").unwrap(), image::ImageFormat::Png);
        println!("Operation complete in in {}s\n", now.elapsed().as_secs_f32());

        Ok(())
    }

    if matches.is_present("watch") {
        let source = matches.value_of("SOURCE").unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_millis(500)).unwrap();
        watcher.watch(source, notify::RecursiveMode::NonRecursive).unwrap();

        println!("Now listening for file changes at {}", source);
        loop {
            match rx.recv() {
                Ok(event) if matches!(event, notify::DebouncedEvent::Write(_)) => {
                    if let Err(e) = render(&matches) {
                        println!("Failed to render: {}", e);
                    }
                }
                Err(_) => panic!("failed to watch file!"),
                _ => (),
            }
        }
    } else {
        if let Err(e) = render(&matches) {
            println!("Failed to render: {}", e);
        }
    }
}
