#[cfg(feature = "full")]
use serde::de::DeserializeOwned;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ReturnTypes {
    Empty = 0,
    Number,
    String,
    Vector3,
    MsgPack,
    Unk,
}

impl From<u32> for ReturnTypes {
    fn from(val: u32) -> Self {
        match val {
            0 => ReturnTypes::Empty,
            1 => ReturnTypes::Number,
            2 => ReturnTypes::String,
            3 => ReturnTypes::Vector3,
            4 => ReturnTypes::MsgPack,
            _ => ReturnTypes::Unk,
        }
    }
}

#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pad_0: u32,

    pub y: f32,
    pad_1: u32,

    pub z: f32,
    pad_2: u32,
}

#[repr(C)]
pub struct ScrObject {
    pub data: u64,
    pub length: u64,
}

#[cfg(feature = "full")]
pub struct Packed<T: DeserializeOwned> {
    inner: T,
}

#[cfg(feature = "full")]
impl<T: DeserializeOwned> Packed<T> {
    pub fn payload(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

unsafe impl RetVal for () {
    const IDENT: ReturnTypes = ReturnTypes::Empty;

    unsafe fn convert(_: *mut u8) -> Self {
        ()
    }
}

unsafe impl RetVal for f32 {
    const IDENT: ReturnTypes = ReturnTypes::Number;

    unsafe fn convert(bytes: *mut u8) -> Self {
        std::mem::transmute(bytes as usize as u32)
    }
}

unsafe impl RetVal for u32 {
    const IDENT: ReturnTypes = ReturnTypes::Number;

    unsafe fn convert(bytes: *mut u8) -> Self {
        bytes as u32
    }
}

unsafe impl RetVal for String {
    const IDENT: ReturnTypes = ReturnTypes::String;

    unsafe fn convert(bytes: *mut u8) -> Self {
        let cstr = std::ffi::CStr::from_ptr(bytes as *const _);
        cstr.to_str().unwrap().to_owned()
    }
}

unsafe impl RetVal for Box<Vector3> {
    const IDENT: ReturnTypes = ReturnTypes::Vector3;

    unsafe fn convert(bytes: *mut u8) -> Self {
        Box::from_raw(bytes as _)
    }
}

unsafe impl RetVal for Box<ScrObject> {
    const IDENT: ReturnTypes = ReturnTypes::MsgPack;

    unsafe fn convert(bytes: *mut u8) -> Self {
        Box::from_raw(bytes as _)
    }
}

#[cfg(feature = "full")]
unsafe impl<T: DeserializeOwned> RetVal for Packed<T> {
    const IDENT: ReturnTypes = ReturnTypes::MsgPack;

    unsafe fn convert(bytes: *mut u8) -> Self {
        let scrobj: Box<ScrObject> = Box::from_raw(bytes as _);
        let bytes = Vec::from_raw_parts(
            scrobj.data as *mut u8,
            scrobj.length as _,
            scrobj.length as _,
        );

        let inner = rmp_serde::from_read_ref(bytes.as_slice()).unwrap();

        Packed { inner }
    }
}

pub unsafe trait RetVal {
    const IDENT: ReturnTypes;

    unsafe fn convert(bytes: *mut u8) -> Self;
}
