#[cfg(feature = "full")]
use serde::de::DeserializeOwned;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ReturnType {
    Empty = 0,
    Number,
    String,
    Vector3,
    MsgPack,
    Unk,
}

impl From<u32> for ReturnType {
    fn from(val: u32) -> Self {
        match val {
            0 => ReturnType::Empty,
            1 => ReturnType::Number,
            2 => ReturnType::String,
            3 => ReturnType::Vector3,
            4 => ReturnType::MsgPack,
            _ => ReturnType::Unk,
        }
    }
}

#[repr(C)]
pub struct ReturnValue {
    pub rettype: ReturnType,
    pub buffer: u32, // ptr
    pub capacity: u32,
}

impl ReturnValue {
    pub unsafe fn new<T: RetVal>(buf: &Vec<u8>) -> ReturnValue {
        ReturnValue {
            rettype: T::IDENT as _,
            buffer: buf.as_ptr() as _,
            capacity: buf.capacity() as _,
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
    const IDENT: ReturnType = ReturnType::Empty;

    unsafe fn convert(_: &[u8]) -> Self {
        ()
    }
}

unsafe impl RetVal for f32 {
    const IDENT: ReturnType = ReturnType::Number;

    unsafe fn convert(bytes: &[u8]) -> Self {
        (bytes.as_ptr() as *const f32).read()
    }
}

unsafe impl RetVal for u32 {
    const IDENT: ReturnType = ReturnType::Number;

    unsafe fn convert(bytes: &[u8]) -> Self {
        (bytes.as_ptr() as *const u32).read()
    }
}

unsafe impl RetVal for String {
    const IDENT: ReturnType = ReturnType::String;

    unsafe fn convert(bytes: &[u8]) -> Self {
        std::str::from_utf8_unchecked(bytes).to_owned()
    }
}

unsafe impl RetVal for Vector3 {
    const IDENT: ReturnType = ReturnType::Vector3;

    unsafe fn convert(bytes: &[u8]) -> Self {
        (bytes.as_ptr() as *const Vector3).read()
    }
}

#[cfg(feature = "full")]
unsafe impl<T: DeserializeOwned> RetVal for Packed<T> {
    const IDENT: ReturnType = ReturnType::MsgPack;

    unsafe fn convert(bytes: &[u8]) -> Self {
        let inner = rmp_serde::from_read_ref(bytes).unwrap();
        Packed { inner }
    }
}

pub unsafe trait RetVal {
    const IDENT: ReturnType;

    unsafe fn convert(bytes: &[u8]) -> Self;
}
