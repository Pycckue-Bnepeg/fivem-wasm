use fivem::ref_funcs::{ExternRefFunction, RefFunction};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
        model: 0x4A8E5536,
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

    fivem::client::cfx::set_discord_app_id("843983771278901279");

    let logger = async {
        let wrapper = || {
            let player = fivem::client::player::player_ped_id();
            let camera = fivem::client::cam::get_gameplay_cam_coord();
            let player_pos = fivem::client::entity::get_entity_coords(player, false);
            let id = fivem::client::player::player_id();
            let name = fivem::client::player::get_player_name(id)?;

            fivem::log(format!(
                "player {} camera: {:?} player: {:?} name: {:?} id: {}",
                player, camera, player_pos, name, id
            ));

            Some(())
        };

        loop {
            wrapper();
            fivem::runtime::sleep_for(Duration::from_secs(5)).await;
        }
    };

    let _ = fivem::runtime::spawn(logger);
    let _ = fivem::runtime::spawn(task);
}
