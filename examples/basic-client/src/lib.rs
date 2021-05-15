use fivem::ref_funcs::{ExternRefFunction, RefFunction};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Export(ExternRefFunction);

// hack!
macro_rules! cfx_export {
    ($res:expr, $exp:expr) => {{
        let link = std::rc::Rc::new(std::cell::RefCell::new(None));
        let link_clone = link.clone();
        let export0 = format!("__cfx_export_{}_{}", $res, $exp);

        let func = fivem::ref_funcs::RefFunction::new(move |input: Vec<Export>| -> Vec<bool> {
            *link_clone.borrow_mut() = Some(input[0].0.clone());

            vec![true]
        });

        let export_data = func.as_extern_ref_func();
        fivem::events::emit(&export0, vec![Export(export_data)]);

        link
    }};
}

#[derive(Debug, Serialize, Deserialize)]
struct SpawnInfo {
    x: f32,
    y: f32,
    z: f32,
    heading: f32,
    idx: u32,
    model: u64,

    #[serde(rename = "skipFade")]
    skip_fade: bool,
}

#[derive(Serialize)]
struct SpawnPlayer(SpawnInfo, ExternRefFunction);

#[no_mangle]
pub extern "C" fn _start() {
    const SPAWN_INFO: SpawnInfo = SpawnInfo {
        x: 686.245,
        y: 577.950,
        z: 130.461,

        heading: 0.0,
        idx: 0,
        model: 0x5761f4ad, // g_m_m_mexboss_01
        skip_fade: false,
    };

    let set_callback = cfx_export!("spawnmanager", "setAutoSpawnCallback");
    let spawn_player = cfx_export!("spawnmanager", "spawnPlayer");
    let set_autospawn = cfx_export!("spawnmanager", "setAutoSpawn");
    let force_respawn = cfx_export!("spawnmanager", "forceRespawn");

    let task = async move {
        let on_spawn = RefFunction::new(|spawn_info: Vec<SpawnInfo>| -> Vec<bool> {
            fivem::log(format!("player spawned: {:?}", spawn_info));
            vec![true]
        });

        let callback = RefFunction::new(move |_: Vec<()>| -> Vec<u8> {
            spawn_player
                .borrow()
                .as_ref()
                .unwrap()
                .invoke::<(), _>(SpawnPlayer(SPAWN_INFO, on_spawn.as_extern_ref_func()));

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
