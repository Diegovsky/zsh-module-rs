use std::collections::HashMap;

use zsh_module::{
    zsh::{self, ParamValue},
    CStrArray, MaybeZError, Module, ModuleBuilder, Opts, ZResult,
};

zsh_module::export_module!(parameters, setup);

#[derive(Debug, Default)]
pub struct ParameterModule {
    pub seen: HashMap<String, String>,
}
impl ParameterModule {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn export_as_module(self) -> ZResult<Module> {
        let module = ModuleBuilder::new(self)
            .builtin(ParameterModule::get_param, "get_param".into())
            .builtin(ParameterModule::get_param_short, "get_param_short".into())
            .build();

        Ok(module)
    }
    /// A verbose example of how you can get the value of a parameter.
    fn get_param(&mut self, _name: &std::ffi::CStr, args: CStrArray, _opts: Opts) -> MaybeZError {
        for arg in args.iter_str().filter_map(|a| a.ok()) {
            if let Some(mut p) = zsh::get(arg) {
                let flags = p.flags();
                // `p.get_value()` is a mutable function, so it must come after `p.flags()`
                let value = p.get_value();

                let cache_key = arg.to_string();

                let cache_value = match value {
                    ParamValue::Array(arr) => {
                        let array_string = arr
                            .iter()
                            .map(|a| format!("\t'{}'", a.to_string_lossy().replace("'", "'\\''")))
                            .collect::<Vec<_>>()
                            .join("\n");

                        println!(
                            "'{}' is an array with the following elements:\n(\n{}\n)",
                            arg, &array_string
                        );

                        array_string
                    }
                    ParamValue::Scalar(scal) => {
                        // C strings don't technically have to be UTF-8, so we have to convert this.
                        let s_string = scal.to_string_lossy().to_string();

                        println!(
                            "'{}' is a scalar (string) with the following value: '{}'",
                            arg, &s_string
                        );

                        s_string
                    }
                    ParamValue::Float(float) => {
                        println!(
                            "'{}' is a floating-point number with the following value: {}",
                            arg, float
                        );

                        // In this example, we only want Strings.
                        float.to_string()
                    }
                    ParamValue::Integer(int) => {
                        println!("'{}' is an integer with the following value: {}", arg, int);
                        int.to_string()
                    }
                    ParamValue::HashTable => {
                        println!("'{}' is a hash table. We don't support those yet.", arg);
                        String::from("Hashtable (unsupported)")
                    }
                };
                // cache the value if you want
                self.seen.insert(cache_key, cache_value);
            } else {
                println!("Parameter not found, possibly uninitialized: {arg}");
            }
        }
        Ok(())
    }
    /// A slightly shorter version
    fn get_param_short(
        &mut self,
        _name: &std::ffi::CStr,
        args: CStrArray,
        _opts: Opts,
    ) -> MaybeZError {
        let params = args
            .iter_str()
            .filter_map(|a| a.ok())
            .filter_map(|a| {
                if let Some(mut v) = zsh::get(a) {
                    Some(format!("{}: {:?}", a, v.get_value()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        println!("{}", params);

        Ok(())
    }
}

fn setup() -> ZResult<Module> {
    ParameterModule::new().export_as_module()
}
