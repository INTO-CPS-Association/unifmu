use quick_xml::de::from_reader;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Fmi2CoSimulation {
    #[serde(rename = "modelIdentifier")]
    pub model_identifier: String,

    #[serde(rename = "canRunAsynchronuously")]
    pub can_run_asynchronously: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Fmi2ScalarVariable {
    #[serde(rename = "valueReference")]
    pub value_reference: u32,
    pub name: String,
    #[serde(rename = "$value")]
    pub var: Fmi2Variable,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Fmi2ModelVariables {
    #[serde(rename = "ScalarVariable")]
    pub variables: Vec<Fmi2ScalarVariable>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Fmi2ModelDescription {
    pub guid: String,

    #[serde(rename = "CoSimulation")]
    pub cosimulation: Option<Fmi2CoSimulation>,

    #[serde(rename = "ModelVariables")]
    pub model_variables: Fmi2ModelVariables,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Fmi2Variable {
    Real { start: Option<f64> },
    Integer { start: Option<i32> },
    Boolean { start: Option<bool> },

    String { start: Option<String> },
}

pub enum Fmi2ModelDescriptionError {
    UnableToRead,
    UnableToParse,
}

pub fn parse_fmi2_model_description(
    buf: &[u8],
) -> Result<Fmi2ModelDescription, Fmi2ModelDescriptionError> {
    match from_reader::<_, Fmi2ModelDescription>(buf) {
        Ok(md) => Ok(md),
        Err(_) => Err(Fmi2ModelDescriptionError::UnableToParse),
    }
}
