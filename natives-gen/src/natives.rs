use convert_case::{Case, Casing};
use itertools::Itertools;
use std::collections::HashMap;
use std::io::*;

use crate::parser::*;
use crate::types::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ApiSet {
    Server,
    Client,
    Shared,
}

#[derive(Debug, Default)]
struct RustNative {
    name: String,
    hash: u64,
    apiset: ApiSet,
    namespace: Option<String>,
    game: Option<String>,
    arguments: Vec<RustArgument>,
    returns: Option<RustType>,
    doc: Option<String>,
}

impl RustNative {
    fn from_cfx(types: &HashMap<String, CfxType>, cfx: &CfxNative) -> RustNative {
        RustNative {
            name: cfx.name.clone(),
            hash: cfx.hash,
            apiset: cfx.apiset,
            namespace: cfx.namespace.clone(),
            game: cfx.game.clone(),
            arguments: cfx
                .arguments
                .iter()
                .filter_map(|(name, ty)| {
                    let (is_ptr, ty) = find_type(types, ty)?;
                    let ty = convert_type(ty, false);

                    Some(RustArgument {
                        name: name.to_owned(),
                        is_ptr,
                        ty,
                    })
                })
                .collect(),

            returns: find_type(types, &cfx.returns).map(|(_, ty)| convert_type(ty, true)),
            doc: cfx.doc.clone(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CfxNative {
    pub name: String,
    pub hash: u64,
    pub apiset: ApiSet,
    pub namespace: Option<String>,
    pub game: Option<String>,
    pub arguments: Vec<(String, String)>,
    pub returns: String,
    pub doc: Option<String>,
}

#[derive(Debug)]
pub struct RustArgument {
    pub name: String,
    pub is_ptr: bool,
    pub ty: RustType,
}

#[derive(Debug, Clone, Copy)]
pub enum ReturnStyle {
    /// Full unwrap
    Unwrap,
    /// Uses .unwrap_or_default on primitives, strings are option.
    /// If no return just ignores
    /// ```rust,ignore
    /// fn native_primitive(args...) -> i32 {
    ///     invoke(hash, args...).unwrap_or_default()
    /// }
    ///
    /// fn native_str(args...) -> Option<String> {
    ///     invoke(hash, args...).ok()
    /// }
    ///
    /// fn native_no_return_or_void(args...) -> () {
    ///     let _ = invoke::<(), _>(hash, args...);
    /// }
    /// ```
    UnwrapOrDefault,
    /// `fn native(args...) -> Option<T>`
    Option,
    /// `fn native(args...) -> Result<T, InvokeError>`
    Result,
}

impl From<String> for ApiSet {
    fn from(text: String) -> Self {
        match text.as_str() {
            "client" => Self::Client,
            "shared" => Self::Shared,
            _ => Self::Server,
        }
    }
}

impl ToString for ApiSet {
    fn to_string(&self) -> String {
        match self {
            ApiSet::Client => "client".to_owned(),
            ApiSet::Server => "server".to_owned(),
            ApiSet::Shared => "shared".to_owned(),
        }
    }
}

impl Default for ApiSet {
    fn default() -> Self {
        ApiSet::Server
    }
}

pub fn natives_from_file(file: &str, default_set: ApiSet) -> Vec<CfxNative> {
    let params = parse_file(file);
    format_natives(params, default_set)
}

fn format_natives(params: Vec<FuncExec>, default_set: ApiSet) -> Vec<CfxNative> {
    let mut params = params.iter();
    let mut native: Option<CfxNative> = None;
    let mut natives = vec![];

    loop {
        if let Some(param) = params.next() {
            if param.name == "native" {
                if let Some(mut native) = native.take() {
                    if native.hash == 0 {
                        native.hash = joaat::hash_ascii_lowercase(
                            native.name.to_ascii_lowercase().as_bytes(),
                        ) as u64;
                    }

                    natives.push(native);
                }

                if let Argument::String(str) = &param.argument {
                    native = Some(CfxNative {
                        name: str.to_owned(),
                        apiset: default_set,
                        ..Default::default()
                    });
                }

                continue;
            } else {
                if let Some(native) = native.as_mut() {
                    let arg = param.argument.to_string();

                    match param.name.as_str() {
                        "jhash" => (),
                        "hash" => {
                            native.hash = arg
                                .strip_prefix("0x")
                                .and_then(|arg| u64::from_str_radix(&arg, 16).ok())
                                .unwrap_or(0)
                        }
                        "apiset" => native.apiset = ApiSet::from(arg),
                        "ns" => native.namespace = Some(arg),
                        "game" => native.game = Some(arg),
                        "returns" => {
                            native.returns = arg;
                        }

                        "arguments" => {
                            if let Argument::Table(args) = &param.argument {
                                native.arguments = args
                                    .iter()
                                    .filter_map(|(ty, name)| {
                                        Some((
                                            name.get(0)?.to_string().to_case(Case::Snake),
                                            ty.to_owned(),
                                        ))
                                    })
                                    .collect();
                            }
                        }

                        "doc" => native.doc = Some(arg),
                        _ => (),
                    }
                }
            }
        } else {
            break;
        }
    }

    natives
}

fn make_native(native: RustNative, return_style: ReturnStyle) -> String {
    let name = {
        let generics = if let Some(ret) = &native.returns {
            if ret.name.ends_with("<Ret>") {
                "<Ret: serde::de::DeserializeOwned>"
            } else {
                ""
            }
        } else {
            ""
        };

        if native.name.starts_with("0x") {
            format!("_{}{}", native.name.to_ascii_lowercase(), generics)
        } else {
            format!("{}{}", native.name.to_ascii_lowercase(), generics)
        }
    };

    let rettype = native
        .returns
        .as_ref()
        .map(|ret| ret.name.clone())
        .unwrap_or_else(|| "()".to_owned());

    let args = native
        .arguments
        .iter()
        .map(|arg| {
            format!(
                "_{}: {}{}",
                arg.name,
                if arg.is_ptr { "&mut " } else { "" },
                arg.ty.name
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    let body = {
        let args = native
            .arguments
            .iter()
            .map(|arg| {
                if let Some(conv) = &arg.ty.convert {
                    format!("_{}.{}", arg.name, conv)
                } else {
                    format!("_{}.into()", arg.name)
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        let (prefix, turbofish) = if let ReturnStyle::UnwrapOrDefault = return_style {
            if native.returns.is_none() {
                ("let _ = ", "::<(), _>")
            } else {
                ("", "")
            }
        } else {
            ("", "")
        };

        let suffix = match return_style {
            ReturnStyle::Option => ".ok()",
            ReturnStyle::Result => "",
            ReturnStyle::Unwrap => ".unwrap()",
            ReturnStyle::UnwrapOrDefault => {
                if let Some(ret) = native.returns.as_ref() {
                    if ret.may_be_default {
                        ".unwrap_or_default()"
                    } else {
                        ".ok()"
                    }
                } else {
                    ";"
                }
            }
        };

        format!(
            "{}cfx_core::invoker::invoke{}(0x{:X?}, &[{}]){}",
            prefix, turbofish, native.hash, args, suffix,
        )
    };

    let ret = match return_style {
        ReturnStyle::Option => format!("Option<{}>", rettype),
        ReturnStyle::Unwrap => format!("{}", rettype),
        ReturnStyle::Result => format!("Result<{}, cfx_core::invoker::InvokeError>", rettype),
        ReturnStyle::UnwrapOrDefault => {
            if let Some(ret) = native.returns.as_ref() {
                if ret.may_be_default {
                    format!("{}", rettype)
                } else {
                    format!("Option<{}>", rettype)
                }
            } else {
                "()".to_owned()
            }
        }
    };

    let doc = native
        .doc
        .map(|doc| {
            doc.lines()
                .skip(1)
                .map(|line| format!("/// {} \r\n", line))
                .join("")
        })
        .unwrap_or_default();

    format!(
        "{}#[inline] pub fn {}({}) -> {} {{ {} }}",
        doc, name, args, ret, body
    )
}

pub fn make_natives_for_set(
    apiset: ApiSet,
    types: &mut HashMap<String, CfxType>,
    natives: Vec<CfxNative>,
    return_style: ReturnStyle,
) {
    // fix type
    let replace = if let ApiSet::Server = apiset {
        types.insert(
            "Player".to_owned(),
            CfxType {
                name: "Player".to_owned(),
                native_type: "string".to_owned(),
                parent: None,
            },
        )
    } else {
        None
    };

    let mut file =
        std::fs::File::create(format!("bindings/{}/src/natives.rs", apiset.to_string())).unwrap();

    let namespaces = natives
        .iter()
        .sorted_by_key(|native| &native.name)
        .dedup_by(|f, s| f.name == s.name)
        .into_group_map_by(|native| native.namespace.clone().unwrap_or_default());

    for (namespace, natives) in namespaces {
        if namespace.len() > 0 {
            let _ = writeln!(file, "pub mod {} {{", namespace.to_ascii_lowercase());
            let _ = writeln!(file, "use cfx_core::types::ToMessagePack;");
        }

        for native in natives
            .iter()
            .map(|native| RustNative::from_cfx(types, native))
        {
            let _ = writeln!(file, "{}", make_native(native, return_style));
        }

        if namespace.len() > 0 {
            let _ = writeln!(file, "}}");
        }
    }

    if let Some(replace) = replace {
        types.insert("Player".to_owned(), replace);
    }
}
