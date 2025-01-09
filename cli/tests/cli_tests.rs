mod common;

use common::{
    distributed_fmu_python_test,
    fmu_python_test,
    vdm_check,
    TestableFmu,
    LocalFmu,
    DistributedFmu,
    FmiVersion,
    FmuBackendImplementationLanguage
};

#[test]
fn test_vdm_check_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    vdm_check(fmu);
}

// Failing due to a bug in fmpy (?)
//#[test]
fn test_platform_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_platform");
}

// Failing due to a bug in fmpy (?)
//#[test]
fn test_platform_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_platform");
}

// Failing due to a bug in fmpy (?)
//#[test]
fn test_platform_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_platform");
}

// Failing due to a bug in fmpy (?)
//#[test]
fn test_platform_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_platform");
}

// Failing due to a bug in fmpy (?)
//#[test]
fn test_platform_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_platform");
}

// Failing due to a bug in fmpy (?)
//#[test]
fn test_platform_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_version_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi3_version");
}

#[test]
fn test_version_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_instantiate_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

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
fn test_simulate_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi3_simulate");
}

#[test]
fn test_simulate_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    distributed_fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    distributed_fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    distributed_fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_instantiate - instantiation: Failed to instantiate model")]
fn test_unexpected_exit_in_handshake_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu.break_model();

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_instantiate - instantiation: Failed to instantiate model")]
fn test_unexpected_exit_in_handshake_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu.break_model();

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_instantiate - instantiation: Failed to instantiate model")]
fn test_unexpected_exit_in_handshake_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.break_model();

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi3_instantiate - instantiation: Failed to instantiate FMU")]
fn test_unexpected_exit_in_handshake_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.break_model();

    fmu_python_test(fmu, "fmi3_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_simulate: fmi2DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu.break_do_step_function();

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_simulate: fmi2DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu.break_do_step_function();

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_simulate: fmi2DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.break_do_step_function();

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi3_simulate: fmi3DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.break_do_step_function();

    fmu_python_test(fmu, "fmi3_simulate");
}