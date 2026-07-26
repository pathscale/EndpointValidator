//! Types for reading `services.json`, the machine-readable endpoint description
//! written by `endpoint-gen`.
//!
//! The schema types are **not** redefined here. They are re-exported from
//! [`endpoint_libs::model`], which is where `endpoint-gen` gets them from when it
//! writes the file. This crate previously kept hand-copied duplicates, which
//! silently rotted: `EnumVariant.comment` was renamed to `description` upstream
//! and the copy here never followed, so every current `services.json` failed to
//! parse with `missing field 'comment'`. The vendored `Type` had also drifted
//! badly — it still listed `Date`, `Int`, `BigInt`, `Numeric`, `Inet` and
//! `DataTable`, none of which exist upstream, and was missing `UInt32`, `Int32`,
//! `Int64`, `Float64`, `NanoId`, `IpAddr` and `StructTable`.
//!
//! Anything describing the wire schema belongs upstream. Only this tool's own
//! config types are defined here.

use std::collections::HashMap;

use serde::Deserialize;

pub use endpoint_libs::model::{EndpointSchema, EnumVariant, Field, Service, Type};

/// The whole of `services.json`.
///
/// `enums` and `structs` deserialize as bare [`Type`]s because that is literally
/// what they are: serde's external tagging renders `Type::Enum` as
/// `{"Enum": {...}}` and `Type::Struct` as `{"Struct": {...}}`, exactly as the
/// file contains them.
///
/// Note the file holds only `frontend_facing` endpoints — `endpoint-gen`'s
/// deliberate choice, so this tool sees the public surface and nothing else.
#[derive(Debug, Deserialize)]
pub struct Services {
    #[serde(default)]
    pub enums: Vec<Type>,
    pub services: Vec<Service>,
    #[serde(default)]
    pub structs: Vec<Type>,
}

/// One endpoint, flattened into what the driver needs to issue a call.
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

/// `config.toml` — preset parameter values, keyed by endpoint.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub endpoints: HashMap<String, EndpointData>,
}

#[derive(Debug, Deserialize)]
pub struct EndpointData {
    pub name: String,
    #[serde(default)]
    pub params: HashMap<String, ParamValue>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    String(String),
    Number(i64),
    Bool(bool),
    Object(HashMap<String, ParamValue>),
    Array(Vec<ParamValue>),
}
