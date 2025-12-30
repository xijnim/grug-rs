use grug_rs::{
    Grug,
    grug_value::{Arguments, GrugValue},
};

use anyhow::Result;
use grug_rs_proc_macro::game_function;

fn main() -> Result<()> {
    // Initializes grug
    let grug = Grug::new(
        "./examples/mod_api.json",
        "./examples/mods",
        "./examples/mods_dll",
        1000,
    )?;

    let mut args = Arguments::new(vec![GrugValue::String("hello, world".to_string())]);
    loop {
        grug.activate_on_function("World", "on_update", &mut Arguments::empty())?;
        grug.activate_on_function("World", "on_argument_test", &mut args)?;
    }
    Ok(())
}

#[game_function]
fn println(message: String) {
    println!("{message}");
}

#[game_function]
fn println_int(message: i32) {
    println!("{message}");
}
