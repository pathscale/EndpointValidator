use serde::*;
#[derive(Clone, Debug, Hash, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: i64,
    pub comment: String,
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

#[derive(Debug, Clone)]
pub struct EndpointMetadata {
    pub service_name: String,
    pub method_id: u32,
    pub params: Vec<ParameterMetadata>,
    pub is_stream: bool,
}

#[derive(Debug, Clone)]
pub struct ParameterMetadata {
    pub name: String,
    pub ty: Type,
}

