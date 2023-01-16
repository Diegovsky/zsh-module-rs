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
