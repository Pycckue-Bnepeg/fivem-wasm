use cfx::events::EventScope;
use cfx::ref_funcs::{ExternRefFunction, RefFunction};
use futures::StreamExt;
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

const ANIM_DICT: &str = "amb@code_human_cower@male@exit";
const ANIM_NAME: &str = "exit_flee";

async fn play_animation() {
    use cfx::client::TaskSequenceBuilder;

    TaskSequenceBuilder::new()
        .play_anim(
            ANIM_DICT, ANIM_NAME, 8.0, 8.0, -1, 0, 0.0, false, false, false,
        )
        .await
        .run_and_wait(true)
        .await;
}

async fn listen_to_pongs() {
    #[derive(Debug, Deserialize)]
    struct Pong {
        msg: String,
        counter: u64,
    }

    let events = cfx::events::subscribe::<Pong>("server_pong", EventScope::Network);

    futures::pin_mut!(events);

    while let Some(event) = events.next().await {
        let pong = event.payload();

        cfx::log(format!(
            "got a pong from {:?} with message: {:?}",
            event.source(),
            pong
        ));
    }
}

fn set_command_handler() {
    #[derive(Serialize)]
    struct Ping {
        req: String,
    }

    #[derive(Deserialize)]
    struct Command {
        _source: u32,
        _arguments: Vec<()>,
        _raw_cmd: String,
    }

    let handler = RefFunction::new(|_: Command| {
        cfx::events::emit_to_server(
            "client_ping",
            Ping {
                req: "pong me please".to_owned(),
            },
        );
    });

    cfx::client::cfx::register_command("wasm_ping", handler, false);
}

#[no_mangle]
pub extern "C" fn _start() {
    const SPAWN_INFO: SpawnInfo = SpawnInfo {
        x: 686.245,
        y: 577.950,
        z: 130.461,

        heading: 0.0,
        idx: 0,
        model: 0x705E61F2,
        skip_fade: false,
    };

    let set_callback =
        cfx::exports::import_function("spawnmanager", "setAutoSpawnCallback").unwrap();
    let spawn_player = cfx::exports::import_function("spawnmanager", "spawnPlayer").unwrap();
    let set_autospawn = cfx::exports::import_function("spawnmanager", "setAutoSpawn").unwrap();
    let force_respawn = cfx::exports::import_function("spawnmanager", "forceRespawn").unwrap();

    let task = async move {
        let on_spawn = RefFunction::new(|spawn_info: Vec<SpawnInfo>| -> Vec<bool> {
            cfx::log(format!("player spawned: {:?}", spawn_info));

            cfx::client::ped::set_ped_default_component_variation(
                cfx::client::player::player_ped_id(),
            );

            let task = async {
                play_animation().await;
                cfx::log("play_animation()");
            };

            let _ = cfx::runtime::spawn(task);

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

    cfx::client::cfx::set_discord_app_id("843983771278901279");

    let logger = async {
        let wrapper = || {
            let player = cfx::client::player::player_ped_id();
            let camera = cfx::client::cam::get_gameplay_cam_coord();
            let player_pos = cfx::client::entity::get_entity_coords(player, false);
            let id = cfx::client::player::player_id();
            let name = cfx::client::player::get_player_name(id)?;

            cfx::log(format!(
                "player {} camera: {:?} player: {:?} name: {:?} id: {}",
                player, camera, player_pos, name, id
            ));

            Some(())
        };

        loop {
            wrapper();
            cfx::runtime::sleep_for(Duration::from_secs(5)).await;
        }
    };

    let _ = cfx::runtime::spawn(logger);
    let _ = cfx::runtime::spawn(task);

    // commands
    set_command_handler();
    let _ = cfx::runtime::spawn(listen_to_pongs());
}
