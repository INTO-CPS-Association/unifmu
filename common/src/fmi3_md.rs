use quick_xml::de::from_reader;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Fmi3CoSimulation {
    #[serde(rename = "modelIdentifier")]
    pub model_identifier: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Fmi3Variable {
    Float32 {},
    Float64 {},
    Int8 {},
    UInt8 {},
    Int16 {},
    Uint16 {},
    Int32 {},
    Uint32 {},
    Int64 {},
    UInt64 {},
    Boolean {},
    String {},
    Binary {},
    Enumeration {},
    Clock {},
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Fmi3ModelDescription {
    #[serde(rename = "instantiationToken")]
    pub instantiation_token: String,

    #[serde(rename = "CoSimulation")]
    pub cosimulation: Option<Fmi3CoSimulation>,
    // #[serde(rename = "ModelVariables")]
    // pub model_variables: Vec<Fmi3Variable>,
}

pub enum Fmi3ModelDescriptionError {
    UnableToRead,
    UnableToParse,
}

pub fn parse_fmi3_model_description(
    buf: &[u8],
) -> Result<Fmi3ModelDescription, Fmi3ModelDescriptionError> {
    match from_reader::<_, Fmi3ModelDescription>(buf) {
        Ok(md) => Ok(md),
        Err(_) => Err(Fmi3ModelDescriptionError::UnableToParse),
    }
}
