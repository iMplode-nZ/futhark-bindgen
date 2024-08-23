use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use crate::*;

mod rust;

use convert_case::{Case, Casing};
pub use rust::Rust;

pub(crate) fn first_uppercase(s: &str) -> String {
    let mut s = s.to_string();
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
    s
}

pub(crate) fn convert_struct_name(s: &str) -> &str {
    s.strip_prefix("struct")
        .unwrap()
        .strip_suffix('*')
        .unwrap()
        .strip_prefix(|x: char| x.is_ascii_whitespace())
        .unwrap()
        .strip_suffix(|x: char| x.is_ascii_whitespace())
        .unwrap()
}

/// Code generation config
pub struct Config {
    /// Output file
    pub output_path: PathBuf,

    /// Path to output file
    pub output_file: File,

    pub namer: Box<dyn Namer>,
    pub type_names: HashMap<String, String>,
    pub raw_names: HashMap<String, String>,
    pub entry_points_within_context: bool,
}

impl Config {
    /// Create a new config using the provided output file path
    pub fn new(output: impl AsRef<Path>, namer: impl Namer + 'static) -> Result<Config, Error> {
        Ok(Config {
            output_path: output.as_ref().to_path_buf(),
            output_file: File::create(output)?,
            namer: Box::new(namer),
            type_names: HashMap::new(),
            raw_names: HashMap::new(),
            entry_points_within_context: false,
        })
    }
}

pub trait Generate {
    /// Iterates through the manifest and generates code
    fn generate(&mut self, pkg: &Package, config: &mut Config) -> Result<(), Error> {
        config.namer.init(&pkg.manifest);
        for (name, ty) in &pkg.manifest.types {
            let tyname = config.namer.type_name(name, ty, &pkg.manifest);
            config.type_names.insert(name.clone(), tyname);
            config
                .raw_names
                .insert(name.clone(), convert_struct_name(ty.ctype()).to_string());
        }
        self.bindings(pkg, config)?;
        for (name, ty) in &pkg.manifest.types {
            match ty {
                manifest::Type::Array(ty) => {
                    self.array_type(pkg, config, name, ty)?;
                }
                manifest::Type::Opaque(ty) => {
                    self.opaque_type(pkg, config, name, ty)?;
                }
            }
        }

        for (name, entry) in &pkg.manifest.entry_points {
            self.entry(pkg, config, name, entry)?;
        }
        self.format(&config.output_path)?;
        Ok(())
    }

    /// Step 1: generate any setup code or low-level bindings
    fn bindings(&mut self, _pkg: &Package, _config: &mut Config) -> Result<(), Error>;

    /// Step 2: generate code for array types
    fn array_type(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::ArrayType,
    ) -> Result<(), Error>;

    /// Step 3: generate code for opaque types
    fn opaque_type(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<(), Error>;

    /// Step 4: generate code for entry points
    fn entry(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error>;

    /// Step 5: Optionally, run any formatting program or post-processing on the output file
    fn format(&mut self, _output: &Path) -> Result<(), Error> {
        Ok(())
    }
}

fn rust() -> Box<impl Generate> {
    Box::<Rust>::default()
}

// fn ocaml(config: &Config) -> Box<impl Generate> {
//     Box::new(OCaml::new(config).unwrap())
// }

impl Config {
    /// Automatically detect output language
    pub fn detect(&self) -> Option<Box<dyn Generate>> {
        match self
            .output_path
            .extension()
            .map(|x| x.to_str().expect("Invalid extension"))
        {
            Some("rs") => Some(rust()),
            // Some("ml") => Some(ocaml(self)),
            _ => None,
        }
    }
}

pub trait Namer {
    fn init(&mut self, manifest: &Manifest);
    fn type_name(
        &mut self,
        futhark_name: &str,
        data: &manifest::Type,
        manifest: &Manifest,
    ) -> String;
    fn project_name(&mut self, futhark_name: &str, manifest: &Manifest) -> String;
    fn new_field_name(&mut self, futhark_name: &str, manifest: &Manifest) -> String;
}

#[derive(Default, Debug, Clone)]
pub struct DefaultNamer {
    ctypes: HashMap<String, String>,
}

fn is_valid_name(name: &str) -> bool {
    let first = name.chars().next().expect("Empty name");
    !['(', '{', '#', '['].contains(&first)
}

impl Namer for DefaultNamer {
    fn init(&mut self, manifest: &Manifest) {
        for (futhark_name, ty) in &manifest.types {
            if let manifest::Type::Opaque(manifest::OpaqueType { ctype, .. }) = ty {
                self.ctypes
                    .insert(futhark_name.clone(), convert_struct_name(ctype).to_string());
            }
        }
    }
    fn type_name(
        &mut self,
        futhark_name: &str,
        data: &manifest::Type,
        manifest: &Manifest,
    ) -> String {
        if is_valid_name(futhark_name) {
            return futhark_name.to_case(Case::Pascal);
        }
        match data {
            manifest::Type::Array(array) => {
                let elemtype = array.elemtype.to_str();
                let elemname = first_uppercase(elemtype);
                format!("{}Array{}d", elemname, array.rank)
            }
            manifest::Type::Opaque(opaque) => match &opaque.options {
                manifest::OpaqueOptions::OpaqueArray(array)
                | manifest::OpaqueOptions::RecordArray(array) => {
                    let elemname =
                        self.type_name(&array.elemtype, &manifest.types[&array.elemtype], manifest);
                    format!("{}Array{}d", elemname, array.rank)
                }
                manifest::OpaqueOptions::Sum(_) | manifest::OpaqueOptions::Record(_) => {
                    let ctype = self.ctypes.get(futhark_name).expect("Cannot find type.");
                    "Unnamed".to_string() + ctype.strip_prefix("futhark_opaque").unwrap()
                }
            },
        }
    }
    fn new_field_name(&mut self, futhark_name: &str, _manifest: &Manifest) -> String {
        if futhark_name.chars().next().unwrap().is_ascii_digit() {
            format!("f{}", futhark_name)
        } else {
            futhark_name.to_string()
        }
    }
    fn project_name(&mut self, futhark_name: &str, _manifest: &Manifest) -> String {
        if futhark_name.chars().next().unwrap().is_ascii_digit() {
            format!("f{}", futhark_name)
        } else {
            futhark_name.to_string()
        }
    }
}
