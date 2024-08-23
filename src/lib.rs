pub(crate) use std::collections::BTreeMap;

mod compiler;
mod error;
pub(crate) mod generate;
pub mod manifest;
mod package;

pub use compiler::Compiler;
pub use error::Error;
pub use generate::{Config, DefaultNamer, Generate, Rust};
pub use manifest::Manifest;
pub use package::Package;

/// `Backend` is used to select a backend when running the `futhark` executable
#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Backend {
    /// Sequential C backend: `futhark c`
    ///
    /// Requires a C compiler
    #[serde(rename = "c")]
    C,

    /// CUDA backend: `futhark cuda`
    ///
    /// Requires the CUDA runtime and a C compiler
    #[serde(rename = "cuda")]
    Cuda,

    /// OpenCL backend: `futhark opencl`
    ///
    /// Requires OpenCL and a C compiler
    #[serde(rename = "opencl")]
    OpenCl,

    /// Multicore C backend: `futhark multicore`
    ///
    /// Requires a C compiler
    #[serde(rename = "multicore")]
    Multicore,

    /// ISPC backend: `futhark ispc`
    ///
    /// Requires the `ispc` compiler in your `$PATH`
    /// and a C compiler
    #[serde(rename = "ispc")]
    Ispc,

    /// HIP backend: `futhark hip`
    ///
    /// Requires a C compiler
    #[serde(rename = "hip")]
    Hip,
}

impl Backend {
    /// Get the name of a backend
    pub fn to_str(&self) -> &'static str {
        match self {
            Backend::C => "c",
            Backend::Cuda => "cuda",
            Backend::OpenCl => "opencl",
            Backend::Multicore => "multicore",
            Backend::Ispc => "ispc",
            Backend::Hip => "hip",
        }
    }

    /// Return the backend specified by the given name if valid
    pub fn from_name(name: &str) -> Option<Backend> {
        match name.to_ascii_lowercase().as_str() {
            "c" => Some(Backend::C),
            "cuda" => Some(Backend::Cuda),
            "opencl" => Some(Backend::OpenCl),
            "multicore" => Some(Backend::Multicore),
            "ispc" => Some(Backend::Ispc),
            _ => None,
        }
    }

    /// Get the backend from the `FUTHARK_BACKEND` environment variable
    pub fn from_env() -> Option<Backend> {
        match std::env::var("FUTHARK_BACKEND") {
            Ok(name) => Backend::from_name(&name),
            Err(_) => None,
        }
    }

    /// Returns the C libraries that need to be linked for a backend
    pub fn required_c_libs(&self) -> &'static [&'static str] {
        match self {
            Backend::Cuda => &["cuda", "cudart", "nvrtc", "m"],
            Backend::OpenCl => &["OpenCL", "m"],
            Backend::Multicore | Backend::Ispc => &["pthread", "m"],
            Backend::Hip => &["hiprtc", "amdhip64"],
            _ => &[],
        }
    }
}

#[cfg(feature = "build")]
/// Generate the bindings and link the Futhark C code
///
/// `backend` selects the backend to use when generating C code: `futhark $backend --lib`
///
/// `src` is the full path to your Futhark code
///
/// `dest` is expected to be a relative path that will
/// be appended to `$OUT_DIR`
pub fn build(
    backend: Backend,
    src: impl AsRef<std::path::Path>,
    dest: impl AsRef<std::path::Path>,
) {
    use generate::DefaultNamer;

    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest = std::path::PathBuf::from(&out).join(dest);
    let lib = Compiler::new(backend, src)
        .with_output_dir(out)
        .compile()
        .expect("Compilation failed");

    let mut config =
        Config::new(&dest, DefaultNamer::default()).expect("Unable to configure codegen");
    let mut gen = config.detect().expect("Invalid output language");
    gen.generate(&lib, &mut config)
        .expect("Code generation failed");
    lib.link();
}
