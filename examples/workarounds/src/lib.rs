use serde::{Deserialize, Serialize};
use zsh_module::{zsh, CStrArray, MaybeZError, Module, ModuleBuilder, Opts, ZResult};

const PRECMD_SERIALIZED_STRING: &'static str =
    r#"{"error_code": %?, "job_count": %j, "current_dir": "%~", "line_number": %I}"#;
const ASSOCIATED_VARIABLE: &'static str = "__workarounds_module_precmd_data";

zsh_module::export_module!(workarounds, setup);

/// An example of some workarounds one might need to use to make a zsh prompt.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentState {
    pub error_code: isize,
    pub job_count: usize,
    pub current_dir: String,
    pub line_number: usize,
}
impl CurrentState {
    pub fn export_module() -> ZResult<Module> {
        Ok(ModuleBuilder::new(Self::default())
            .builtin(CurrentState::init_hooks, "workarounds_init".into())
            .builtin(
                CurrentState::refresh_state,
                "workarounds_refresh_state".into(),
            )
            .builtin(CurrentState::render, "workarounds_render".into())
            .builtin(CurrentState::vartest, "workarounds_vartest".into())
            .build())
    }

    /// TODO: This does not work on my setup
    fn init_hooks(&mut self, _name: &std::ffi::CStr, _args: CStrArray, _opts: Opts) -> MaybeZError {
        let inserted_function = format!(
            "function __workarounds::precmd() {{
                print -v {} -P '{}'
                workarounds_refresh_state
            }}
            typeset -aUg precmd_functions
            setopt prompt_subst
            precmd_functions+=(__workarounds::precmd)
            PROMPT=$(workarounds_render)",
            ASSOCIATED_VARIABLE, PRECMD_SERIALIZED_STRING
        );
        println!("{}", &inserted_function);
        zsh::eval_simple(inserted_function)?;
        Ok(())
    }
    fn vartest(&mut self, _name: &std::ffi::CStr, _args: CStrArray, _opts: Opts) -> MaybeZError {
        let mut retval = zsh::Param::get("?").unwrap();
        println!("{:?}", retval.get_value());
        Ok(())
    }
    /// Get the current value of the precmd data
    fn refresh_state(
        &mut self,
        _name: &std::ffi::CStr,
        _args: CStrArray,
        _opts: Opts,
    ) -> MaybeZError {
        println!("Refreshing precmd data");
        let mut precmd_data = if let Some(d) = zsh::Param::get(ASSOCIATED_VARIABLE) {
            d
        } else {
            return Err("Could not get precmd data".into());
        };

        let data_string = match precmd_data.get_value() {
            zsh::ParamValue::Scalar(s) => s,
            _ => {
                return Err("Precmd data was the wrong type! Expected a scalar".into());
            }
        };

        let me = match serde_json::from_str::<Self>(data_string.to_string_lossy().as_ref()) {
            Ok(m) => m,
            Err(e) => {
                return Err(format!("Could not parse precmd data: {}", e).into());
            }
        };

        // set props -- self is a &mut here, so it cannot be directly set with the returns
        self.error_code = me.error_code;
        self.job_count = me.job_count;
        self.current_dir = me.current_dir;
        self.line_number = me.line_number;

        Ok(())
    }

    fn render(&mut self, _name: &std::ffi::CStr, _args: CStrArray, _opts: Opts) -> MaybeZError {
        let has_err = self.error_code != 0;
        let has_jobs = self.job_count > 0;

        let error_code_string = if has_err {
            format!(
                "Returned {}{}",
                self.error_code,
                if has_jobs { " " } else { "" }
            )
        } else {
            String::new()
        };

        let job_count_string = if has_jobs {
            format!("with {} jobs", self.job_count)
        } else {
            String::new()
        };

        println!(
            "{} {}{} {}",
            self.line_number, error_code_string, job_count_string, self.current_dir
        );
        Ok(())
    }
}

#[inline]
fn setup() -> ZResult<Module> {
    CurrentState::export_module()
}
