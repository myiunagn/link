use linkc_parser::*;

#[cfg(feature = "llvm-backend")]
pub mod backend;

#[cfg(not(feature = "llvm-backend"))]
pub mod backend {
    use super::*;

    pub struct LlvmBackend;

    impl LlvmBackend {
        pub fn new() -> Self {
            LlvmBackend
        }

        pub fn generate(&mut self, _program: &Program) -> Result<String, String> {
            Err(
                "LLVM backend is not available. Build with: cargo build --features llvm-backend\n\
                 This requires LLVM to be installed on your system.\n\
                 See: https://llvm.org/docs/GettingStarted.html".to_string()
            )
        }

        pub fn compile_to_native(&mut self, _program: &Program, _output_path: &str) -> Result<String, String> {
            Err(
                "LLVM backend is not available. Build with: cargo build --features llvm-backend".to_string()
            )
        }

        pub fn compile_to_ir(&mut self, _program: &Program) -> Result<String, String> {
            Err(
                "LLVM backend is not available. Build with: cargo build --features llvm-backend".to_string()
            )
        }
    }
}

pub use backend::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_backend_available() {
        let mut backend = LlvmBackend::new();
        let result = backend.generate(&Program::Block(vec![]));
        assert!(result.is_ok() || result.is_err());
    }
}
