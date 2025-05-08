use log::{debug, error, info, warn};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Represents compiler error messages
#[derive(Debug)]
pub struct CompileError {
    pub message: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CompileError {}

/// Gets the base directories for shader compilation
pub fn get_shader_dirs() -> (PathBuf, PathBuf) {
    let base_dir = crate::shaders::get_shader_base_dir();
    let src_dir = base_dir.join("src");
    let bin_dir = base_dir.join("bin");
    (src_dir, bin_dir)
}

/// Ensures the shader directories exist
pub fn ensure_dirs() -> std::io::Result<()> {
    let (src_dir, bin_dir) = get_shader_dirs();
    
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)?;
    }
    
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)?;
    }
    
    Ok(())
}

/// Compiles a single shader file
pub fn compile_shader(shader_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let (src_dir, bin_dir) = get_shader_dirs();
    
    // Construct paths
    let src_path = src_dir.join(format!("{}.comp", shader_name));
    let spv_path = bin_dir.join(format!("{}.comp.spv", shader_name));
    
    // Check if source file exists
    if !src_path.exists() {
        return Err(Box::new(CompileError {
            message: format!("Shader source file does not exist: {:?}", src_path),
        }));
    }
    
    // Check if we need to recompile (binary doesn't exist or is older than source)
    let needs_compile = !spv_path.exists() || fs::metadata(&spv_path)?.modified()? < fs::metadata(&src_path)?.modified()?;
    
    if needs_compile {
        info!("Compiling shader: {}", shader_name);
        
        // Use glslc (from Google's Shaderc) to compile the shader
        let output = Command::new("glslc")
            .arg("-fshader-stage=compute")
            .arg("-o")
            .arg(&spv_path)
            .arg(&src_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Shader compilation failed: {}", stderr);
            return Err(Box::new(CompileError {
                message: format!("Failed to compile shader {}: {}", shader_name, stderr),
            }));
        }
        
        debug!("Successfully compiled shader: {}", shader_name);
    } else {
        debug!("Shader {} is already up to date", shader_name);
    }
    
    Ok(spv_path)
}

/// Extracts embedded shaders from the binary to the shader directory
pub fn extract_embedded_shaders() -> std::io::Result<()> {
    let (src_dir, _) = get_shader_dirs();
    
    // Define embedded shaders
    let shaders = [
        ("bilinear_upscale", include_str!("embedded/bilinear_upscale.comp")),
        ("bicubic_upscale", include_str!("embedded/bicubic_upscale.comp")),
    ];
    
    for (name, content) in &shaders {
        let dest_path = src_dir.join(format!("{}.comp", name));
        
        // Only write if file doesn't exist or has different content
        let should_write = if dest_path.exists() {
            let mut file = File::open(&dest_path)?;
            let mut existing_content = String::new();
            file.read_to_string(&mut existing_content)?;
            existing_content != *content
        } else {
            true
        };
        
        if should_write {
            debug!("Extracting embedded shader: {}", name);
            let mut file = File::create(&dest_path)?;
            file.write_all(content.as_bytes())?;
        }
    }
    
    Ok(())
}

/// Compiles all shaders
pub fn compile_all_shaders() -> Result<(), Box<dyn std::error::Error>> {
    let (src_dir, _) = get_shader_dirs();
    
    // Ensure embedded shaders are extracted
    extract_embedded_shaders()?;
    
    // Find all compute shader files
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "comp") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                match compile_shader(stem) {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Failed to compile shader {}: {}", stem, e);
                    }
                }
            }
        }
    }
    
    Ok(())
} 