# Zsh Module
This is a high level crate that allows you to define your own zsh module. It is in a very early state but it can be used to define commands.

In the future, most zsh module functionality will be added. Feel free to send a PR if you would like to add more functionality :)

## Getting started
Take a look at our online [documentation](https://docs.rs/zsh-module/latest/zsh-module) for a quick guide.

## Example module
Here's a simple greeter module:

```rust
 use zsh_module::{ Module, ModuleBuilder, Actions, Result };

 zsh_module::impl_hooks!();

 struct Greeter;

 impl Actions for Greeter {
     fn boot(&mut self) -> Result<()> {
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

For more information, take at look at the [`example` module][example]

[example]: https://github.com/Diegovsky/zsh-module-rs/tree/master/greeter
