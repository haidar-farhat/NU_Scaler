//! NuScaler Core Library - SOLID API scaffolding

pub mod capture;
pub mod gpu;
pub mod upscale;
pub mod renderer;

/// Public API for initializing the core library (placeholder)
pub fn initialize() {
    // TODO: Initialize logging, config, etc.
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
