use zsh_module::{Builtin, MaybeError, Module, ModuleBuilder, Opts};

zsh_module::export_module!(setup);

struct Greeter;

impl Greeter {
    fn greet_cmd(&mut self, name: &str, args: &[&str], opts: Opts) -> MaybeError {
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
