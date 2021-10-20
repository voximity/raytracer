use std::{
    fs::File,
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use clap::{App, Arg};
use notify::Watcher;

use crate::interpret::{InterpretError, Interpreter, Value};

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
        .arg(
            Arg::with_name("sequence")
                .long("sequence")
                .help("The number of frames to capture. SDL files can read from global variable `t` to check what frame they're on. The output destination will create a folder and inside each frame will be placed.")
                .required(false)
                .takes_value(true)
        )
        .get_matches();

    fn render(matches: &clap::ArgMatches) -> Result<(), InterpretError> {
        let now = Instant::now();
        let scene =
            Interpreter::new(File::open(matches.value_of("SOURCE").unwrap()).unwrap())?.run()?;

        println!("Scene constructed in {}s", now.elapsed().as_secs_f32());

        scene.render_to(matches.value_of("output").unwrap(), image::ImageFormat::Png);
        println!(
            "Operation complete in in {}s\n",
            now.elapsed().as_secs_f32()
        );

        Ok(())
    }

    if matches.is_present("sequence") {
        let source = matches.value_of("SOURCE").unwrap();
        let out = matches.value_of("output").unwrap();

        let frames: u32 = matches
            .value_of("sequence")
            .unwrap()
            .parse()
            .expect("Failed to parse sequence frame count");

        let mut interpreter = Interpreter::new(File::open(source).unwrap()).unwrap();
        let _ = std::fs::remove_dir_all(out);
        let _ = std::fs::create_dir_all(out);

        for i in 0..frames {
            let mut path = PathBuf::from(out);
            path.push(format!("frame_{}.png", i));
            interpreter.set_global(String::from("t"), Value::Number(i as f64));

            let scene = interpreter.run_cloned().expect("Failed to construct scene");
            println!("Rendering to {}", path.as_os_str().to_str().unwrap());
            scene.render_to(path.as_os_str().to_str().unwrap(), image::ImageFormat::Png);
        }

        return;
    }

    if matches.is_present("watch") {
        let source = matches.value_of("SOURCE").unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_millis(500)).unwrap();
        watcher
            .watch(source, notify::RecursiveMode::NonRecursive)
            .unwrap();

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
