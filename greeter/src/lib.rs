use zsh_module::{Builtin, MaybeZError, Module, ModuleBuilder, Opts};

// Notice how this module gets installed as `rgreeter`
zsh_module::export_module!(rgreeter, setup);

struct Greeter;

impl Greeter {
    fn greet_cmd(&mut self, _name: &str, _args: &[&str], _opts: Opts) -> MaybeZError {
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
