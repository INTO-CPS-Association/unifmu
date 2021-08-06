use std::{fs, path::Path};

use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct MdCoSimulation {}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MdScalarVariable {
    #[serde(rename = "valueReference")]
    pub value_reference: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MdModelVariables {
    #[serde(rename = "ScalarVariable")]
    pub variables: Vec<MdScalarVariable>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MdModelDescription {
    pub guid: String,
    // #[serde(rename = "CoSimulation")]
    // cosimulation: MdCoSimulation,
    #[serde(rename = "ModelVariables")]
    pub model_variables: MdModelVariables,
}

pub enum ModelDescriptionError {
    UnableToRead,
    UnableToParse,
}

pub fn parse_model_description(
    md_path: &Path,
) -> Result<MdModelDescription, ModelDescriptionError> {
    match fs::read_to_string(md_path) {
        Ok(contents) => match from_str::<MdModelDescription>(&contents) {
            Ok(md) => Ok(md),
            Err(e) => Err(ModelDescriptionError::UnableToParse),
        },
        Err(e) => Err(ModelDescriptionError::UnableToRead),
    }
}
