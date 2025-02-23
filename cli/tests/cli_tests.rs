/// This contains all tests run on fmu's generated by the UNIFMU CLI.
/// The setup for all tests is done in rust, but for actual verification this
/// relies on two non-rust dependencies.
/// 
/// 1. FMU importing and functionality is tested in python using the fmpy
///    library. These tests are located in the `python_tests` subdirectory.
/// 2. FMU validation is done using the vdmcheck java application. This app
///    can be found in the `test_dependencies` directory located in the root
///    of this repository.
/// 
/// See the `common` submodule for shared test functionality.
mod common;

use common::{
    distributed_fmu_python_test,
    fmu_python_test,
    vdm_check,
    BasicFmu,
    BreakableFmu,
    DistributedFmu,
    FmiVersion,
    FmuBackendImplementationLanguage,
    LocalFmu,
    ZippedDistributedFmu,
    ZippedLocalFmu
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
fn test_vdm_check_csharp_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_java_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_python_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_python_fmi3_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
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

#[test]
fn test_vdm_check_csharp_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_java_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    vdm_check(fmu);
}

#[test]
fn test_vdm_check_python_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    vdm_check(fmu);
}

#[test]
fn test_platform_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_csharp_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_java_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_python_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_csharp_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_java_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_python_fmi2_distributed() {
    let fmu = DistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_csharp_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_java_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_platform");
}

#[test]
fn test_platform_python_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
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
fn test_version_csharp_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_java_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_python_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_python_fmi3_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
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
fn test_version_csharp_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_java_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_version_python_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_version");
}

#[test]
fn test_extract_csharp_fmi2_local() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2,
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "extract_fmu");
}

#[test]
fn test_extract_java_fmi2_local() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2,
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "extract_fmu");
}

#[test]
fn test_extract_python_fmi2_local() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2,
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "extract_fmu");
}

#[test]
fn test_extract_python_fmi3_local() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi3,
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "extract_fmu");
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
fn test_instantiate_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi3_instantiate");
}

#[test]
fn test_instantiate_csharp_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_java_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_python_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_python_fmi3_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi3_instantiate");
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
fn test_instantiate_csharp_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    distributed_fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_java_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    distributed_fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_python_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
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
fn test_simulate_csharp_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_java_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_python_fmi2_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_python_fmi3_local_zipped() {
    let fmu = ZippedLocalFmu::get_clone(
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
fn test_simulate_csharp_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    distributed_fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_java_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    distributed_fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
fn test_simulate_python_fmi2_distributed_zipped() {
    let fmu = ZippedDistributedFmu::get_clone(
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

    fmu.inject_fault_into_backend_model_file();

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_instantiate - instantiation: Failed to instantiate model")]
fn test_unexpected_exit_in_handshake_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu.inject_fault_into_backend_model_file();

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_instantiate - instantiation: Failed to instantiate model")]
fn test_unexpected_exit_in_handshake_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.inject_fault_into_backend_model_file();

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi3_instantiate - instantiation: Failed to instantiate FMU")]
fn test_unexpected_exit_in_handshake_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.inject_fault_into_backend_model_file();

    fmu_python_test(fmu, "fmi3_instantiate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_simulate: fmi2DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_csharp_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu.inject_fault_into_backend_do_step_function();

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_simulate: fmi2DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_java_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu.inject_fault_into_backend_do_step_function();

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi2_simulate: fmi2DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.inject_fault_into_backend_do_step_function();

    fmu_python_test(fmu, "fmi2_simulate");
}

#[test]
#[should_panic(expected = "PYTHON TEST FAILED - fmi3_simulate: fmi3DoStep failed with status 3 (error).")]
fn test_unexpected_exit_during_command_python_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu.inject_fault_into_backend_do_step_function();

    fmu_python_test(fmu, "fmi3_simulate");
}

#[test]
#[should_panic(expected = "Cannot find shared library")]
fn test_instantiate_csharp_fmi2_as_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::CSharp
    );

    fmu_python_test(fmu, "fmi3_instantiate");
}

#[test]
#[should_panic(expected = "Cannot find shared library")]
fn test_instantiate_java_fmi2_as_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Java
    );

    fmu_python_test(fmu, "fmi3_instantiate");
}

#[test]
#[should_panic(expected = "Cannot find shared library")]
fn test_instantiate_python_fmi2_as_fmi3_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi3_instantiate");
}

#[test]
#[should_panic(expected = "Cannot find shared library")]
fn test_instantiate_python_fmi3_as_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi3, 
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_instantiate");
}

#[test]
fn test_instantiate_multiple_fmus_python_fmi2_local() {
    let fmu = LocalFmu::get_clone(
        &FmiVersion::Fmi2,
        &FmuBackendImplementationLanguage::Python
    );

    fmu_python_test(fmu, "fmi2_instantiate_multiple");
}