use std::ffi::CStr;

use zsh_module::{Builtin, CStrArray, MaybeZError, Module, ModuleBuilder, Opts, ZError};

// Notice how this module gets installed as `rgreeter`
zsh_module::export_module!(rgreeter, setup);

struct Greeter;

impl Greeter {
    fn greet_cmd(&mut self, _name: &CStr, _args: CStrArray, _opts: Opts) -> MaybeZError {
        println!("Hello, world!");
        Ok(())
    }
    fn get_cmd(&mut self, _name: &CStr, args: CStrArray, _opts: Opts) -> MaybeZError {
        if args.len() == 0 {
            return Err(ZError::Conversion("Expected at least 1 element".into()));
        }
        for arg in args.iter() {
            if let Some(mut param) = zsh_module::zsh::get(arg) {
                println!("{}={:?}", arg.to_string_lossy(), param.get_value());
            }
        }
        Ok(())
    }
}

fn setup() -> Result<Module, Box<dyn std::error::Error>> {
    let module = ModuleBuilder::new(Greeter)
        .builtin(Greeter::greet_cmd, Builtin::new("greet"))
        .builtin(Greeter::get_cmd, Builtin::new("rget"))
        .build();
    Ok(module)
}
