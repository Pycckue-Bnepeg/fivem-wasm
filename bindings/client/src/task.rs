use crate::natives::{player::*, streaming::*, task::*};
use cfx_core::runtime::sleep_for;
use std::time::Duration;

#[derive(Debug)]
pub struct TaskSequenceBuilder {
    sequence: i32,
    repeat: bool,
}

impl TaskSequenceBuilder {
    pub fn new() -> TaskSequenceBuilder {
        let mut sequence = 0;
        open_sequence_task(&mut sequence);

        TaskSequenceBuilder {
            sequence,
            repeat: false,
        }
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

    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn run(self, clear_tasks: bool) {
        let ped = player_ped_id();

        if clear_tasks {
            clear_ped_tasks(ped);
        }

        set_sequence_to_repeat(self.sequence, self.repeat);
        close_sequence_task(self.sequence);
        task_perform_sequence(ped, self.sequence);
    }

    pub async fn run_and_wait(self, clear_tasks: bool) {
        self.run(clear_tasks);

        sleep_for(Duration::from_millis(10)).await;

        let ped = player_ped_id();

        while crate::natives::task::get_sequence_progress(ped) != -1 {
            sleep_for(Duration::from_millis(10)).await;
        }
    }
}

impl Drop for TaskSequenceBuilder {
    fn drop(&mut self) {
        clear_sequence_task(&mut self.sequence);
    }
}
