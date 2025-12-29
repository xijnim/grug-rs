//! Safe bindings for [grug](https://github.com/grug-lang/grug)

//! # Basic Usage
//! ```rs
//! use grug_rs::Grug;

//! use anyhow::Result;
//! use grug_rs_proc_macro::game_function;

//! fn main() -> Result<()> {
//!     // Initializes grug
//!     let grug = Grug::new(
//!         "./mod_api.json",
//!         "./mods",
//!         "./mods_dll",
//!         1000,
//!     )?;

//!     loop {
//!         grug.activate_on_function("World", "on_update")?;
//!     }
//! }

//! #[game_function]
//! fn println(message: String) {
//!     println!("{message}");
//! }
//! ```
//! Use this as your `main.rs`.

//! You will need to create a `mods` directory and a `mod_api.json`.

//! Inside `mod_api.json` put this:
//! ```json
//! {
//!   "entities": {
//!     "World": {
//!       "description": "Let's print in here",
//!       "on_functions": {
//!         "on_update": {
//!           "description": "Called every tick"
//!         }
//!       }
//!     }
//!   },
//!   "game_functions": {
//!     "println": {
//!       "description": "Prints a string with a new line",
//!       "arguments": [
//!         {
//!           "name": "msg",
//!           "type": "string"
//!         }
//!       ]
//!     }
//!   }
//! }
//! ```

//! Inside of mods create an `about.json` and put this:
//! ```json
//! {
//!     "name": "hello_world",
//!     "version": "1.0.0",
//!     "game_version": "1.0.0",
//!     "author": "YOUR NAME HERE"
//! }
//! ```

//! And create a file called `hello-World.grug` and put:
//! ```grug
//! on_update() {
//!     println("Hello world!", 10)
//! }
//! ```

//! Then run your program!

//! If there are errors with unable to find symbols you might have to create a basic `build.rs` that looks like this:
//! ```rs
//! fn main() {
//!     println!("cargo:rustc-link-arg=-rdynamic");
//! }
//! ```

pub mod mod_api_type;
mod to_string_wrapper;

use std::{
    collections::HashMap,
    ffi::{CStr, CString, OsString, c_char, c_void},
    fs::read_to_string,
    path::PathBuf,
    ptr::null_mut,
    slice::from_raw_parts,
};

use grug_sys::*;
use serde_json::from_str;
use thiserror::Error;

use crate::{mod_api_type::ModAPI, to_string_wrapper::ToStringWrapper};

/// Errors from Grug
#[derive(Error, Debug)]
pub enum GrugError {
    #[error("Failed to initialize Grug: `{error}`")]
    Init { error: String },
    #[error("Failed to read: `{path}`: `{error}`")]
    ReadModAPI { path: PathBuf, error: String },
    #[error("Failed to deserialize `{path}`: `{error}`")]
    Deserialize { path: PathBuf, error: String },
    #[error("`{function_name}` is not a on_function")]
    NotAnOnFunction { function_name: String },
    #[error("`{entity_name}` is not an entity")]
    NotAnEntity { entity_name: String },
    #[error("Grug failed to load: `{name}` in `{path}`")]
    FileLoading { name: String, path: String },
    #[error("Grug regenerating error: `{error}`")]
    Regenerating { error: String },
    #[error("Grug function not defined")]
    UndefinedFunction,
}

pub type ErrorHandler = fn(String, grug_runtime_error_type, String, String);

unsafe extern "C" fn runtime_error_handler(
    reason: *const c_char,
    _type_: grug_runtime_error_type,
    on_fn_name: *const c_char,
    on_fn_path: *const c_char,
) {
    // Convert inputs safely
    let reason = if !reason.is_null() {
        unsafe { CStr::from_ptr(reason).to_string_lossy() }
    } else {
        "<no reason>".into()
    };

    let fn_name = if !on_fn_name.is_null() {
        unsafe { CStr::from_ptr(on_fn_name).to_string_lossy() }
    } else {
        "<unknown fn>".into()
    };

    let fn_path = if !on_fn_path.is_null() {
        unsafe { CStr::from_ptr(on_fn_path).to_string_lossy() }
    } else {
        "<unknown path>".into()
    };

    eprintln!(
        "Grug runtime error: {}\n  at {} ({})",
        reason, fn_name, fn_path
    );
}

pub struct Grug {
    #[allow(dead_code)]
    mod_api: ModAPI, // Here just in case
    entities: HashMap<String, HashMap<String, usize>>,
}

impl Grug {
    /// Initializes grug for usage.
    /// You should only do this once or bad things will happen.
    ///
    /// # Example
    /// ```rs
    /// let grug = Grug::new(
    ///     "./examples/mod_api.json",
    ///     "./examples/mods",
    ///     "./examples/mods_dll",
    ///     1000,
    /// ).unwrap();
    /// ```
    pub fn new<P1, P2, P3>(
        // error_handler: ErrorHandler,
        mod_api_path: P1,
        mods_folder: P2,
        mods_dll_folder: P3,
        timeout_ms: u64,
    ) -> Result<Self, GrugError>
    where
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
        P3: Into<PathBuf>,
    {
        let mod_api_path: PathBuf = mod_api_path.into();
        let mods_folder: PathBuf = mods_folder.into();
        let mods_dll_folder: PathBuf = mods_dll_folder.into();

        assert!(mod_api_path.is_file()); // Ensure that it's a file to begin with
        assert!(mod_api_path.extension().is_some()); // Ensure it has an extension
        assert_eq!(
            mod_api_path.extension().unwrap().to_os_string(),
            OsString::from("json".to_string())
        ); // Ensure that it's a json extension

        assert!(!mods_folder.is_file()); // Ensure it's a folder

        // We need to get the on function count
        let mod_api_json = read_to_string(&mod_api_path).map_err(|x| GrugError::ReadModAPI {
            path: mod_api_path.clone(),
            error: x.to_string().clone(),
        })?;
        let mod_api: ModAPI = from_str(&mod_api_json).map_err(|x| GrugError::Deserialize {
            path: mod_api_path.clone(),
            error: x.to_string(),
        })?;

        let entities = mod_api
            .entities
            .iter()
            .map(|(name, data)| {
                let mut i = 0;
                (
                    name.clone(),
                    data.on_functions
                        .keys()
                        .map(|k| {
                            let return_val = (k.clone(), i);
                            i += 1;
                            return_val
                        })
                        .collect(),
                )
            })
            .collect();

        // Initialize grug
        let result = unsafe {
            grug_init(
                Some(runtime_error_handler),
                CString::new(mod_api_path.as_os_str().to_string_lossy().to_string())
                    .unwrap()
                    .as_ptr(),
                CString::new(mods_folder.as_os_str().to_string_lossy().to_string())
                    .unwrap()
                    .as_ptr(),
                CString::new(mods_dll_folder.as_os_str().to_string_lossy().to_string())
                    .unwrap()
                    .as_ptr(),
                timeout_ms,
            )
        };

        if result {
            #[allow(static_mut_refs)]
            let error = unsafe { grug_error }; // SAFETY: This implements the copy trait so it's safe to use
            return Err(GrugError::Init {
                error: error.msg.to_string(),
            });
        }

        Ok(Self { mod_api, entities })
    }

    /// Regenerates modified mods
    pub fn regenerate_modified_mods(&self) -> Result<(), GrugError> {
        let failed = unsafe { grug_regenerate_modified_mods() };

        if failed {
            #[allow(static_mut_refs)]
            let error = unsafe { grug_error }; // SAFETY: This implements the copy trait so it's safe to use
            if unsafe { grug_loading_error_in_grug_file } {
                return Err(GrugError::FileLoading {
                    name: error.msg.to_string(),
                    path: error.path.to_string(),
                });
            } else {
                return Err(GrugError::Regenerating {
                    error: error.msg.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Activates an `on_function` on a given `entity`
    ///
    /// Automatically calls `regenerate_modified_mods`
    ///
    /// # Example
    /// ```rs
    /// grug.activate_on_function("World", "on_update").unwrap();
    /// ```
    pub fn activate_on_function<S1: ToString, S2: ToString>(
        &self,
        entity_name: S1,
        on_function_name: S2,
    ) -> Result<(), GrugError> {
        self.regenerate_modified_mods()?;

        let on_functions = self.entities.get(&entity_name.to_string());

        if on_functions.is_none() {
            return Err(GrugError::NotAnEntity {
                entity_name: entity_name.to_string(),
            });
        }

        let index = on_functions.unwrap().get(&on_function_name.to_string());

        if index.is_none() {
            return Err(GrugError::NotAnOnFunction {
                function_name: on_function_name.to_string(),
            });
        }

        let index = *index.unwrap();

        let files = self.get_files_by_entity_type(entity_name);

        for file in files {
            unsafe { file.run_on_function(index, null_mut())? }; // TODO: Replace null_mut() with actual global variables
        }

        Ok(())
    }

    // This is only self because we want to ensure grug is initialized
    pub fn get_files_by_entity_type<S: ToString>(&self, name: S) -> Vec<GrugFile> {
        let name = name.to_string();

        #[allow(static_mut_refs)]
        let mods = unsafe { grug_mods }; // SAFETY: This implements the copy trait so it's safe to use
        let mods = unsafe { from_raw_parts(mods.dirs, mods.dirs_size) };

        let mut return_files = vec![];

        for mod_ in mods.iter() {
            let files = unsafe { from_raw_parts(mod_.files, mod_.files_size) };
            for file in files {
                let mod_entity_name = unsafe {
                    CStr::from_ptr(file.entity_type)
                        .to_string_lossy()
                        .into_owned()
                };
                if mod_entity_name == name {
                    return_files.push(GrugFile::new(*file));
                }
            }
        }

        return_files
    }
}

pub struct GrugFile {
    pub inner: grug_file,
}

impl GrugFile {
    pub fn new(file: grug_file) -> Self {
        Self { inner: file }
    }

    /// # SAFETY
    /// Will segfault if you put an invalid index.
    pub unsafe fn run_on_function(
        &self,
        index: usize,
        mut arguments: *mut c_void,
    ) -> Result<(), GrugError> {
        let ptr = self.inner.on_fns as *mut unsafe extern "C" fn(*mut *mut c_void);
        let func = unsafe { from_raw_parts(ptr, index + 1) }.last();

        if func.is_none() {
            // Ensure the function actually has a definition
            return Err(GrugError::UndefinedFunction);
        }

        unsafe {
            func.unwrap()(&mut arguments as *mut _);
        }

        Ok(())
    }
}
