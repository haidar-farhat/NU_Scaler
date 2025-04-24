pub mod compile;

use std::path::{Path, PathBuf};

/// Returns the path to a compiled shader file
pub fn get_compiled_shader_path(shader_name: &str) -> PathBuf {
    let base_path = Path::new("data/shaders_compiled");
    base_path.join(format!("{}.spv", shader_name))
}

/// Initialize the shader system
pub fn init() -> Result<(), String> {
    compile::ensure_shader_directories().map_err(|e| e.to_string())?;
    // We can optionally compile shaders here or leave it for a build script
    // compile::auto_compile_shaders()?;
    Ok(())
} 