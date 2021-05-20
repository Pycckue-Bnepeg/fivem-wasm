use itertools::Itertools;

pub(crate) mod natives;
pub(crate) mod parser;
pub(crate) mod rpcs;
pub(crate) mod types;

use natives::*;
use rpcs::{extend_natives_with_rpcs, rpcs_from_file};
use types::types_from_file;

fn main() {
    let return_style = ReturnStyle::UnwrapOrDefault;

    let mut types = types_from_file("E:/sources/c/fivem-fork/ext/natives/codegen_types.lua");
    let rpcs = rpcs_from_file("E:/sources/c/fivem-fork/ext/natives/rpc_spec_natives.lua");

    let natives_cfx = natives_from_file(
        "E:/sources/c/fivem-fork/ext/natives/inp/natives_cfx.lua",
        ApiSet::Server,
    );

    let natives_global = natives_from_file(
        "E:/sources/c/fivem-fork/ext/natives/inp/natives_global.lua",
        ApiSet::Client,
    );

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
