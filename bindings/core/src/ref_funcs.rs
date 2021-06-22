use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::wasm_impl::ref_funcs::{canonicalize_ref, HANDLERS, REF_IDX};

pub(crate) struct InnerRefFunction {
    pub(crate) _idx: u32,
    pub(crate) func: Box<dyn Fn(&[u8], &RefCell<Vec<u8>>)>,
    pub(crate) refs: Cell<i32>,
}

impl InnerRefFunction {
    pub(crate) fn handle(&self, input: &[u8], output: &RefCell<Vec<u8>>) {
        (self.func)(input, output);
    }
}

/// External ref function (from exports or events).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "_ExtStruct")]
pub struct ExternRefFunction((i8, serde_bytes::ByteBuf));

impl ExternRefFunction {
    pub(crate) fn new(name: &str) -> ExternRefFunction {
        let bytes = serde_bytes::ByteBuf::from(name.bytes().collect::<Vec<u8>>());
        ExternRefFunction((10, bytes))
    }

    pub(crate) fn name(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0 .1) }
    }

    /// Invoke the function.
    pub fn invoke<Out, In>(&self, args: In) -> Option<Out>
    where
        In: Serialize,
        Out: DeserializeOwned,
    {
        crate::invoker::invoke_ref_func(&self, args)
    }
}

#[derive(Clone)]
pub struct RefFunction {
    name: String,
    inner: Rc<InnerRefFunction>,
}

impl RefFunction {
    /// Creates a new ref function that can be passed to CitizenFX.
    ///
    /// # Example
    /// ```rust,ignore
    /// let export = RefFunction::new(|vector: Vec<Vector>| {
    ///     if let Some(vec) = vector.get(0) {
    ///         let length = (vec.x.powi(2) + vec.y.powi(2) + vec.z.powi(2)).sqrt();
    ///         return vec![length];
    ///     }
    ///
    ///     vec![0.0]
    /// });
    /// ```
    pub fn new<Handler, Input, Output>(handler: Handler) -> RefFunction
    where
        Handler: Fn(Input) -> Output + 'static,
        Input: DeserializeOwned,
        Output: Serialize,
    {
        let idx = REF_IDX.with(|idx| {
            let mut idx = idx.borrow_mut();
            *idx += 1;
            *idx
        });

        let name = canonicalize_ref(idx);

        let func = move |input: &[u8], out_buf: &RefCell<Vec<u8>>| {
            let input = rmp_serde::decode::from_read(input).unwrap();
            let out = handler(input);
            {
                let mut out_buf = out_buf.borrow_mut();

                unsafe {
                    out_buf.set_len(0);
                }

                let _ = rmp_serde::encode::write_named(&mut *out_buf, &out);
            }
        };

        let inner = InnerRefFunction {
            _idx: idx,
            func: Box::new(func),
            refs: Cell::new(0),
        };

        let inner = Rc::new(inner);

        HANDLERS.with(|handlers| {
            let mut handlers = handlers.borrow_mut();
            handlers.insert(idx, inner.clone());
        });

        RefFunction { name, inner }
    }

    /// Same as [`RefFunction::new`] but doesn't se/deserialize output/input value.
    pub fn new_raw<Handler>(handler: Handler) -> RefFunction
    where
        Handler: Fn(&[u8]) -> Vec<u8> + 'static,
    {
        let idx = REF_IDX.with(|idx| {
            let mut idx = idx.borrow_mut();
            *idx += 1;
            *idx
        });

        let name = canonicalize_ref(idx);

        let func = move |input: &[u8], out_buf: &RefCell<Vec<u8>>| {
            let out = handler(input);
            {
                let mut out_buf = out_buf.borrow_mut();

                unsafe {
                    out_buf.set_len(0);
                }

                out_buf.extend(out.iter());
            }
        };

        let inner = InnerRefFunction {
            _idx: idx,
            func: Box::new(func),
            refs: Cell::new(0),
        };

        let inner = Rc::new(inner);

        HANDLERS.with(|handlers| {
            let mut handlers = handlers.borrow_mut();
            handlers.insert(idx, inner.clone());
        });

        RefFunction { name, inner }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Converts [`RefFunction`] into [`ExternRefFunction`] that can be serialized by serde.
    pub fn as_extern_ref_func(&self) -> ExternRefFunction {
        ExternRefFunction::new(self.name())
    }
}
