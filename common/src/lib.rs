pub mod fmi2_md;
pub mod fmi3_md;

use serde;
use serde::Deserialize;

#[derive(Debug)]
pub enum FmiVersion {
    Fmi3,
    Fmi2,
    Fmi1,
}

#[derive(Debug, Deserialize, PartialEq)]
struct FmiCommonModelDescription {
    #[serde(rename = "fmiVersion")]
    fmi_version: String,
}

pub fn get_model_description_major_version(buf: &[u8]) -> Result<FmiVersion, ()> {
    match quick_xml::de::from_reader::<_, FmiCommonModelDescription>(buf) {
        Ok(md) => match md.fmi_version.chars().next() {
            Some('3') => Ok(FmiVersion::Fmi3),
            Some('2') => Ok(FmiVersion::Fmi2),
            Some('1') => Ok(FmiVersion::Fmi1),
            Some(_) => Err(()),
            None => Err(()),
        },
        Err(_) => Err(()),
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::fmi2_md::Fmi2ModelDescription;

//     #[test]
//     fn it_works() {
//         let md = r#"
//     <?xml version='1.0' encoding='utf-8'?>
//     <fmiModelDescription fmiVersion="2.0" modelName="unifmu" guid="77236337-210e-4e9c-8f2c-c1a0677db21b" author="Christian Møldrup Legaard" generationDateAndTime="2020-10-23T19:51:25Z" variableNamingConvention="flat" generationTool="unifmu">
//     <CoSimulation modelIdentifier="unifmu" needsExecutionTool="true" canNotUseMemoryManagementFunctions="false" canHandleVariableCommunicationStepSize="true" />
//     <LogCategories>
//         <Category name="logStatusWarning" />
//         <Category name="logStatusDiscard" />
//         <Category name="logStatusError" />
//         <Category name="logStatusFatal" />
//         <Category name="logStatusPending" />
//         <Category name="logAll" />
//     </LogCategories>
//     <ModelVariables>
//         <!--Index of variable = "1"-->
//         <ScalarVariable name="real_a" valueReference="0" variability="continuous" causality="input">
//         <Real start="0.0" />
//         </ScalarVariable>
//         <!--Index of variable = "2"-->
//         <ScalarVariable name="real_b" valueReference="1" variability="continuous" causality="input">
//         <Real start="0.0" />
//         </ScalarVariable>
//         <!--Index of variable = "3"-->
//         <ScalarVariable name="real_c" valueReference="2" variability="continuous" causality="output" initial="calculated">
//         <Real />
//         </ScalarVariable>
//         <!--Index of variable = "4"-->
//         <ScalarVariable name="integer_a" valueReference="3" variability="discrete" causality="input">
//         <Integer start="0" />
//         </ScalarVariable>
//         <!--Index of variable = "5"-->
//         <ScalarVariable name="integer_b" valueReference="4" variability="discrete" causality="input">
//         <Integer start="0" />
//         </ScalarVariable>
//         <!--Index of variable = "6"-->
//         <ScalarVariable name="integer_c" valueReference="5" variability="discrete" causality="output" initial="calculated">
//         <Integer />
//         </ScalarVariable>
//         <!--Index of variable = "7"-->
//         <ScalarVariable name="boolean_a" valueReference="6" variability="discrete" causality="input">
//         <Boolean start="false" />
//         </ScalarVariable>
//         <!--Index of variable = "8"-->
//         <ScalarVariable name="boolean_b" valueReference="7" variability="discrete" causality="input">
//         <Boolean start="false" />
//         </ScalarVariable>
//         <!--Index of variable = "9"-->
//         <ScalarVariable name="boolean_c" valueReference="8" variability="discrete" causality="output" initial="calculated">
//         <Boolean />
//         </ScalarVariable>
//         <!--Index of variable = "10"-->
//         <ScalarVariable name="string_a" valueReference="9" variability="discrete" causality="input">
//         <String start="" />
//         </ScalarVariable>
//         <!--Index of variable = "11"-->
//         <ScalarVariable name="string_b" valueReference="10" variability="discrete" causality="input">
//         <String start="" />
//         </ScalarVariable>
//         <!--Index of variable = "12"-->
//         <ScalarVariable name="string_c" valueReference="11" variability="discrete" causality="output" initial="calculated">
//         <String />
//         </ScalarVariable>

//     </ModelVariables>
//     <ModelStructure>
//         <Outputs>
//         <Unknown index="3" dependencies="" />
//         <Unknown index="6" dependencies="" />
//         <Unknown index="9" dependencies="" />
//         <Unknown index="12" dependencies="" />
//         </Outputs>
//         <InitialUnknowns>
//         <Unknown index="3" dependencies="" />
//         <Unknown index="6" dependencies="" />
//         <Unknown index="9" dependencies="" />
//         <Unknown index="12" dependencies="" />
//         </InitialUnknowns>
//     </ModelStructure>
//     </fmiModelDescription>"#;

//         let md: Fmi2ModelDescription = quick_xml::de::from_str(md).unwrap();
//         println!("{:?}", md);
//     }
// }
