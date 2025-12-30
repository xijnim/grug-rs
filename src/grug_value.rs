use std::{
    any::Any,
    collections::HashMap,
    ffi::{CString, c_void},
    marker::PhantomData,
};

use crate::OpaqueGrugType;

pub struct CustomValue<'a> {
    raw: *mut c_void,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a> CustomValue<'a> {
    pub fn new<T: Any + 'static>(value: &'a mut T) -> Self {
        Self {
            raw: value as *mut T as *mut c_void,
            _marker: PhantomData,
        }
    }
}

pub enum GrugValue<'a> {
    String(String),
    I32(i32),
    F32(f32),
    Bool(bool),
    Custom(CustomValue<'a>),
}

impl<'a> GrugValue<'a> {
    pub fn custom<T: Any + 'static>(value: &'a mut T) -> Self {
        Self::Custom(CustomValue::new(value))
    }
}

/// Arguments to a grug function
///
/// # Example
/// ```no_run
/// use grug_rs::{Arguments, Grug, GrugValue};
///
/// # fn main() -> Result<(), grug_rs::GrugError> {
/// let grug: Grug = todo!();
/// let mut args = Arguments::new(vec![GrugValue::String("hello, world".to_string())]);
/// grug.activate_on_function("World", "on_update", &mut Arguments::empty())?;
/// grug.activate_on_function("World", "on_argument_test", &mut args)?;
/// # Ok(())
/// # }
/// ```
pub struct Arguments<'a> {
    pub(crate) values: Vec<GrugValue<'a>>,
    raw_values: Option<Vec<*mut c_void>>,
    opaque_values: Option<Vec<OpaqueGrugType>>,
    stored_c_strings: HashMap<String, CString>,
}

impl<'a> Arguments<'a> {
    pub fn new(values: Vec<GrugValue<'a>>) -> Self {
        Self {
            values,
            raw_values: None,
            opaque_values: None,
            stored_c_strings: HashMap::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            values: vec![],
            raw_values: None,
            opaque_values: None,
            stored_c_strings: HashMap::new(),
        }
    }

    pub fn into_raw(&mut self) -> *mut *mut c_void {
        let mut opaque_values = Vec::with_capacity(self.values.len());

        for v in self.values.iter_mut() {
            let raw_ptr = match v {
                GrugValue::String(v) => {
                    let c_string = self
                        .stored_c_strings
                        .entry(v.clone())
                        .or_insert_with(|| CString::new(v.as_str()).unwrap());
                    c_string.as_ptr() as *mut c_void
                }
                GrugValue::I32(v) => v as *mut i32 as *mut c_void,
                GrugValue::F32(v) => v as *mut f32 as *mut c_void,
                GrugValue::Bool(v) => v as *mut bool as *mut c_void,
                GrugValue::Custom(v) => v.raw,
            };

            opaque_values.push(OpaqueGrugType { raw: raw_ptr });
        }

        let mut raw_values = Vec::with_capacity(opaque_values.len());
        for value in opaque_values.iter_mut() {
            raw_values.push(value as *mut OpaqueGrugType as *mut c_void);
        }

        self.opaque_values = Some(opaque_values);
        self.raw_values = Some(raw_values);

        self.raw_values.as_mut().unwrap().as_mut_ptr()
    }
}
