This is a high level crate that allows you to define your own zsh module.

# Getting started
To get started, first, you need to create library, not an executable. Then, change your crate
type to `"cdylib"` on your `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]
```

# Boilerplate
On your `lib.rs`, you need to put a [`impl_hooks`] macro call, alongside a `setup` function
with the following signature:
```rust
use zsh_module::{ Module, ModuleBuilder }

zsh_module::impl_hooks!();

fn setup() -> Result<Module, ()> {
   todo!() 
}
```

# Defining [`Actions`]
The main point part of crating a module is implementing [`Actions`]. Here's an example module:
```rust
use zsh_module::{ Module, ModuleBuilder, Actions, Result }

zsh_module::impl_hooks!();

struct Greeter;

impl Actions for Greeter {
    fn boot(mut &self) -> Result<()> {
        println!("Hello, everyone!");
        Ok(())
    }
    fn cleanup(&mut self) -> Result<()> {
        println!("Bye, everyone!");
        Ok(())
    }
}

fn setup() -> Result<Module> {
    let module = ModuleBuilder::new()
        .build(Greeter);
    Ok(module)
}
```

# Installing
When your module is ready, copy your shared library to your distribution's zsh module folder,
without the `lib` prefix.
On Arch Linux, it's `/usr/lib/zsh/<zsh-version>/zsh/`.

That is it!

If everything went fine, you can load it in zsh using the following command:
```zsh
zmodload zsh/<module-name>
```
