Safe bindings for [grug](https://github.com/grug-lang/grug)

# Basic Usage
```rs
use grug_rs::{Grug, Arguments};

use anyhow::Result;
use grug_rs_proc_macro::game_function;

fn main() -> Result<()> {
    // Initializes grug
    let grug = Grug::new(
        "./mod_api.json",
        "./mods",
        "./mods_dll",
        1000,
    )?;

    loop {
        grug.activate_on_function("World", "on_update", Arguments::empty())?;
    }
}

#[game_function]
fn println(message: String) {
    println!("{message}");
}
```
Use this as your `main.rs`.

You will need to create a `mods` directory and a `mod_api.json`.

Inside `mod_api.json` put this:
```json
{
  "entities": {
    "World": {
      "description": "Let's print in here",
      "on_functions": {
        "on_update": {
          "description": "Called every tick"
        }
      }
    }
  },
  "game_functions": {
    "println": {
      "description": "Prints a string with a new line",
      "arguments": [
        {
          "name": "msg",
          "type": "string"
        }
      ]
    }
  }
}
```

Inside of mods create an `about.json` and put this:
```json
{
    "name": "hello_world",
    "version": "1.0.0",
    "game_version": "1.0.0",
    "author": "YOUR NAME HERE"
}
```

And create a file called `hello-World.grug` and put:
```grug
on_update() {
    println("Hello world!", 10)
}
```

Then run your program!

If there are errors with unable to find symbols you might have to create a basic `build.rs` that looks like this:
```rs
fn main() {
    println!("cargo:rustc-link-arg=-rdynamic");
}
```
