use std::collections::HashMap;

use fivem::ref_funcs::{ExternRefFunction, RefFunction};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Export {
    export_data: ExternRefFunction,
}

// hack!
macro_rules! cfx_export {
    ($res:expr, $exp:expr) => {{
        #[derive(serde::Deserialize, serde::Serialize)]
        struct Export {
            export_data: fivem::ref_funcs::ExternRefFunction,
        }

        let link = std::rc::Rc::new(std::cell::RefCell::new(None));
        let link_clone = link.clone();
        let export0 = format!("__cfx_export_{}_{}", $res, $exp);

        let func = fivem::ref_funcs::RefFunction::new(move |input: Export| -> Vec<u8> {
            *link_clone.borrow_mut() = Some(input.export_data.clone());

            vec![]
        });

        let export_data = func.as_extern_ref_func();
        fivem::events::emit(&export0, Export { export_data });

        link
    }};
}

#[derive(Serialize)]
struct Pos {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize)]
struct SpawnPlayer {
    pos: HashMap<String, f32>,
    on_spawn: ExternRefFunction,
}

#[no_mangle]
pub extern "C" fn _start() {
    const POS: Pos = Pos {
        x: 686.245,
        y: 577.950,
        z: 130.461,
    };

    let set_callback = cfx_export!("spawnmanager", "setAutoSpawnCallback");
    let spawn_player = cfx_export!("spawnmanager", "spawnPlayer");
    let set_autospawn = cfx_export!("spawnmanager", "setAutoSpawn");
    let force_respawn = cfx_export!("spawnmanager", "forceRespawn");

    let task = async move {
        let on_spawn = RefFunction::new(|_: Vec<()>| -> Vec<u8> { vec![] });
        let callback = RefFunction::new(move |_: Vec<()>| -> Vec<u8> {
            let mut pos = HashMap::new();
            pos.insert("x".to_owned(), POS.x);
            pos.insert("y".to_owned(), POS.y);
            pos.insert("z".to_owned(), POS.z);

            spawn_player
                .borrow()
                .as_ref()
                .unwrap()
                .invoke::<(), _>(SpawnPlayer {
                    pos,
                    on_spawn: on_spawn.as_extern_ref_func(),
                });

            vec![]
        });

        set_callback
            .borrow()
            .as_ref()
            .unwrap()
            .invoke::<(), _>(vec![callback.as_extern_ref_func()]);

        set_autospawn
            .borrow()
            .as_ref()
            .unwrap()
            .invoke::<(), _>(vec![true]);

        force_respawn
            .borrow()
            .as_ref()
            .unwrap()
            .invoke::<(), Vec<u8>>(vec![]);
    };

    fivem::log("started ...");

    let _ = fivem::runtime::spawn(task);
}
