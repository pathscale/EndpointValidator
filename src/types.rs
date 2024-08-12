use serde::*;
#[derive(Clone, Debug, Hash, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

impl Field {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: i64,
    pub comment: String,
}
impl EnumVariant {
    pub fn new(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            value,
            comment: "".to_owned(),
        }
    }
    pub fn new_with_comment(
        name: impl Into<String>,
        value: i64,
        comment: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            comment: comment.into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum Type {
    TimeStampMs,
    Date,
    Int,
    BigInt,
    Numeric,
    Boolean,
    String,
    Bytea,
    UUID,
    Inet,
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    StructRef(String),
    Object,
    DataTable {
        name: String,
        fields: Vec<Field>,
    },
    Vec(Box<Type>),
    Unit,
    Optional(Box<Type>),
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    EnumRef(String),
    BlockchainDecimal,
    BlockchainAddress,
    BlockchainTransactionHash,
}
impl Type {
    pub fn struct_(name: impl Into<String>, fields: Vec<Field>) -> Self {
        Self::Struct {
            name: name.into(),
            fields,
        }
    }
    pub fn struct_ref(name: impl Into<String>) -> Self {
        Self::StructRef(name.into())
    }
    pub fn datatable(name: impl Into<String>, fields: Vec<Field>) -> Self {
        Self::DataTable {
            name: name.into(),
            fields,
        }
    }
    pub fn vec(ty: Type) -> Self {
        Self::Vec(Box::new(ty))
    }
    pub fn optional(ty: Type) -> Self {
        Self::Optional(Box::new(ty))
    }
    pub fn enum_ref(name: impl Into<String>) -> Self {
        Self::EnumRef(name.into())
    }
    pub fn enum_(name: impl Into<String>, fields: Vec<EnumVariant>) -> Self {
        Self::Enum {
            name: name.into(),
            variants: fields,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnumData {
    #[serde(rename = "Enum")]
    enum_def: EnumDef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointSchema {
    pub name: String,
    pub code: u32,
    pub parameters: Vec<Field>,
    pub returns: Vec<Field>,
    pub stream_response: Option<Type>,
    pub description: String,
    pub json_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointsType {
    pub endpoints: Vec<EndpointSchema>,
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Services {
    pub enums: Vec<EnumData>,
    pub services: Vec<EndpointsType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoints(pub Vec<String>);

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorCode {
    pub code: u32,
    pub symbol: String,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorCodes {
    pub language: String,
    pub codes: Vec<ErrorCode>,
}
