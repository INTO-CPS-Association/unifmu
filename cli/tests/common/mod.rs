//! Contains common functions for the test suite.
#![allow(dead_code)]

use std::{
    ffi::OsString,
    fs::{copy, create_dir, create_dir_all, read_dir, remove_dir_all, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::LazyLock
};

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use unifmu::utils::zip_dir;
use walkdir::WalkDir;
use zip::CompressionMethod;

/// When set true all FMUs that fail during the python tests are persisted
/// instead of being cleaned up at the end of the test.
static DEBUG_PERSIST_FAILING_TEST_FMUS: bool = false;

/// Realtive path from the cli directory to the directory to store test FMUs
/// created with the new_persistent() method.
/// 
/// This is NOT where initially temporary FMUs that are persistet are saved.
/// They are located where they were initially created for debugging reasons.
static PERSITENTLY_CREATED_UNIFMUS_DIRECTORY: &str = "../generated_fmus/cli_tests";

/// Pass the FMU to the given test function in a python subprocess
/// 
/// Monitors the output of this subprocess, and if
/// the subprocess returns a TEST FAILED message, this function panics with
/// "PYTHON " + that message.
/// 
/// Should NOT be called with a Distributed FMU and a python function that
/// instantiates that FMU as part of the test.
/// Distributed FMUs that aren't instantiated during testing are ok.
pub fn fmu_python_test(
    fmu: impl BasicFmu,
    python_test_function_name: &str
) {
    let python_test_process = start_python_test_process(
        python_test_function_name,
        fmu.importable_path(),
        fmu.is_zipped()
    );

    let python_test_output_reader = BufReader::new(&python_test_process);

    for read_result in python_test_output_reader.lines() {
        match read_result {
            Ok(line) => {
                if line.contains("TEST FAILED") {
                    if DEBUG_PERSIST_FAILING_TEST_FMUS {
                        fmu.persist_directory();
                    }
                    panic!("PYTHON {line}");
                
                } else {
                    println!("{line}");
                }
            },
            Err(e) => {
                if DEBUG_PERSIST_FAILING_TEST_FMUS {
                    fmu.persist_directory();
                }
                panic!("Reading output from python test script returned error '{e}'");
            }
        }
    }
}

/// Pass the Distributed FMU to the given test function in a python subprocess,
/// and start the private backend when the FMU is instantiated.
/// 
/// Monitors the output of this subprocess, and if
/// the subprocess returns a TEST FAILED message, this function panics with
/// "PYTHON " + that message.
///
/// If for whatever reason the private backend wasn't started, this function
/// panics.
/// As such this hould ONLY be called with Distributed FMUs that are
/// instantiated as part of the test.
pub fn distributed_fmu_python_test(
    fmu: impl BasicFmu + RemoteBackend,
    python_test_function_name: &str
) {
    let python_test_process = start_python_test_process(
        python_test_function_name,
        fmu.importable_path(),
        fmu.is_zipped()
    );

    let python_test_output_reader = BufReader::new(&python_test_process);

    let mut remote_process: Option<duct::Handle> = None;

    for read_result in python_test_output_reader.lines() {
        match read_result {
            Ok(line) => {
                if line.contains("TEST FAILED") {
                    if let Some(remote_process) = remote_process {
                        let _ = remote_process.kill();
                    }

                    if DEBUG_PERSIST_FAILING_TEST_FMUS {
                        fmu.persist_directory();
                    }

                    panic!("PYTHON {line}");
                
                } else if line.contains("Connect remote backend to dispatcher through port") {
                    let port_string =  line[50..].to_string();

                    println!("Connecting remote backend to fmu dispatcher through port {port_string}");

                    remote_process = Some(fmu.start_remote_backend(port_string));

                } else {
                    println!("{line}");
                }
            },
            Err(e) => {
                if let Some(remote_process) = remote_process {
                    let _ = remote_process.kill();
                }

                if DEBUG_PERSIST_FAILING_TEST_FMUS {
                    fmu.persist_directory();
                }

                panic!("Reading output from python test script returned error '{e}'");
            }
        }
    }

    if let Some(remote_process) = remote_process {
        let _ = remote_process.wait();
    } else {
        if DEBUG_PERSIST_FAILING_TEST_FMUS {
            fmu.persist_directory();
        }
        panic!("Remote backend wasn't started!");
    }
}

/// Starts the python test subprocess, returning a duct::ReaderHandle to it.
/// 
/// Panics if the test script isn't available or cannot be executed.
fn start_python_test_process(
    python_test_function_name: &str,
    fmu_path: impl Into<OsString>,
    is_zipped: bool
) -> duct::ReaderHandle {
    let test_directory = std::env::current_dir()
        .expect("Should be able to get current directory")
        .join("tests")
        .join("python_tests");

    let python_test_script_name = "fmu_tests.py";

    assert!(
        test_directory.join(python_test_script_name)
            .exists(),
        "Python test script '{python_test_script_name}' wasn't found in test directory '{}'",
        test_directory.display()
    );

    // Unix systems differentiates version 2 and 3 of python in their binary names
    // Windows doesn't
    let python_interpreter_binary_name = match std::env::consts::OS {
        "windows" => "python",
        _other => "python3"
    };

    duct::cmd!(
        python_interpreter_binary_name,
        python_test_script_name,
        python_test_function_name,
        fmu_path,
        is_zipped.to_string()
    )
        .dir(test_directory)
        .stderr_to_stdout()
        .reader()
        .expect("Should be able to start python test process")
}

/// Checks the validity of the FMU model description byt converting it to
/// VDM-SL and comparing them to a prebuild model.
/// 
/// Panics if the vdmcheck tool isn't available or if it returns an error.
pub fn vdm_check(fmu: impl BasicFmu) {
    let fmu = fmu.zipped();

    let test_dependencies = std::env::current_dir()
        .expect("Couldn't access current directory")
        .parent()
        .expect("Current directory does not have a parent")
        .join("test_dependencies");

    assert!(
        test_dependencies.exists(),
        "The directory {}, which should contain the vdm check dependency, does not exist",
        test_dependencies.display()
    );

    let version_string = match fmu.version() {
        FmiVersion::Fmi2 => "2",
        FmiVersion::Fmi3 => "3"
    };

    let vdm_check_root_name = format!("vdmcheck{version_string}");
    let vdm_check_jar_name = format!("{vdm_check_root_name}.jar");

    let test_dependencies_unopenable = format!(
        "Couldn't open test dependencies directory {}",
        test_dependencies.display()
    );

    let vdm_check_directory_not_found = format!(
        "No vdm_check directory found in {}",
        test_dependencies.display()
    );

    let vdm_check_dir = test_dependencies.read_dir()
        .expect(&test_dependencies_unopenable)
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry.file_name()
                .to_string_lossy()
                .starts_with(&vdm_check_root_name)
        })
        .expect(&vdm_check_directory_not_found)
        .path();

    let vdm_check_jar = vdm_check_dir.join(vdm_check_jar_name);

    assert!(
        vdm_check_jar.exists(),
        "{} not found in {}",
        vdm_check_jar.display(),
        vdm_check_dir.display()
    );

    let mut vdm_check_cmd = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_jar)
        .arg(fmu.importable_path())
        .assert()
        .success()
        .stdout(contains("No errors found"));
}

/// The UNIFMU backend languages supported by default by the project.
#[derive(Clone, PartialEq)]
pub enum FmuBackendImplementationLanguage {
    CSharp,
    Java,
    Python
}

impl FmuBackendImplementationLanguage {
    pub fn cmd_str(&self) -> &str {
        match self {
            FmuBackendImplementationLanguage::CSharp => "c-sharp",
            FmuBackendImplementationLanguage::Java => "java",
            FmuBackendImplementationLanguage::Python => "python"
        }
    }

    pub fn fault_str(&self) -> &str {
        match self {
            FmuBackendImplementationLanguage::CSharp => "throw new Exception();",
            FmuBackendImplementationLanguage::Java => "int doesNotCompute = 1/0;",
            FmuBackendImplementationLanguage::Python => "raise Exception()"
        }
    }

    pub fn model_str(&self) -> &str {
        match self {
            FmuBackendImplementationLanguage::CSharp => "model.cs",
            FmuBackendImplementationLanguage::Java => "src/main/java/Model.java",
            FmuBackendImplementationLanguage::Python => "model.py"
        }
    }
}

fn gradle_command_base() -> (String, String) {
    match std::env::consts::OS {
        "windows" => (
            String::from("powershell.exe"),
            String::from("./gradlew.bat")
        ),
        "macos" => (
            String::from("zsh"),
            String::from("./gradlew")
        ),
        _other => (
            String::from("sh"),
            String::from("gradlew")
        )
    }
}

/// The major versions of FMI supported by UNIFMU.
#[derive(Clone)]
pub enum FmiVersion {
    Fmi2,
    Fmi3
}

impl FmiVersion {
    pub fn as_str(&self) -> &str {
        match self {
            FmiVersion::Fmi2 => "fmi2",
            FmiVersion::Fmi3 => "fmi3"
        }
    }
}

enum FmuDirectory {
    Persistent(PathBuf),
    Temporary(TempDir),
}

impl FmuDirectory {
    pub fn path(&self) -> &Path {
        match self {
            FmuDirectory::Persistent(path_buf) => path_buf.as_path(),
            FmuDirectory::Temporary(temp_dir) => temp_dir.path()
        }
    }

    pub fn persist(self) -> FmuDirectory {
        match self {
            FmuDirectory::Persistent(_) => self,
            FmuDirectory::Temporary(temp_dir) => FmuDirectory::Persistent(temp_dir.into_path())
        }
    }
}

/// A standard unzipped FMU generated by the UNIFMU CLI.
pub struct LocalFmu {
    name: String,
    directory: FmuDirectory,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

/// A standard zipped FMU generated by the UNIFMU CLI.
pub struct ZippedLocalFmu {
    name: String,
    file: File,
    directory: FmuDirectory,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

/// A distributed unzipped FMU generated by the UNIFMU CLI.
pub struct DistributedFmu {
    name: String,
    directory: FmuDirectory,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

/// A distributed zipped FMU generated by the UNIFMU CLI.
/// Note that only the proxy part of the FMU is zipped - the private part is
/// still accesible as a normal directory.
pub struct ZippedDistributedFmu {
    name: String,
    file: File,
    directory: FmuDirectory,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

/// A distributed zipped blackbox FMU generated by the UNIFMU CLI.
/// Note that only the proxy part of the FMU is zipped - the private part is
/// still accesible as a normal directory.
/// The black-box contained in the private part is a ZippedLocalFmu.
pub struct BlackboxDistributedFmu {
    name: String,
    file: File,
    directory: FmuDirectory,
    version: FmiVersion,
    black_box_fmu: ZippedLocalFmu,
}

impl BasicFmu for LocalFmu {
    fn new(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> LocalFmu {
        let directory = FmuDirectory::Temporary(Self::new_tmp_dir());

        let this = LocalFmu {
            name,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this.post_generation_setup();

        this
    }

    fn new_persistent(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage,
        container_directory_name: &str
    ) -> LocalFmu {
        let container_directory_path = std::env::current_dir()
            .expect("Should be able to get current directory")
            .join(PERSITENTLY_CREATED_UNIFMUS_DIRECTORY)
            .join(container_directory_name);

        if container_directory_path.exists() {
            remove_dir_all(&container_directory_path)
                .expect("Should be able to remove old persisted FMU.");
        }

        create_dir_all(&container_directory_path)
            .expect("Should be able to create container directory for new persistent FMU.");

        let directory = FmuDirectory::Persistent(container_directory_path);

        let this = LocalFmu {
            name,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this.post_generation_setup();

        this
    }

    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> LocalFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (*CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (*JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (*PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (*PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from(self.language().cmd_str()),
            String::from(self.name()),
        ];

        match self.version() {
            FmiVersion::Fmi2 => (),
            FmiVersion::Fmi3 => args.push(
                String::from(
                    self.version().as_str()
                )
            ),
        };

        args
    }

    fn directory(&self) -> &Path {
        self.directory.path()
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn importable_path(&self) -> PathBuf {
        self.directory()
            .join(self.name())
    }

    fn is_zipped(&self) -> bool {
        false
    }

    fn zipped(&self) -> impl ZippedTestableFmu {
        let new_directory = TempDir::new()
            .expect("Should be able to create new temporary FMU directory.");

        let fmu_name = self.name();

        let zip_file_path = new_directory
            .path()
            .join(
                ZippedLocalFmu::with_zip_suffix(fmu_name)
            );

        let zip_file = File::create(&zip_file_path)
            .expect("Should be able to create new FMU zipfile.");

        let old_fmu_directory = self.directory.path().join(fmu_name);
        
        let mut iterable_old_fmu_directory = WalkDir::new(&old_fmu_directory)
            .into_iter()
            .filter_map(|e| e.ok());

        let old_prefix = old_fmu_directory
            .to_str()
            .expect("Should be able to represent old directory as str.");

        zip_dir(
            &mut iterable_old_fmu_directory,
            old_prefix,
            zip_file,
            CompressionMethod::Deflated
        ).expect("Should be able to zip old directory into new file.");

        ZippedLocalFmu {
            name: String::from(fmu_name),
            file: File::open(&zip_file_path).expect("Should be able to open newly created zipfile."),
            directory: FmuDirectory::Temporary(new_directory),
            language: self.language().clone(),
            version: self.version().clone(),
        }
    }

    fn persist_directory(mut self) {
        self.directory = self.directory.persist();
        println!("Persisted FMU in directory: {:?}", self.directory());
    }
}

impl BreakableFmu for LocalFmu {
    fn model_file_path(&self) -> PathBuf {
        self.directory()
            .join(self.name())
            .join("resources")
            .join(self.language().model_str())
    }

    fn do_step_function_line_number(&self) -> u64 {
        match self.language() {
            FmuBackendImplementationLanguage::CSharp => 55,
            FmuBackendImplementationLanguage::Java => 94,
            FmuBackendImplementationLanguage::Python => {
                match self.version() {
                    FmiVersion::Fmi2 => 38,
                    FmiVersion::Fmi3 => 193
                }
            },
        }
    }

    fn do_step_function_injection_prefix(&self) -> &str {
        match self.language() {
            FmuBackendImplementationLanguage::Python => "        ",
            _ => ""
        }
    }
}

impl Clone for LocalFmu {
    fn clone(&self) -> Self {
        let new_directory = Self::new_tmp_dir();

        copy_directory_recursive(
            self.directory()
                .join(self.name()), 
            new_directory
                .path()
                .join(self.name())
        )
            .expect("Should be able to recursively copy cloned FMU into new FMU path");
        
        Self {
            name: String::from(self.name()),
            directory: FmuDirectory::Temporary(new_directory),
            language: self.language().clone(),
            version: self.version().clone(),
        }
    }
}

impl PostGenerationSetup for LocalFmu {
    fn backend_path(&self) -> PathBuf {
        self.directory()
            .join(self.name())
            .join("resources")
    }
}

impl ZippedLocalFmu {
    fn copy_to(&self, new_directory: FmuDirectory) -> Self {
        let file_name = self.zip_name();

        let new_file_path = new_directory
            .path()
            .join(&file_name);

        copy(
            self.directory().join(&file_name),
            &new_file_path
        ).expect("Should be able to copy zip file.");

        Self {
            name: String::from(self.name()),
            file: File::open(&new_file_path).expect("Should be able to open new cloned zip file."),
            directory: new_directory,
            language: self.language().clone(),
            version: self.version().clone()
        }
    }
}

impl BasicFmu for ZippedLocalFmu {
    fn new(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> ZippedLocalFmu {
        let directory = FmuDirectory::Temporary(Self::new_tmp_dir());

        let file = File::create(
            directory.path().join(Self::with_zip_suffix(&name))
        ).expect("Should be able to create FMU zip file.");

        let this = ZippedLocalFmu {
            name,
            file,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this
    }

    fn new_persistent(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage,
        container_directory_name: &str
    ) -> ZippedLocalFmu {
        let container_directory_path = std::env::current_dir()
            .expect("Should be able to get current directory")
            .join(PERSITENTLY_CREATED_UNIFMUS_DIRECTORY)
            .join(container_directory_name);

        if container_directory_path.exists() {
            remove_dir_all(&container_directory_path)
                .expect("Should be able to remove old persisted FMU.");
        }

        create_dir_all(&container_directory_path)
            .expect("Should be able to create container directory for new persistent FMU.");

        let directory = FmuDirectory::Persistent(container_directory_path);

        let file = File::create(
            directory.path().join(Self::with_zip_suffix(&name))
        ).expect("Should be able to create FMU zip file.");

        let this = ZippedLocalFmu {
            name,
            file,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this
    }

    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> ZippedLocalFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (*ZIPPED_CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (*ZIPPED_JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (*ZIPPED_PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (*ZIPPED_PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from(self.language().cmd_str()),
            String::from(self.name()),
        ];

        match self.version() {
            FmiVersion::Fmi2 => (),
            FmiVersion::Fmi3 => args.push(
                String::from(
                    self.version().as_str()
                )
            ),
        };

        args.push(String::from("--zipped"));

        args
    }

    fn directory(&self) -> &Path {
        self.directory.path()
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn importable_path(&self) -> PathBuf {
        self.directory()
            .join(self.zip_name())
    }

    fn is_zipped(&self) -> bool {
        true
    }

    #[allow(refining_impl_trait)]
    fn zipped(&self) -> ZippedLocalFmu {
        self.clone()
    }

    fn persist_directory(mut self) {
        self.directory = self.directory.persist();
        println!("Persisted FMU in directory: {:?}", self.directory());
    }
}

impl ZippedTestableFmu for ZippedLocalFmu {
    fn file(self) -> File {
        self.file
    }
}

impl Clone for ZippedLocalFmu {
    fn clone(&self) -> Self {
        self.copy_to(FmuDirectory::Temporary(Self::new_tmp_dir()))
    }
}

impl BasicFmu for DistributedFmu {
    fn new(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> DistributedFmu {
        let directory = FmuDirectory::Temporary(Self::new_tmp_dir());

        let this = DistributedFmu {
            name,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this.post_generation_setup();

        this
    }

    fn new_persistent(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage,
        container_directory_name: &str
    ) -> DistributedFmu {
        let container_directory_path = std::env::current_dir()
            .expect("Should be able to get current directory")
            .join(PERSITENTLY_CREATED_UNIFMUS_DIRECTORY)
            .join(container_directory_name);

        if container_directory_path.exists() {
            remove_dir_all(&container_directory_path)
                .expect("Should be able to remove old persisted FMU.");
        }

        create_dir_all(&container_directory_path)
            .expect("Should be able to create container directory for new persistent FMU.");

        let directory = FmuDirectory::Persistent(container_directory_path);

        let this = DistributedFmu {
            name,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this.post_generation_setup();

        this
    }

    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> DistributedFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (*DISTRIBUTED_CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (*DISTRIBUTED_JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (*DISTRIBUTED_PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (*DISTRIBUTED_PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate-distributed"),
            String::from(self.language().cmd_str()),
            String::from(self.name()),
        ];

        match self.version() {
            FmiVersion::Fmi2 => (),
            FmiVersion::Fmi3 => args.push(
                String::from(
                    self.version().as_str()
                )
            ),
        };

        args
    }

    fn directory(&self) -> &Path {
        self.directory.path()
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn name(&self) -> &str {
        &self.name
    }
    
    fn importable_path(&self) -> PathBuf {
        self.proxy_directory_path()
    }

    fn is_zipped(&self) -> bool {
        false
    }

    #[allow(refining_impl_trait)]
    fn zipped(&self) -> ZippedDistributedFmu {
        let new_directory = TempDir::new()
            .expect("Should be able to create new temporary FMU directory.");

        let zip_file_path = new_directory
            .path()
            .join(
                ZippedDistributedFmu::with_zip_suffix(
                    &self.proxy_directory_name()
                )
            );

        let zip_file = File::create(&zip_file_path)
            .expect("Should be able to create new FMU zipfile.");

        let old_fmu_directory = self.proxy_directory_path();
        
        let mut iterable_old_fmu_directory = WalkDir::new(&old_fmu_directory)
            .into_iter()
            .filter_map(|e| e.ok());

        let old_prefix = old_fmu_directory
            .to_str()
            .expect("Should be able to represent old directory as str.");

        zip_dir(
            &mut iterable_old_fmu_directory,
            old_prefix,
            zip_file,
            CompressionMethod::Deflated
        ).expect("Should be able to zip old proxy directory into new file.");

        copy_directory_recursive(
            self.backend_directory_path(),
            new_directory.path().join(self.backend_directory_name())
        ).expect("Should be able to copy backend/private directory.");

        ZippedDistributedFmu {
            name: String::from(self.name()),
            directory: FmuDirectory::Temporary(new_directory),
            file: File::open(&zip_file_path).expect("Should be able to open newly created zipfile."),
            language: self.language().clone(),
            version: self.version().clone(),
        }
    }

    fn persist_directory(mut self) {
        self.directory = self.directory.persist();
        println!("Persisted FMU in directory: {:?}", self.directory());
    }
}

impl PostGenerationSetup for DistributedFmu {
    fn backend_path(&self) -> PathBuf {
        self.backend_directory_path()
    }
}

impl DistributedFileStructure for DistributedFmu {}

impl RemoteBackend for DistributedFmu {}

impl Clone for DistributedFmu {
    fn clone(&self) -> Self {
        let new_directory = Self::new_tmp_dir();

        copy_directory_recursive(
            self.proxy_directory_path(),
            new_directory.path().join(self.proxy_directory_name())
        ).expect("Should be able to copy proxy directory.");

        copy_directory_recursive(
            self.backend_directory_path(),
            new_directory.path().join(self.backend_directory_name())
        ).expect("Should be able to copy backend/private directory.");
        
        Self {
            name: String::from(self.name()),
            directory: FmuDirectory::Temporary(new_directory),
            language: self.language().clone(),
            version: self.version().clone(),
        }
    }
}

impl BasicFmu for ZippedDistributedFmu {
    fn new(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> ZippedDistributedFmu {
        let directory = FmuDirectory::Temporary(Self::new_tmp_dir());

        let file = File::create(
            directory.path().join(ZippedDistributedFmu::with_zip_suffix(&name))
        ).expect("Should be able to create FMU zip file.");

        let this = ZippedDistributedFmu {
            name,
            file,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this
    }

    fn new_persistent(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage,
        container_directory_name: &str
    ) -> ZippedDistributedFmu {
        let container_directory_path = std::env::current_dir()
            .expect("Should be able to get current directory")
            .join(PERSITENTLY_CREATED_UNIFMUS_DIRECTORY)
            .join(container_directory_name);

        if container_directory_path.exists() {
            remove_dir_all(&container_directory_path)
                .expect("Should be able to remove old persisted FMU.");
        }

        create_dir_all(&container_directory_path)
            .expect("Should be able to create container directory for new persistent FMU.");

        let directory = FmuDirectory::Persistent(container_directory_path);

        let file = File::create(
            directory.path().join(ZippedDistributedFmu::with_zip_suffix(&name))
        ).expect("Should be able to create FMU zip file.");

        let this = ZippedDistributedFmu {
            name,
            file,
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this
    }

    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> ZippedDistributedFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (*ZIPPED_DISTRIBUTED_CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (*ZIPPED_DISTRIBUTED_JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (*ZIPPED_DISTRIBUTED_PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (*ZIPPED_DISTRIBUTED_PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate-distributed"),
            String::from(self.language().cmd_str()),
            String::from(self.name()),
        ];

        match self.version() {
            FmiVersion::Fmi2 => (),
            FmiVersion::Fmi3 => args.push(
                String::from(
                    self.version().as_str()
                )
            ),
        };

        args.push(String::from("--zipped"));

        args
    }

    fn directory(&self) -> &Path {
        self.directory.path()
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn importable_path(&self) -> PathBuf {
        self.proxy_directory_path()
    }

    fn is_zipped(&self) -> bool {
        true
    }

    #[allow(refining_impl_trait)]
    fn zipped(&self) -> ZippedDistributedFmu {
        self.clone()
    }

    fn persist_directory(mut self) {
        self.directory = self.directory.persist();
        println!("Persisted FMU in directory: {:?}", self.directory());
    }
}

impl Clone for ZippedDistributedFmu {
    fn clone(&self) -> Self {
        let new_directory = Self::new_tmp_dir();

        let proxy_file_name = self.proxy_directory_name();

        let new_proxy_file_path = new_directory.path().join(&proxy_file_name);

        copy(
            self.proxy_directory_path(),
            &new_proxy_file_path
        ).expect("Should be able to copy zip file.");

        copy_directory_recursive(
            self.backend_directory_path(),
            new_directory.path().join(self.backend_directory_name())
        ).expect("Should be able to copy backend/private directory.");
        
        Self {
            name: String::from(self.name()),
            file: File::open(&new_proxy_file_path).expect("Should be able to open new cloned zip file."),
            directory: FmuDirectory::Temporary(new_directory),
            language: self.language().clone(),
            version: self.version().clone()
        }
    }
}

impl DistributedFileStructure for ZippedDistributedFmu {
    fn proxy_directory_name(&self) -> String {
        Self::with_zip_suffix(
            &Self::append_proxy_marker(
                self.name()
            )
        )
    }
}

impl RemoteBackend for ZippedDistributedFmu {}

impl ZippedTestableFmu for ZippedDistributedFmu {
    fn file(self) -> File {
        self.file
    }

    fn zip_name(&self) -> String {
        self.proxy_directory_name()
    }
}

impl BasicFmu for BlackboxDistributedFmu {
    fn new(
        _name: String,
        _version: FmiVersion,
        _language: FmuBackendImplementationLanguage
    ) -> Self {
        todo!("Use new_persistent until implemented");
    }

    fn new_persistent(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage,
        container_directory_name: &str
    ) -> Self {
        let container_directory_path = std::env::current_dir()
            .expect("Should be able to get current directory")
            .join(PERSITENTLY_CREATED_UNIFMUS_DIRECTORY)
            .join(container_directory_name);

        if container_directory_path.exists() {
            remove_dir_all(&container_directory_path)
                .expect("Should be able to remove old persisted FMU.");
        }

        create_dir_all(&container_directory_path)
            .expect("Should be able to create container directory for new persistent FMU.");

        let directory = FmuDirectory::Persistent(container_directory_path);

        let file = File::create(
            directory.path().join(Self::with_zip_suffix(&name))
        ).expect("Should be able to create FMU zip file.");

        let black_box_container_directory_name = format!(
            "{}/{}",
            &container_directory_name,
            Self::append_backend_marker(&name)
        );

        let black_box_fmu = ZippedLocalFmu::new_persistent(
            String::from(&name),
            version.clone(),
            language,
            &black_box_container_directory_name
        );

        let this = BlackboxDistributedFmu {
            name,
            file,
            directory,
            version,
            black_box_fmu
        };

        this.generate_fmu_files();

        this
    }

    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> Self {
        match language {
            FmuBackendImplementationLanguage::CSharp => (*BLACKBOX_DISTRIBUTED_CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (*BLACKBOX_DISTRIBUTED_JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (*BLACKBOX_DISTRIBUTED_PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (*BLACKBOX_DISTRIBUTED_PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        // While this passes the backend language to the UniFMU
        // generate-distributed command this currently doesn't do anything as
        // all black-box containing (Uni)FMUs use a python interface in the
        // remote backend which then runs the contained black-box (whose
        // implementation we don't know or care about).
        // The language is passed because it is less brittle than hardcoding
        // a dummy value.
        let mut args = vec![
            String::from("generate-distributed"),
            String::from(self.language().cmd_str()), // <--
            String::from(self.name()),
        ];

        match self.version() {
            FmiVersion::Fmi2 => (),
            FmiVersion::Fmi3 => args.push(
                String::from(
                    self.version().as_str()
                )
            ),
        };

        args.push(String::from("--black-box-fmu"));
        args.push(String::from("--zipped"));

        args
    }

    fn directory(&self) -> &Path {
        self.directory.path()
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        self.black_box_fmu.language()
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn importable_path(&self) -> PathBuf {
        self.proxy_directory_path()
    }

    fn is_zipped(&self) -> bool {
        true
    }

    #[allow(refining_impl_trait)]
    fn zipped(&self) -> BlackboxDistributedFmu {
        self.clone()
    }

    fn persist_directory(mut self) {
        self.directory = self.directory.persist();
        println!("Persisted FMU in directory: {:?}", self.directory());
    }
}

impl Clone for BlackboxDistributedFmu {
    fn clone(&self) -> Self {
        let new_directory = Self::new_tmp_dir();

        let new_proxy_file_path = new_directory
            .path()
            .join(self.proxy_directory_name());

        let backend_destination = new_directory
            .path()
            .join(self.backend_directory_name());

        copy(
            self.proxy_directory_path(),
            &new_proxy_file_path
        ).expect("Should be able to copy zip file.");

        copy_directory_recursive(
            self.backend_directory_path(),
            &backend_destination
        ).expect("Should be able to copy backend/private directory.");

        let copied_black_box_zip_file = File::open(
            backend_destination
                .join(self.black_box_fmu.zip_name())
        ).expect("Should be able to open new cloned zip file.");

        let copied_black_box_fmu = ZippedLocalFmu {
            name: String::from(self.black_box_fmu.name()),
            file: copied_black_box_zip_file,
            directory: FmuDirectory::Persistent(backend_destination),
            language: self.black_box_fmu.language().clone(),
            version: self.black_box_fmu.version().clone()
        };

        Self {
            name: String::from(self.name()),
            file: File::open(&new_proxy_file_path).expect("Should be able to open new cloned zip file."),
            directory: FmuDirectory::Temporary(new_directory),
            version: self.version.clone(),
            black_box_fmu: copied_black_box_fmu
        }
    }
}

impl DistributedFileStructure for BlackboxDistributedFmu {
    fn proxy_directory_name(&self) -> String {
        Self::with_zip_suffix(
            &Self::append_proxy_marker(
                self.name()
            )
        )
    }
}

impl RemoteBackend for BlackboxDistributedFmu {}

impl ZippedTestableFmu for BlackboxDistributedFmu {
    fn file(self) -> File {
        self.file
    }

    fn zip_name(&self) -> String {
        self.proxy_directory_name()
    }
}

/// Behaviour shared by all FMUs.
pub trait BasicFmu {
    /// Generate a wholly new FMU using the UNIFMU CLI in a temporary
    /// directory.
    /// 
    /// If a new FMU is needed `get_clone()` should be called instead unless
    /// it is neccesarry to explicitly invoke the CLI for the test.
    fn new(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> Self;

    /// Generate a wholly new FMU using the UNIFMU CLI in a persistent
    /// directory.
    /// 
    /// If a new FMU is needed `get_clone()` should be called instead unless
    /// it is neccesarry to explicitly invoke the CLI for the test.
    fn new_persistent(
        name: String,
        version: FmiVersion,
        language: FmuBackendImplementationLanguage,
        container_directory_name: &str
    ) -> Self;

    /// Returns a copy of one of the lazily pregenerated FMUs. The copy is
    /// saved in a temporary directory.
    /// 
    /// Significantly faster than calling `new()`, while still giving a clean
    /// FMU instance.
    /// 
    /// This module has one FMU for each valid combination of FMI version,
    /// backend language and FMU location, that is lazily generated once this
    /// function is called. These FMUs are generated from the UNIFMU assets
    /// without further modification, and are never modified after creation.
    /// When cloned, all FMU assets are copied from the pregenerated one and
    /// collected in a new temporary directory. As such, calling `get_clone()`
    /// will return an effectively new FMU without invoking the UNIFMU CLI,
    /// from the second call in prgram execution and onwards.
    /// 
    /// The first time this is called with any set of parameters it takes as
    /// long as `new()`, but the second time it iscalled with the same
    /// parameter variants it will be much faster as the cloned FMU has already
    /// been generated. The difference is platform dependent, but two order of
    /// magnitude differences have been observed.
    /// 
    /// If it is neccesarry to explicitly invoke the UNIFMU CLI as part of the
    /// test `new()` should be used instead.
    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> Self;

    /// The arguments passed to the UNIFMU CLI command that will result in the
    /// generation of this FMU.
    fn cmd_args(&self) -> Vec<String>;

    fn directory(&self) -> &Path;

    fn language(&self) -> &FmuBackendImplementationLanguage;

    fn version(&self) -> &FmiVersion;

    /// Name of the importable part of the FMU (the directory/zip-file that
    /// contains the FMU binary). Only the base file name, not the full
    /// path name.
    fn name(&self) -> &str;

    /// Path to the directory/zip-file that contains the FMU binary.
    fn importable_path(&self) -> PathBuf;

    /// Whether or not the FMU is zipped or not.
    fn is_zipped(&self) -> bool;

    /// Returns a clone of the FMU but with the importable folder zipped.
    /// 
    /// All modifications on the cloned FMU will also be present in the
    /// zipped fmu.
    fn zipped(&self) -> impl ZippedTestableFmu;

    /// Ensures that rust wont clean up the FMUs directory when the FMU goes
    /// out of scope.
    /// 
    /// Does NOT ensure that the OS doesn't itself delete the directory (in
    /// case of automatic claners).
    fn persist_directory(self);

    fn generate_fmu_files(&self) {
        Command::cargo_bin("unifmu")
            .expect("The unifmu binary should be present in this crate.")
            .current_dir(self.directory())
            .args(self.cmd_args())
            .assert()
            .success()
            .stderr(contains("generated successfully"));
    }

    fn new_tmp_dir() -> TempDir {
        TempDir::new()
            .expect("Couldn't create temporary Fmu directory.")
    }
}

/// Behaviour for FMUs that can be reconfigured after generation
/// by the UNIFMU CLI.
/// 
/// Currently this is all non-zipped FMUs.
pub trait PostGenerationSetup: BasicFmu {
    fn backend_path(&self) -> PathBuf;
    
    /// Code to run after the fmu files have been generated, but before it
    /// is considered a valid, testable fmu.
    fn post_generation_setup(&self) {
        if self.language() == &FmuBackendImplementationLanguage::Java {
            self.java_setup()
        }
    }

    /// Compiles the Java using gradle to reduce test execution time.
    fn java_setup(&self) {
        let (
            shell_command,
            gradle_file
        ) = gradle_command_base();

        let gradle_build_command = duct::cmd!(
            shell_command, gradle_file, "compileJava", "--build-cache"
        );

        let gradle_build_command = gradle_build_command.dir(self.backend_path());

        gradle_build_command.run()
            .expect("Should be able to prebuild Javabased FMU");
    }
}

/// Behaviour for FMUs that can be generated by the UNIFMU CLI.
pub trait BreakableFmu: BasicFmu {
    fn model_file_path(&self) -> PathBuf;

    fn do_step_function_line_number(&self) -> u64;

    fn do_step_function_injection_prefix(&self) -> &str;

    /// Breaks the model by adding an error/exception to the first line of
    /// the code.
    fn inject_fault_into_backend_model_file(&self) {
        inject_line(
            &self.model_file_path(),
            self.language().fault_str(),
            1
        ).expect("Should be able to inject fault into model.");
    }

    /// Breaks the do_step function in the model by addind an error/exception
    /// after the function definition.
    fn inject_fault_into_backend_do_step_function(&self) {
        let injection = format!(
            "{}{}",
            self.do_step_function_injection_prefix(),
            self.language().fault_str()
        );

        inject_line(
            &self.model_file_path(),
            &injection,
            self.do_step_function_line_number()
        ).expect("Should be able to inject fault into model.");
    }
}

/// Behaviour for zipped FMUs.
pub trait ZippedTestableFmu: BasicFmu {
    /// Returns the FMU zipfile.
    fn file(self) -> File;

    fn zip_name(&self) -> String {
        Self::with_zip_suffix(self.name())
    }

    fn with_zip_suffix(name: &str) -> String {
        String::from(name) + ".fmu"
    }
}

/// Functionality for accessing the underlying file structure of
/// distributed FMUs
pub trait DistributedFileStructure: BasicFmu {
    fn proxy_directory_name(&self) -> String {
        Self::append_proxy_marker(self.name())
    }

    fn backend_directory_name(&self) -> String {
        Self::append_backend_marker(self.name())
    }

    fn proxy_directory_path(&self) -> PathBuf {
        self.directory()
            .join(self.proxy_directory_name())
    }

    fn backend_directory_path(&self) -> PathBuf {
        self.directory()
            .join(self.backend_directory_name())
    }

    fn append_proxy_marker(name: &str) -> String {
        String::from(name) + "_proxy"
    }

    fn append_backend_marker(name: &str) -> String {
        String::from(name) + "_private"
    }
}

/// Functionality for starting the private backend of distributed FMUs.
pub trait RemoteBackend: DistributedFileStructure {
    /// Constructs the command for running the remote backend process.
    fn get_remote_command(&self, port: String) -> duct::Expression {
        let backend_process_cmd = match self.language() {
            FmuBackendImplementationLanguage::CSharp => duct::cmd!(
                "dotnet", "run", "backend_head.cs", port
            ),
            FmuBackendImplementationLanguage::Java => {
                let args_list = format!("--args='{port}'");
                let (shell_command, gradle_file) = gradle_command_base();
                duct::cmd!(
                    shell_command, gradle_file, "run", args_list, "--build-cache"
                )
            },
            FmuBackendImplementationLanguage::Python => {
                // Unix systems differentiates version 2 and 3 of python in their binary names
                // Windows doesn't
                let python_interpreter_binary_name = match std::env::consts::OS {
                    "windows" => "python",
                    _other => "python3"
                };
                duct::cmd!(
                    python_interpreter_binary_name, "main.py", port
                )
            }
        };

        backend_process_cmd.dir(self.backend_directory_path())
    }

    /// Starts the remote backend telling it to connect to the dispatcher
    /// through the given port.
    /// 
    /// The subprocess containing the backend is started immediately and can be
    /// interacted with through the returned duct::Handle.
    fn start_remote_backend(&self, port: String) -> duct::Handle{
        let backend_process_cmd = self.get_remote_command(port);

        backend_process_cmd.start()
            .expect("Should be able to start the remote backend.")
    }
}

fn copy_directory_recursive(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>
)
    -> io::Result<()>
{
    create_dir(&destination)?;
    for entry in read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            copy_directory_recursive(
                entry.path(), 
                destination.as_ref().join(entry.file_name())
            )?;
        } else {
            copy(
                entry.path(), 
                destination.as_ref().join(entry.file_name())
            )?;
        }
    }
    Ok(())
}

/// Modifies the file at file_path by adding the injection at the line_number.
/// 
/// This does NOT overwrite the line. Instead, all content that is located at
/// and after the line at line_number is shifted one line, effectively increasing
/// the total number of lines by one.
fn inject_line(
    file_path: &PathBuf,
    injection: &str,
    line_number: u64
) -> io::Result<()> {
    let file = File::open(file_path)?;

    let lines = BufReader::new(&file).lines();

    let mut current_line_number: u64 = 1;

    let mut buffer = Vec::<u8>::new();

    for line in lines {
        let line = line?;
        if line_number == current_line_number {
            writeln!(buffer, "{}", injection)?;
        }
        writeln!(buffer, "{}", &line)?;
        current_line_number += 1;
    }

    let mut file = File::create(file_path)?;

    file.write_all(&buffer)?;

    Ok(())
}

static CSHARP_FMI2: LazyLock<LocalFmu> = LazyLock::new(|| {
    LocalFmu::new_persistent(
        String::from("csharp_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::CSharp,
        "PROMETHEAN_csharp_fmi2",
    )
});

static JAVA_FMI2: LazyLock<LocalFmu> = LazyLock::new(|| {
    LocalFmu::new_persistent(
        String::from("java_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Java,
        "PROMETHEAN_java_fmi2",
    )
});

static PYTHON_FMI2: LazyLock<LocalFmu> = LazyLock::new(|| {
    LocalFmu::new_persistent(
        String::from("python_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_python_fmi2",
    )
});

static PYTHON_FMI3: LazyLock<LocalFmu> = LazyLock::new(|| {
    LocalFmu::new_persistent(
        String::from("python_fmi3"),
        FmiVersion::Fmi3,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_python_fmi3",
    )
});

static ZIPPED_CSHARP_FMI2: LazyLock<ZippedLocalFmu> = LazyLock::new(|| {
    ZippedLocalFmu::new_persistent(
        String::from("zipped_csharp_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::CSharp,
        "PROMETHEAN_zipped_csharp_fmi2",
    )
});

static ZIPPED_JAVA_FMI2: LazyLock<ZippedLocalFmu> = LazyLock::new(|| {
    ZippedLocalFmu::new_persistent(
        String::from("zipped_java_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Java,
        "PROMETHEAN_zipped_java_fmi2",
    )
});

static ZIPPED_PYTHON_FMI2: LazyLock<ZippedLocalFmu> = LazyLock::new(|| {
    ZippedLocalFmu::new_persistent(
        String::from("zipped_python_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_zipped_python_fmi2",
    )
});

static ZIPPED_PYTHON_FMI3: LazyLock<ZippedLocalFmu> = LazyLock::new(|| {
    ZippedLocalFmu::new_persistent(
        String::from("zipped_python_fmi3"),
        FmiVersion::Fmi3,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_zipped_python_fmi3",
    )
});

static DISTRIBUTED_CSHARP_FMI2: LazyLock<DistributedFmu> = LazyLock::new(|| {
    DistributedFmu::new_persistent(
        String::from("distributed_csharp_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::CSharp,
        "PROMETHEAN_distributed_csharp_fmi2",
    )
});

static DISTRIBUTED_JAVA_FMI2: LazyLock<DistributedFmu> = LazyLock::new(|| {
    DistributedFmu::new_persistent(
        String::from("distributed_java_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Java,
        "PROMETHEAN_distributed_java_fmi2",
    )
});

static DISTRIBUTED_PYTHON_FMI2: LazyLock<DistributedFmu> = LazyLock::new(|| {
    DistributedFmu::new_persistent(
        String::from("distributed_python_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_distributed_python_fmi2",
    )
});

static DISTRIBUTED_PYTHON_FMI3: LazyLock<DistributedFmu> = LazyLock::new(|| {
    DistributedFmu::new_persistent(
        String::from("distributed_python_fmi3"),
        FmiVersion::Fmi3,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_distributed_python_fmi3",
    )
});

static ZIPPED_DISTRIBUTED_CSHARP_FMI2: LazyLock<ZippedDistributedFmu> = LazyLock::new(|| {
    ZippedDistributedFmu::new_persistent(
        String::from("zipped_distributed_csharp_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::CSharp,
        "PROMETHEAN_zipped_distributed_csharp_fmi2",
    )
});

static ZIPPED_DISTRIBUTED_JAVA_FMI2: LazyLock<ZippedDistributedFmu> = LazyLock::new(|| {
    ZippedDistributedFmu::new_persistent(
        String::from("zipped_distributed_java_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Java,
        "PROMETHEAN_zipped_distributed_java_fmi2",
    )
});

static ZIPPED_DISTRIBUTED_PYTHON_FMI2: LazyLock<ZippedDistributedFmu> = LazyLock::new(|| {
    ZippedDistributedFmu::new_persistent(
        String::from("zipped_distributed_python_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_zipped_distributed_python_fmi2",
    )
});

static ZIPPED_DISTRIBUTED_PYTHON_FMI3: LazyLock<ZippedDistributedFmu> = LazyLock::new(|| {
    ZippedDistributedFmu::new_persistent(
        String::from("zipped_distributed_python_fmi3"),
        FmiVersion::Fmi3,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_zipped_distributed_python_fmi3",
    )
});

static BLACKBOX_DISTRIBUTED_CSHARP_FMI2: LazyLock<BlackboxDistributedFmu> = LazyLock::new(|| {
    BlackboxDistributedFmu::new_persistent(
        String::from("blackbox_distributed_csharp_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::CSharp,
        "PROMETHEAN_blackbox_distributed_csharp_fmi2",
    )
});

static BLACKBOX_DISTRIBUTED_JAVA_FMI2: LazyLock<BlackboxDistributedFmu> = LazyLock::new(|| {
    BlackboxDistributedFmu::new_persistent(
        String::from("blackbox_distributed_java_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Java,
        "PROMETHEAN_blackbox_distributed_java_fmi2",
    )
});

static BLACKBOX_DISTRIBUTED_PYTHON_FMI2: LazyLock<BlackboxDistributedFmu> = LazyLock::new(|| {
    BlackboxDistributedFmu::new_persistent(
        String::from("blackbox_distributed_python_fmi2"),
        FmiVersion::Fmi2,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_blackbox_distributed_python_fmi2",
    )
});

static BLACKBOX_DISTRIBUTED_PYTHON_FMI3: LazyLock<BlackboxDistributedFmu> = LazyLock::new(|| {
    BlackboxDistributedFmu::new_persistent(
        String::from("blackbox_distributed_python_fmi3"),
        FmiVersion::Fmi3,
        FmuBackendImplementationLanguage::Python,
        "PROMETHEAN_blackbox_distributed_python_fmi3",
    )
});