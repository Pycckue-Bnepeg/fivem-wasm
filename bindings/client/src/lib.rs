use serde::Serialize;

pub mod natives;

pub mod events {
    use fivem_core::events::Event;
    use futures::Stream;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ClientGameTypeStart {
        pub resource_name: String,
    }

    pub fn client_game_type_start() -> impl Stream<Item = Event<ClientGameTypeStart>> {
        fivem_core::events::subscribe(
            "onClientGameTypeStart",
            fivem_core::events::EventScope::Local,
        )
    }
}

use fivem_core::runtime::sleep_for;
use natives::{player::*, streaming::*, task::*};
use std::time::Duration;

#[derive(Debug)]
pub struct TaskSequenceBuilder {
    sequence: i32,
}

impl TaskSequenceBuilder {
    pub fn new() -> TaskSequenceBuilder {
        let mut sequence = 0;
        open_sequence_task(&mut sequence);

        TaskSequenceBuilder { sequence }
    }

    pub async fn play_anim<Dict, Name>(
        self,
        anim_dict: Dict,
        anim_name: Name,
        blend_in_speed: f32,
        blend_out_speed: f32,
        duration: i32,
        flag: i32,
        playback_rate: f32,
        lock_x: bool,
        lock_y: bool,
        lock_z: bool,
    ) -> Self
    where
        Dict: AsRef<str>,
        Name: AsRef<str>,
    {
        let anim_dict = anim_dict.as_ref();
        let anim_name = anim_name.as_ref();

        request_anim_dict(anim_dict);

        while !has_anim_dict_loaded(anim_dict) {
            sleep_for(Duration::from_millis(5)).await;
        }

        task_play_anim(
            0,
            anim_dict,
            anim_name,
            blend_in_speed,
            blend_out_speed,
            duration,
            flag,
            playback_rate,
            lock_x,
            lock_y,
            lock_z,
        );

        self
    }

    pub fn run(self, clear_tasks: bool) {
        let ped = player_ped_id();

        if clear_tasks {
            clear_ped_tasks(ped);
        }

        close_sequence_task(self.sequence);
        task_perform_sequence(ped, self.sequence);
    }
}

impl Drop for TaskSequenceBuilder {
    fn drop(&mut self) {
        clear_sequence_task(&mut self.sequence);
    }
}

pub fn emit_net<T: Serialize>(event_name: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        natives::cfx::trigger_server_event_internal(
            event_name,
            payload.as_slice(),
            payload.len() as _,
        );
    }
}
