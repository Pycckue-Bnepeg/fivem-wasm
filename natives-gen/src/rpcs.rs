use std::collections::HashMap;

use crate::{
    natives::{ApiSet, CfxNative},
    parser::{parse_file, Argument, FuncExec},
    types::CfxType,
};

#[derive(Debug)]
pub enum RpcType {
    Context,
    Entity,
    Object,
}

#[derive(Debug)]
pub struct Rpc {
    name: String,
    ty: RpcType,
    getter: Option<String>,
}

fn format_rpcs(rpcs: Vec<FuncExec>) -> Vec<Rpc> {
    rpcs.into_iter()
        .filter_map(|rpc| {
            let ty = match rpc.name.as_str() {
                "context_rpc" => RpcType::Context,
                "entity_rpc" => RpcType::Entity,
                "object_rpc" => RpcType::Object,
                _ => return None,
            };

            Some(Rpc {
                ty,
                name: rpc.argument.to_string(),
                getter: rpc.opt_arg.and_then(|arg| match arg {
                    Argument::String(str) => Some(str),
                    Argument::Table(table) => Some(table.get(0)?.0.clone()),
                }),
            })
        })
        .collect()
}

pub fn rpcs_from_file(file: &str) -> Vec<Rpc> {
    let rpcs = parse_file(file);
    format_rpcs(rpcs)
}

fn find_native(
    name: &str,
    set: ApiSet,
    natives: &HashMap<ApiSet, Vec<CfxNative>>,
) -> Option<CfxNative> {
    natives.get(&set).and_then(|set| {
        set.iter()
            .find(|native| &native.name == name)
            .map(|n| n.clone())
    })
}

fn find_client_native(name: &str, natives: &HashMap<ApiSet, Vec<CfxNative>>) -> Option<CfxNative> {
    find_native(name, ApiSet::Client, natives)
}

fn find_server_native(name: &str, natives: &HashMap<ApiSet, Vec<CfxNative>>) -> Option<CfxNative> {
    find_native(name, ApiSet::Server, natives)
}

fn append_native_to_set(native: CfxNative, natives: &mut HashMap<ApiSet, Vec<CfxNative>>) {
    if let Some(set) = natives.get_mut(&ApiSet::Server) {
        set.push(native);
    }
}

macro_rules! is_vector {
    (@ $what:expr, $idx:expr, $types:expr) => {
        $what
            .get($idx)
            .and_then(|(_, ty)| $types.get(ty))
            .map(|ty| ty.is_float())
            .unwrap_or(false)
    };

    ($what:expr, $idx_start:expr, $types:expr) => {
        is_vector!(@ $what, $idx_start, $types)
        && is_vector!(@ $what, $idx_start + 1, $types)
        && is_vector!(@ $what, $idx_start + 2, $types)
    };
}

fn make_server_getter(
    rpc: Rpc,
    native: &CfxNative,
    natives: &mut HashMap<ApiSet, Vec<CfxNative>>,
    types: &HashMap<String, CfxType>,
) {
    if let Some(getter) = &rpc.getter {
        if find_server_native(&getter, natives).is_some() {
            return;
        }

        let mut ctx_ty = None;

        for (_, ty) in native
            .arguments
            .iter()
            .filter_map(|(name, ty)| Some((name, types.get(ty)?)))
        {
            if ty.is_type(types, "Entity") {
                ctx_ty = Some("Entity");
            } else if ty.is_type(types, "Player") {
                ctx_ty = Some("Player");
            } else if !ty.is_primitive() {
                if ty.is_ptr() {
                    ctx_ty = Some("ObjDel");
                } else {
                    ctx_ty = Some("ObjRef");
                }
            }

            if ctx_ty.is_some() {
                break;
            }
        }

        if let Some(ctx_ty) = ctx_ty {
            let mut ret_ty = native
                .arguments
                .get(1)
                .and_then(|(_, ret_ty)| types.get(ret_ty).cloned())
                .map(|ty| ty.name)
                .expect("Are you sure that it is a setter?");

            if native.arguments.len() >= 4 && is_vector!(native.arguments, 1, types) {
                ret_ty = "Vector3".to_owned();
            }

            let arg_ty = match ctx_ty {
                "Entity" => "Entity",
                "Player" => "Player",
                "ObjRef" => "int",
                _ => "void",
            };

            let native = CfxNative {
                hash: joaat::hash_ascii_lowercase(getter.to_ascii_lowercase().as_bytes()) as u64,
                name: getter.clone(),
                apiset: ApiSet::Server,
                arguments: vec![("this".to_owned(), arg_ty.to_owned())],
                game: native.game.clone(),
                namespace: Some("CFX".to_owned()),
                returns: ret_ty,
                doc: native.doc.clone(),
            };

            append_native_to_set(native, natives);
        } else {
            panic!("Context RPC natives are required to have a context.");
        }
    }
}

pub fn extend_natives_with_rpcs(
    rpcs: Vec<Rpc>,
    natives: &mut HashMap<ApiSet, Vec<CfxNative>>,
    types: &HashMap<String, CfxType>,
) {
    for rpc in rpcs {
        if let Some(mut native) = find_client_native(&rpc.name, natives) {
            native.apiset = ApiSet::Server;
            native.namespace = Some("CFX".to_owned());

            match &rpc.ty {
                RpcType::Context => native.returns = "void".to_owned(),
                RpcType::Entity => native.returns = "Entity".to_owned(),
                RpcType::Object => (),
            }

            if let RpcType::Context = rpc.ty {
                make_server_getter(rpc, &native, natives, types);
            }

            append_native_to_set(native, natives);
        }
    }
}
