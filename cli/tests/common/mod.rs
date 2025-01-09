use std::{
    ffi::OsString,
    fs::{copy, create_dir, read_dir, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::LazyLock
};

use assert_cmd::Command;
use duct;
use predicates::str::contains;
use tempfile::TempDir;
use unifmu::utils::zip_dir;
use walkdir::WalkDir;
use zip::CompressionMethod;

pub fn fmu_python_test(
    fmu: impl TestableFmu,
    python_test_function_name: &str
) {
    let python_test_process = start_python_test_process(
        python_test_function_name,
        fmu.importable_path()
    );

    let python_test_output_reader = BufReader::new(&python_test_process);

    for read_result in python_test_output_reader.lines() {
        match read_result {
            Ok(line) => {
                if line.contains("TEST FAILED") {
                    panic!("PYTHON {line}");
                
                } else {
                    println!("{line}");
                }
            },
            Err(e) => {
                panic!("Reading output from python test script returned error '{e}'");
            }
        }
    }
}

pub fn distributed_fmu_python_test(
    fmu: DistributedFmu,
    python_test_function_name: &str
) {
    let python_test_process = start_python_test_process(
        python_test_function_name,
        fmu.importable_path()
    );

    let python_test_output_reader = BufReader::new(&python_test_process);

    let mut remote_process: Option<duct::Handle> = None;

    for read_result in python_test_output_reader.lines() {
        match read_result {
            Ok(line) => {
                if line.contains("TEST FAILED") {
                    if remote_process.is_some() {
                        let _ = remote_process.unwrap().kill();
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
                if remote_process.is_some() {
                    let _ = remote_process.unwrap().kill();
                }

                panic!("Reading output from python test script returned error '{e}'");
            }
        }
    }

    if remote_process.is_none() {
        panic!("Remote backend wasn't started!");
    } else {
        let _ = remote_process.unwrap().wait();
    }
}

fn start_python_test_process(
    python_test_function_name: &str,
    fmu_path: impl Into<OsString>
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

    duct::cmd!(
        "python3",
        python_test_script_name,
        python_test_function_name,
        fmu_path
    )
        .dir(test_directory)
        .stderr_to_stdout()
        .reader()
        .expect("Should be able to start python test process")
}

pub fn vdm_check(fmu: impl TestableFmu) {
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

#[derive(Clone)]
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
pub struct LocalFmu {
    directory: TempDir,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

pub struct ZippedLocalFmu {
    file: File,
    directory: TempDir,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

pub struct DistributedFmu {
    directory: TempDir,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

pub struct ZippedDistributedFmu {
    file: File,
    directory: TempDir,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

impl BasicFmu for LocalFmu {
    fn directory(&self) -> &TempDir {
        &self.directory
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn fmu_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String {
        format!("{}fmu_{}.fmu", language.cmd_str(), version.as_str())
    }
}

impl TestableFmu for LocalFmu {
    /// Creates an entirely new FMU in a temporary directory.
    fn new(
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> LocalFmu {
        let directory = Self::new_tmp_dir();

        let this = LocalFmu {
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this
    }

    /// Gets a clone of a lazily evaluated static FMU wit the
    /// given FMI version and backend implementation language.
    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> LocalFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (&*CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (&*JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (&*PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (&*PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from(self.language().cmd_str()),
            Self::fmu_name(self.version(), self.language()),
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

    fn importable_path(&self) -> PathBuf {
        self.directory()
            .path()
            .join(Self::fmu_name(
                self.version(),
                self.language()
            ))
    }

    fn model_file_path(&self) -> PathBuf {
        self.directory.path()
            .join(Self::fmu_name(&self.version, &self.language))
            .join("resources")
            .join(self.language.model_str())
    }

    /// Copies the contents of this FMU and zips them in a new temporary
    /// directory, returning a ZippedTestFmu with the zipped file.
    fn zipped(&self) -> impl ZippedTestableFmu {
        let new_directory = TempDir::new()
            .expect("Should be able to create new temporary FMU directory.");

        let fmu_name = Self::fmu_name(&self.version, &self.language);

        let zip_file_path = new_directory
            .path()
            .join(&fmu_name);

        let zip_file = File::create(&zip_file_path)
            .expect("Should be able to create new FMU zipfile.");

        let old_fmu_directory = self.directory.path().join(&fmu_name);
        
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
            directory: new_directory,
            file: File::open(&zip_file_path).expect("Should be able to open newly created zipfile."),
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}

impl Clone for LocalFmu {
    fn clone(&self) -> Self {
        let new_directory = Self::new_tmp_dir();

        copy_directory_recursive(
            &self.directory
                .path()
                .join(Self::fmu_name(&self.version, &self.language)), 
            &new_directory
                .path()
                .join(Self::fmu_name(&self.version, &self.language))
        )
            .expect("Should be able to recursively copy cloned FMU into new FMU path");
        
        Self {
            directory: new_directory,
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}

impl BasicFmu for ZippedLocalFmu {
    fn directory(&self) -> &TempDir {
        &self.directory
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn fmu_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String {
        format!("{}fmu_{}.fmu", language.cmd_str(), version.as_str())
    }
}

impl ZippedTestableFmu for ZippedLocalFmu {
    fn file(self) -> File {
        self.file
    }

    fn importable_path(&self) -> PathBuf {
        self.directory()
            .path()
            .join(Self::fmu_name(
                self.version(),
                self.language()
            ))
    }
}

impl BasicFmu for DistributedFmu {
    fn directory(&self) -> &TempDir {
        &self.directory
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn fmu_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String {
        format!("distributed_{}fmu_{}", language.cmd_str(), version.as_str())
    }
}

impl DistributedFileStructure for DistributedFmu {
    fn proxy_directory_name(&self) -> String {
        Self::fmu_name(&self.version(), &self.language()) + "_proxy"
    }
}

impl RemoteBackend for DistributedFmu {}

impl TestableFmu for DistributedFmu {
    /// Creates an entirely new FMU in a temporary directory.
    fn new(
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> DistributedFmu {
        let directory = Self::new_tmp_dir();

        let this = DistributedFmu {
            directory,
            language,
            version
        };

        this.generate_fmu_files();

        this
    }

    /// Gets a clone of a lazily evaluated static FMU wit the
    /// given FMI version and backend implementation language.
    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> DistributedFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (&*DISTRIBUTED_CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (&*DISTRIBUTED_JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (&*DISTRIBUTED_PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (&*DISTRIBUTED_PYTHON_FMI3).clone()
            }
        }
    }

    fn cmd_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate-distributed"),
            String::from(self.language().cmd_str()),
            Self::fmu_name(self.version(), self.language()),
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
    
    fn importable_path(&self) -> PathBuf {
        self.proxy_directory_path()
    }

    fn model_file_path(&self) -> PathBuf {
        self.backend_directory_path()
            .join(self.language.model_str())
    }

    #[allow(refining_impl_trait)]
    /// Copies the contents of this FMU and zips them in a new temporary
    /// directory, returning a ZippedTestFmu with the zipped file.
    fn zipped(&self) -> ZippedDistributedFmu {
        let new_directory = TempDir::new()
            .expect("Should be able to create new temporary FMU directory.");

        let mut proxy_fmu_name = self.proxy_directory_name();
        proxy_fmu_name.push_str(".fmu");

        let zip_file_path = new_directory
            .path()
            .join(&proxy_fmu_name);

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
            directory: new_directory,
            file: File::open(&zip_file_path).expect("Should be able to open newly created zipfile."),
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}

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
            directory: new_directory,
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}

impl BasicFmu for ZippedDistributedFmu {
    fn directory(&self) -> &TempDir {
        &self.directory
    }

    fn language(&self) -> &FmuBackendImplementationLanguage {
        &self.language
    }

    fn version(&self) -> &FmiVersion {
        &self.version
    }

    fn fmu_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String {
        format!("distributed_{}fmu_{}", language.cmd_str(), version.as_str())
    }
}

impl DistributedFileStructure for ZippedDistributedFmu {
    fn proxy_directory_name(&self) -> String {
        Self::fmu_name(&self.version(), &self.language()) + "_proxy.fmu"
    }
}

impl RemoteBackend for ZippedDistributedFmu {}

impl ZippedTestableFmu for ZippedDistributedFmu {
    fn file(self) -> File {
        self.file
    }

    fn importable_path(&self) -> PathBuf {
        self.proxy_directory_path()
    }
}

pub trait BasicFmu {
    fn directory(&self) -> &TempDir;

    fn language(&self) -> &FmuBackendImplementationLanguage;

    fn version(&self) -> &FmiVersion;

    fn fmu_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String;
}

pub trait TestableFmu: BasicFmu {
    fn new(
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> Self;

    fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> Self;

    fn cmd_args(&self) -> Vec<String>;

    fn importable_path(&self) -> PathBuf;

    fn model_file_path(&self) -> PathBuf;

    fn zipped(&self) -> impl ZippedTestableFmu;

    fn new_tmp_dir() -> TempDir {
        TempDir::new()
            .expect("Couldn't create temporary Fmu directory.")
    }

    fn generate_fmu_files(&self) {
        Command::cargo_bin("unifmu")
            .expect("The unifmu binary should be present in this crate.")
            .current_dir(&self.directory().path())
            .args(&self.cmd_args())
            .assert()
            .success()
            .stderr(contains("generated successfully"));
    }

    /// Breaks the model by adding an error/exception to the first line of
    /// the code.
    /// 
    /// CURRENTLY DOESN'T WORK FOR DISTRIBUTED FMUS
    fn break_model(&self) {
        inject_line(
            &self.model_file_path(),
            self.language().fault_str(),
            1
        ).expect("Should be able to inject fault into model.");
    }

    /// Breaks the do_step function in the model by addind an error/exception
    /// after the function definition.
    /// 
    /// CURRENTLY DOESN'T WORK FOR DISTRIBUTED FMUS
    fn break_do_step_function(&self) {
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

    fn do_step_function_line_number(&self) -> u64 {
        match self.language() {
            FmuBackendImplementationLanguage::CSharp => 48,
            FmuBackendImplementationLanguage::Java => 47,
            FmuBackendImplementationLanguage::Python => {
                match self.version() {
                    FmiVersion::Fmi2 => 37,
                    FmiVersion::Fmi3 => 116
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

pub trait ZippedTestableFmu: BasicFmu {
    fn file(self) -> File;

    fn importable_path(&self) -> PathBuf;
}

pub trait DistributedFileStructure: BasicFmu {
    fn proxy_directory_name(&self) -> String;

    fn proxy_directory_path(&self) -> PathBuf {
        self.directory()
            .path()
            .join(self.proxy_directory_name())
    }

    fn backend_directory_name(&self) -> String {
        Self::fmu_name(&self.version(), &self.language()) + "_private"
    }

    fn backend_directory_path(&self) -> PathBuf {
        self.directory()
            .path()
            .join(self.backend_directory_name())
    }
}

pub trait RemoteBackend: DistributedFileStructure {
    /// Constructs the command for running the remote backend process.
    fn get_remote_command(&self, port: String) -> duct::Expression {
        let backend_process_cmd = match self.language() {
            FmuBackendImplementationLanguage::CSharp => duct::cmd!(
                "dotnet", "run", "backend.cs", port
            ),
            FmuBackendImplementationLanguage::Java => duct::cmd!(
                "sh", "gradlew", "run", "--args='{port}'"
            ),
            FmuBackendImplementationLanguage::Python => duct::cmd!(
                "python3", "backend.py", port
            )
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

static CSHARP_FMI2: LazyLock<LocalFmu> = LazyLock::new(|| {LocalFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::CSharp,
)});

static JAVA_FMI2: LazyLock<LocalFmu> = LazyLock::new(|| {LocalFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::CSharp,
)});

static PYTHON_FMI2: LazyLock<LocalFmu> = LazyLock::new(|| {LocalFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::Python,
)});

static PYTHON_FMI3: LazyLock<LocalFmu> = LazyLock::new(|| {LocalFmu::new(
    FmiVersion::Fmi3,
    FmuBackendImplementationLanguage::Python,
)});

static DISTRIBUTED_CSHARP_FMI2: LazyLock<DistributedFmu> = LazyLock::new(|| {DistributedFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::CSharp,
)});

static DISTRIBUTED_JAVA_FMI2: LazyLock<DistributedFmu> = LazyLock::new(|| {DistributedFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::CSharp,
)});

static DISTRIBUTED_PYTHON_FMI2: LazyLock<DistributedFmu> = LazyLock::new(|| {DistributedFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::Python,
)});

static DISTRIBUTED_PYTHON_FMI3: LazyLock<DistributedFmu> = LazyLock::new(|| {DistributedFmu::new(
    FmiVersion::Fmi3,
    FmuBackendImplementationLanguage::Python,
)});