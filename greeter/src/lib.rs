use zsh_module::{Actions, Module, ModuleBuilder, Result};

zsh_module::export_module!(setup);

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
    let module = ModuleBuilder::new(Greeter).build();
    Ok(module)
}
