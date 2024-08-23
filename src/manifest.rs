use crate::*;
use serde::Deserialize;

/// Scalar types
#[derive(Clone, Debug, Deserialize)]
pub enum ElemType {
    /// Signed 8 bit integer
    #[serde(rename = "i8")]
    I8,

    /// Signed 16 bit integer
    #[serde(rename = "i16")]
    I16,

    /// Signed 32 bit integer
    #[serde(rename = "i32")]
    I32,

    /// Signed 64 bit integer
    #[serde(rename = "i64")]
    I64,

    /// Unsigned 8 bit integer
    #[serde(rename = "u8")]
    U8,

    /// Unsigned 16 bit integer
    #[serde(rename = "u16")]
    U16,

    /// Unsigned 32 bit integer
    #[serde(rename = "u32")]
    U32,

    /// Unsigned 64 bit integer
    #[serde(rename = "u64")]
    U64,

    /// 16 bit float
    #[serde(rename = "f16")]
    F16,

    /// 32 bit float
    #[serde(rename = "f32")]
    F32,

    /// 64 bit float
    #[serde(rename = "f64")]
    F64,

    /// Boolean
    #[serde(rename = "bool")]
    Bool,
}

impl ElemType {
    pub fn to_str(&self) -> &'static str {
        match self {
            ElemType::I8 => "i8",
            ElemType::I16 => "i16",
            ElemType::I32 => "i32",
            ElemType::I64 => "i64",
            ElemType::U8 => "u8",
            ElemType::U16 => "u16",
            ElemType::U32 => "u32",
            ElemType::U64 => "u64",
            ElemType::F16 => "f16",
            ElemType::F32 => "f32",
            ElemType::F64 => "f64",
            ElemType::Bool => "bool",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Output {
    pub r#type: String,
    pub unique: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Input {
    pub name: String,
    pub r#type: String,
    pub unique: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    pub cfun: String,
    pub outputs: Vec<Output>,
    pub inputs: Vec<Input>,
    pub tuning_params: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArrayOps {
    pub free: String,
    pub index: String,
    pub new: String,
    // pub new_raw: String,
    pub shape: String,
    pub values: String,
    // pub values_raw: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArrayType {
    pub ctype: String,
    pub rank: usize,
    pub elemtype: ElemType,
    pub ops: ArrayOps,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpaqueOps {
    pub free: String,
    pub store: String,
    pub restore: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Field {
    pub name: String,
    pub project: String,
    pub r#type: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpaqueType {
    pub ctype: String,
    pub ops: OpaqueOps,
    #[serde(flatten)]
    pub options: OpaqueOptions,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Record {
    pub new: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Variant {
    pub construct: String,
    pub destruct: String,
    pub payload: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Sum {
    pub variant: String,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RecordArray {
    pub zip: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpaqueArray {
    pub rank: usize,
    pub elemtype: String,
    pub index: String,
    pub shape: String,
    #[serde(flatten)]
    pub record: Option<RecordArray>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum OpaqueOptions {
    #[serde(rename = "record")]
    Record(Record),
    #[serde(rename = "sum")]
    Sum(Sum),
    // Blocking on https://github.com/serde-rs/serde/issues/1847.
    #[serde(rename = "opaque_array")]
    OpaqueArray(OpaqueArray),
    #[serde(rename = "record_array")]
    RecordArray(OpaqueArray),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "array")]
    Array(ArrayType),
    #[serde(rename = "opaque")]
    Opaque(OpaqueType),
}
impl Type {
    pub fn ctype(&self) -> &str {
        match self {
            Type::Array(ArrayType { ctype, .. }) => ctype,
            Type::Opaque(OpaqueType { ctype, .. }) => ctype,
        }
    }
}

/// A Rust encoding of the Futhark manifest file
#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    pub backend: Backend,
    pub version: String,
    pub entry_points: BTreeMap<String, Entry>,
    pub types: BTreeMap<String, Type>,
}

impl Manifest {
    /// Parse the manifest file
    pub fn parse_file(filename: impl AsRef<std::path::Path>) -> Result<Manifest, Error> {
        let r = std::fs::File::open(filename)?;
        let manifest = serde_json::from_reader(r)?;
        Ok(manifest)
    }
}
