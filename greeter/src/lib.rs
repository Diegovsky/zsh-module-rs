use zsh_module::{Builtin, Module, ModuleBuilder, Opts, Result};

zsh_module::export_module!(setup);

struct Greeter;

impl Greeter {
    fn greet_cmd(&mut self, name: &str, args: &[&str], opts: Opts) -> Result<()> {
        println!("Hello, world!");
        Ok(())
    }
}

fn setup() -> Result<Module> {
    let module = ModuleBuilder::new(Greeter)
        .builtin(Greeter::greet_cmd, Builtin::new("greet"))
        .build();
    Ok(module)
}
