mod common;

use common::{
    distributed_fmu_python_test,
    local_fmu_python_test,
    TestableFmu,
    DistributedFmu,
    LocalFmu,
    FmiVersion,
    FmuBackendImplementationLanguage
};

#[test]
fn test_instantiate_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    distributed_fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    distributed_fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    distributed_fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_fmu_functionality_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    distributed_fmu_python_test(fmu, "fmi2_full_functionality");
}

#[test]
fn test_fmu_functionality_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    distributed_fmu_python_test(fmu, "fmi2_full_functionality");
}

#[test]
fn test_fmu_functionality_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    distributed_fmu_python_test(fmu, "fmi2_full_functionality");
}