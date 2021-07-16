pub(crate) mod natives;
pub(crate) mod parser;
pub(crate) mod rpcs;
pub(crate) mod types;

use dotenv::dotenv;
use itertools::Itertools;
use natives::*;
use rpcs::{extend_natives_with_rpcs, rpcs_from_file};
use std::path::PathBuf;
use types::types_from_file;

fn exists_or_panic(path: &PathBuf) {
    if !path.exists() {
        panic!("{:#?} does not exists", path);
    }
}

fn main() {
    dotenv().ok();
    let return_style = ReturnStyle::UnwrapOrDefault;

    let folder: PathBuf = std::env::var("EXT_NATIVES")
        .unwrap_or("E:/sources/c/fivem-fork/ext/natives".to_string())
        .into();
    exists_or_panic(&folder);
    let codegen_types_path = folder.join("codegen_types.lua");
    let rpc_spec_natives_path = folder.join("rpc_spec_natives.lua");
    let natives_cfx_path = folder.join("inp/natives_cfx.lua");
    let natives_global_path = folder.join("inp/natives_global.lua");
    exists_or_panic(&codegen_types_path);
    exists_or_panic(&rpc_spec_natives_path);
    exists_or_panic(&natives_cfx_path);
    exists_or_panic(&natives_global_path);

    let mut types = types_from_file(&codegen_types_path);
    let rpcs = rpcs_from_file(&rpc_spec_natives_path);

    let natives_cfx = natives_from_file(&natives_cfx_path, ApiSet::Server);

    let natives_global = natives_from_file(&natives_global_path, ApiSet::Client);

    let natives = natives_global.into_iter().chain(natives_cfx);
    let mut sets = natives.into_group_map_by(|native| native.apiset);

    if let Some(shared) = sets.remove(&ApiSet::Shared) {
        if let Some(client) = sets.get_mut(&ApiSet::Client) {
            client.extend_from_slice(&shared);
        }

        if let Some(server) = sets.get_mut(&ApiSet::Server) {
            server.extend_from_slice(&shared);
        }
    }

    extend_natives_with_rpcs(rpcs, &mut sets, &types);

    for (apiset, natives) in sets {
        make_natives_for_set(apiset, &mut types, natives, return_style);
    }
}
