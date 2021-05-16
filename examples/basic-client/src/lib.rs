use fivem::ref_funcs::{ExternRefFunction, RefFunction};
use serde::{Deserialize, Serialize};

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

    let set_callback =
        fivem::exports::import_function("spawnmanager", "setAutoSpawnCallback").unwrap();
    let spawn_player = fivem::exports::import_function("spawnmanager", "spawnPlayer").unwrap();
    let set_autospawn = fivem::exports::import_function("spawnmanager", "setAutoSpawn").unwrap();
    let force_respawn = fivem::exports::import_function("spawnmanager", "forceRespawn").unwrap();

    let task = async move {
        let on_spawn = RefFunction::new(|spawn_info: Vec<SpawnInfo>| -> Vec<bool> {
            fivem::log(format!("player spawned: {:?}", spawn_info));
            vec![true]
        });

        let callback = RefFunction::new(move |_: Vec<()>| -> Vec<u8> {
            spawn_player.invoke::<(), _>(SpawnPlayer(SPAWN_INFO, on_spawn.as_extern_ref_func()));
            vec![]
        });

        set_callback.invoke::<(), _>(vec![callback.as_extern_ref_func()]);
        set_autospawn.invoke::<(), _>(vec![true]);
        force_respawn.invoke::<(), Vec<u8>>(vec![]);
    };

    let _ = fivem::runtime::spawn(task);
}
