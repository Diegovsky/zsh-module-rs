<div id="badges">
    <img src="https://img.shields.io/crates/v/zsh-module">
</div>
# Zsh Module
This is a high level crate that allows you to define your own zsh module. It is in a very early state but it can be used to define commands.

In the future, most zsh module functionality will be added. Feel free to send a PR if you would like to add more functionality :)

Features: 
  - [x] Define custom builtin commands
    - [x] Define and query used flags
  - [x] Call into `zsh`
  - [ ] Query environment variables
    - As a workaround: can be done using the `std::os` APIs.
  - [ ] Use zsh's current `stdin` fd.
    - You can use `std::io::stdin`, but it can break in specific ocasions.
  - [ ] Define custom builtin math functions
  - [ ] Define custom global variables
  - [ ] More to come!

## Getting started
Take a look at our online [documentation](https://docs.rs/zsh-module/latest/zsh-module) for a quick guide.

## Example module
Making a module is very easy, here's an excerpt from our example module [`greeter`]

```rust
use zsh_module::{Builtin, MaybeError, Module, ModuleBuilder, Opts};

// Notice how this module gets installed as `rgreeter`
zsh_module::export_module!(rgreeter, setup);

struct Greeter;

impl Greeter {
    fn greet_cmd(&mut self, _name: &str, _args: &[&str], _opts: Opts) -> MaybeError {
        println!("Hello, world!");
        Ok(())
    }
}

fn setup() -> Result<Module, Box<dyn std::error::Error>> {
    let module = ModuleBuilder::new(Greeter)
        .builtin(Greeter::greet_cmd, Builtin::new("greet"))
        .build();
    Ok(module)
}
```

[`greeter`]: https://github.com/Diegovsky/zsh-module-rs/tree/master/greeter
