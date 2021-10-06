use std::{fs, sync::mpsc, time::Duration};

use clap::{App, Arg};
use raytracer::{
    lua::LuaSceneObject,
    material::Material,
    math::Vector3,
    object::{self, SceneObject},
    scene::Scene,
};
use rlua::Lua;

macro_rules! userdata_constructors {
    ($ctx:ident $globals:ident: $($n:literal => $e:expr),+$(,)?) => {
        $($globals.set($n, $ctx.create_function($e).unwrap()).unwrap();)+
    };
}

enum SdlOp {
    SetViewportSize(i32, i32),
    AddObject(Box<dyn SceneObject>),
}

fn main() {
    let matches = App::new("Raytracer Lua SDL Runtime")
        .version("1.0")
        .author("Zander F. <zander@zanderf.net>")
        .about("A SDL runtime that uses Lua to describe a scene to the raytracer")
        .arg(
            Arg::with_name("SOURCE")
                .help("The source file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let mut scene = Scene::default();
    let (tx, rx) = mpsc::channel::<SdlOp>();
    let lua = Lua::new();

    lua.context(|ctx| {
        let globals = ctx.globals();

        // userdata construction
        userdata_constructors!(ctx globals:
            "Vector3" => |_, (x, y, z): (f64, f64, f64)| Ok(Vector3 { x, y, z }),
            "Aabb" => |_, (pos, size): (Vector3, Vector3)| Ok(object::Aabb::new(pos, size, Material::default())),
        );

        globals.set("viewport_size", ctx.create_function(move |_, (vw, vh): (i32, i32)| {
            tx.clone().send(SdlOp::SetViewportSize(vw, vh)).unwrap();
            Ok(())
        }).unwrap()).unwrap();

        ctx.load(
            &fs::read_to_string(matches.value_of("SOURCE").unwrap())
                .expect("Unable to read source file!"),
        )
        .exec()
        .expect("Unable to execute source file!");
    });

    while let Ok(op) = rx.recv_timeout(Duration::ZERO) {
        match op {
            SdlOp::SetViewportSize(vw, vh) => {
                scene.camera.vw = vw;
                scene.camera.vh = vh;
            }
            SdlOp::AddObject(obj) => scene.objects.push(obj),
        }
    }

    println!("{}, {}", scene.camera.vw, scene.camera.vh);
}
