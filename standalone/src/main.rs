use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename = "_ExtStruct")]
// struct RefFunc((u8, String));

// #[derive(Debug, Serialize, Deserialize)]
// struct Deferrals {
//     defer: RefFunc,
//     done: RefFunc,
//     handover: RefFunc,
//     #[serde(rename = "presentCard")]
//     present_card: RefFunc,
//     update: RefFunc,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct PlayerConnecting {
//     player_name: String,
//     set_kick_reason: RefFunc,
//     deferrals: Deferrals,
//     // source: String,
// }

// const BYTES: &[u8] = &[
//     147, 174, 208, 177, 208, 184, 209, 130, 208, 186, 208, 190, 208, 184, 208, 189, 199, 26, 11,
//     95, 99, 102, 120, 95, 105, 110, 116, 101, 114, 110, 97, 108, 58, 49, 49, 50, 56, 52, 49, 57,
//     57, 50, 52, 58, 54, 133, 165, 100, 101, 102, 101, 114, 199, 26, 11, 95, 99, 102, 120, 95, 105,
//     110, 116, 101, 114, 110, 97, 108, 58, 49, 49, 50, 56, 52, 49, 57, 57, 50, 52, 58, 49, 164, 100,
//     111, 110, 101, 199, 26, 11, 95, 99, 102, 120, 95, 105, 110, 116, 101, 114, 110, 97, 108, 58,
//     49, 49, 50, 56, 52, 49, 57, 57, 50, 52, 58, 52, 168, 104, 97, 110, 100, 111, 118, 101, 114,
//     199, 26, 11, 95, 99, 102, 120, 95, 105, 110, 116, 101, 114, 110, 97, 108, 58, 49, 49, 50, 56,
//     52, 49, 57, 57, 50, 52, 58, 53, 171, 112, 114, 101, 115, 101, 110, 116, 67, 97, 114, 100, 199,
//     26, 11, 95, 99, 102, 120, 95, 105, 110, 116, 101, 114, 110, 97, 108, 58, 49, 49, 50, 56, 52,
//     49, 57, 57, 50, 52, 58, 51, 166, 117, 112, 100, 97, 116, 101, 199, 26, 11, 95, 99, 102, 120,
//     95, 105, 110, 116, 101, 114, 110, 97, 108, 58, 49, 49, 50, 56, 52, 49, 57, 57, 50, 52, 58, 50,
// ];

/// MessagePack
fn main() {
    // let result: PlayerConnecting = rmp_serde::from_read(BYTES).unwrap();

    // result.set_kick_reason.0.1
    // let string = unsafe { std::str::from_utf8_unchecked(result.set_kick_reason.0 .1.as_slice()) };
    // println!("{:?}", string);

    // println!("{:#?}", result);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename = "_ExtStruct")]
    pub struct ExternRefFunction((i8, serde_bytes::ByteBuf));

    impl ExternRefFunction {
        pub(crate) fn new(name: &str) -> ExternRefFunction {
            let bytes = serde_bytes::ByteBuf::from(name.bytes().collect::<Vec<u8>>());
            ExternRefFunction((10, bytes))
        }
    }

    #[derive(Serialize)]
    struct Pos {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Serialize)]
    struct SpawnPlayer(Pos, ExternRefFunction);

    #[derive(Serialize)]
    struct Shit {
        position: HashMap<String, f32>,
        func: ExternRefFunction,
    }

    let spawn = SpawnPlayer(
        Pos {
            x: 51.0,
            y: 60.0,
            z: 31.0,
        },
        ExternRefFunction::new("cool:3325:1"),
    );

    let mut position = HashMap::new();
    position.insert("x".to_owned(), 51.0);
    position.insert("y".to_owned(), 60.0);
    position.insert("z".to_owned(), 31.0);

    let shit = Shit {
        position,
        func: ExternRefFunction::new("cool:3325:1"),
    };

    println!("{:?}", rmp_serde::to_vec(&shit));
    println!("{:?}", rmp_serde::to_vec_named(&spawn));
}
