use zsh_module::{Module, ModuleBuilder, ZResult};

zsh_module::export_module!(parameters, setup);

#[derive(Debug, Default)]
pub struct ParameterModule {
    pub seen: Vec<String>,
}
impl ParameterModule {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn export_as_module(&self) -> ZResult<Module> {
        let module = ModuleBuilder::new(self)
            .builtin(Self::greet_cmd, "greet_cmd".into())
            .build();

        Ok(module)
    }
    fn greet_cmd(&mut self, _name: &str, _args: &[&str], _opts: Opts) -> MaybeError {
        println!("Hello, world!");
        Ok(())
    }
}
