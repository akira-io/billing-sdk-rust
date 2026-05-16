/// Compile-time + runtime env resolution for desktop apps.
///
/// In release builds the value is locked to the constant baked at compile
/// time (typically via `option_env!`). In debug builds the runtime env var is
/// honoured first so local development can override without rebuilding.
pub struct EnvSpec<'a> {
    pub name: &'a str,
    pub baked: Option<&'a str>,
    pub debug_default: Option<&'a str>,
    pub release_default: Option<&'a str>,
}

pub fn env_with_debug_override(spec: EnvSpec<'_>) -> Option<String> {
    #[cfg(debug_assertions)]
    {
        if let Ok(v) = std::env::var(spec.name) {
            if !v.is_empty() {
                return Some(v);
            }
        }
        if spec.baked.is_none() {
            if let Some(v) = spec.debug_default {
                return Some(v.to_string());
            }
        }
    }
    spec.baked
        .map(str::to_string)
        .or_else(|| spec.release_default.map(str::to_string))
}
