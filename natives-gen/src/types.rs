use crate::parser::{parse_file, Argument, FuncExec};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Default, Clone)]
pub struct CfxType {
    pub name: String,
    pub native_type: String,
    pub parent: Option<String>,
}

#[derive(Debug)]
pub struct RustType {
    pub name: String,
    pub convert: Option<String>,
    pub may_be_default: bool,
}

impl CfxType {
    pub fn is_type(&self, types: &HashMap<String, CfxType>, name: &str) -> bool {
        if self.name == name {
            return true;
        }

        self.parent
            .as_ref()
            .and_then(|ty| Some((ty, types.get(ty)?)))
            .map(|(_, parent)| parent.is_type(types, name))
            .unwrap_or(false)
    }

    pub fn is_primitive(&self) -> bool {
        self.name == "Any"
            || self.name == "uint"
            || self.name == "int"
            || self.name == "Hash"
            || self.name == "charPtr"
            || self.name == "float"
            || self.name == "vector3"
            || self.name == "BOOL"
    }

    pub fn is_ptr(&self) -> bool {
        self.name.ends_with("Ptr")
    }

    pub fn is_float(&self) -> bool {
        self.native_type == "float"
    }
}

pub fn convert_type(ty: &CfxType, in_ret: bool) -> RustType {
    match ty.native_type.as_str() {
        "string" => {
            if in_ret {
                RustType {
                    name: "String".to_owned(),
                    convert: None,
                    may_be_default: false,
                }
            } else {
                RustType {
                    name: "impl cfx_core::types::AsCharPtr".to_owned(),
                    convert: Some("as_char_ptr().into()".to_owned()),
                    may_be_default: false,
                }
            }
        }

        "int" => RustType {
            name: "i32".to_owned(),
            convert: None,
            may_be_default: true,
        },

        "long" => RustType {
            name: "i64".to_owned(),
            convert: None,
            may_be_default: true,
        },

        "float" => RustType {
            name: "f32".to_owned(),
            convert: None,
            may_be_default: true,
        },

        "vector3" => RustType {
            name: "cfx_core::types::Vector3".to_owned(),
            convert: None,
            may_be_default: true,
        },

        "func" => {
            if in_ret {
                RustType {
                    name: "cfx_core::ref_funcs::ExternRefFunction".to_owned(),
                    convert: None,
                    may_be_default: false,
                }
            } else {
                RustType {
                    name: "cfx_core::ref_funcs::RefFunction".to_owned(),
                    convert: None,
                    may_be_default: false,
                }
            }
        }

        "object" => {
            if in_ret {
                RustType {
                    name: "cfx_core::types::Packed<Ret>".to_owned(),
                    convert: None,
                    may_be_default: false,
                }
            } else {
                RustType {
                    name: "impl serde::Serialize".to_owned(),
                    convert: Some("to_message_pack().as_slice().into()".to_owned()),
                    may_be_default: false,
                }
            }
        }

        "bool" => RustType {
            name: "bool".to_owned(),
            convert: None,
            may_be_default: true,
        },

        _ => RustType {
            name: "()".to_owned(),
            convert: None,
            may_be_default: true,
        },
    }
}

pub fn find_type<'a>(map: &'a HashMap<String, CfxType>, name: &str) -> Option<(bool, &'a CfxType)> {
    if name != "charPtr" && name.ends_with("Ptr") {
        let fixed = name.strip_suffix("Ptr")?;
        Some((true, map.get(fixed)?))
    } else {
        Some((false, map.get(name)?))
    }
}

fn format_types(types: Vec<FuncExec>) -> HashMap<String, CfxType> {
    let mut types = types.iter();
    let mut cfx_type: Option<CfxType> = None;
    let mut formated = HashMap::new();

    loop {
        if let Some(ty) = types.next() {
            if ty.name == "type" {
                if let Some(cfx_type) = cfx_type.take() {
                    formated.insert(cfx_type.name.clone(), cfx_type);
                }

                if let Argument::String(str) = &ty.argument {
                    cfx_type = Some(CfxType {
                        name: str.to_owned(),
                        ..Default::default()
                    });
                }

                continue;
            } else {
                if let Some(cfx_type) = cfx_type.as_mut() {
                    match ty.name.as_str() {
                        "nativeType" => {
                            if cfx_type.native_type.len() == 0 {
                                cfx_type.native_type = ty.argument.to_string().to_ascii_lowercase();
                            }
                        }

                        "extends" => {
                            cfx_type.parent = Some(ty.argument.to_string());
                        }

                        "subType" => {
                            cfx_type.native_type = ty.argument.to_string().to_ascii_lowercase()
                        }

                        _ => (),
                    }
                }
            }
        } else {
            if let Some(cfx_type) = cfx_type.take() {
                formated.insert(cfx_type.name.clone(), cfx_type);
            }

            break;
        }
    }

    formated
}

pub fn types_from_file(file: &PathBuf) -> HashMap<String, CfxType> {
    let types = parse_file(file);
    format_types(types)
}
