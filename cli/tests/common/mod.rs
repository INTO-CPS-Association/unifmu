use std::{
    fs::{copy, create_dir, read_dir, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{mpsc::Sender, LazyLock}
};

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use tokio::{
    io::AsyncBufReadExt,
    process,
    select
};
use unifmu::utils::zip_dir;
use walkdir::WalkDir;
use zip::CompressionMethod;

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

pub struct RemoteFmu {
    directory: TempDir,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

pub struct ZippedRemoteFmu {
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

impl ZippedTestableFmu for ZippedLocalFmu {
    fn file(self) -> File {
        self.file
    }
}

impl BasicFmu for RemoteFmu {
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
        format!("remote_{}fmu_{}", language.cmd_str(), version.as_str())
    }
}

impl RemoteFileStructure for RemoteFmu {}

impl RemoteBackend for RemoteFmu {}

impl TestableFmu for RemoteFmu {
    /// Creates an entirely new FMU in a temporary directory.
    fn new(
        version: FmiVersion,
        language: FmuBackendImplementationLanguage
    ) -> RemoteFmu {
        let directory = Self::new_tmp_dir();

        let this = RemoteFmu {
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
    ) -> RemoteFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (&*REMOTE_CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (&*REMOTE_JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (&*REMOTE_PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (&*REMOTE_PYTHON_FMI3).clone()
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

    fn model_file_path(&self) -> PathBuf {
        self.backend_directory_path()
            .join("resources")
            .join(self.language.model_str())
    }

    /// Copies the contents of this FMU and zips them in a new temporary
    /// directory, returning a ZippedTestFmu with the zipped file.
    fn zipped(&self) -> impl ZippedTestableFmu {
        let new_directory = TempDir::new()
            .expect("Should be able to create new temporary FMU directory.");

        let proxy_fmu_name = self.proxy_directory_name();

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

        ZippedRemoteFmu {
            directory: new_directory,
            file: File::open(&zip_file_path).expect("Should be able to open newly created zipfile."),
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}

impl Clone for RemoteFmu {
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

impl BasicFmu for ZippedRemoteFmu {
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
        format!("remote_{}fmu_{}", language.cmd_str(), version.as_str())
    }
}

impl RemoteFileStructure for ZippedRemoteFmu {}

impl RemoteBackend for ZippedRemoteFmu {}

impl ZippedTestableFmu for ZippedRemoteFmu {
    fn file(self) -> File {
        self.file
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
    fn break_model(&self) {
        inject_line(
            &self.model_file_path(),
            self.language().fault_str(),
            1
        ).expect("Should be able to inject fault into model.");
    }

    /// Breaks the do_step function in the model by addind an error/exception
    /// after the function definition.
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

pub trait ZippedTestableFmu {
    fn file(self) -> File;
}

pub trait RemoteFileStructure: BasicFmu {
    fn proxy_directory_name(&self) -> String {
        Self::fmu_name(&self.version(), &self.language()) + "_proxy"
    }

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

pub trait RemoteBackend: RemoteFileStructure {
    fn get_remote_command(&self, port: String) -> process::Command {
        let mut backend_process_cmd = match self.language() {
            FmuBackendImplementationLanguage::CSharp => process::Command::new("dotnet"),
            FmuBackendImplementationLanguage::Java => process::Command::new("sh"),
            FmuBackendImplementationLanguage::Python => process::Command::new("python3")
        };

        match self.language() {
            FmuBackendImplementationLanguage::CSharp => {
                backend_process_cmd
                    .current_dir(self.backend_directory_path())
                    .arg("run")
                    .arg("backend.cs")
                    .arg(port)
            },
            FmuBackendImplementationLanguage::Java => {
                backend_process_cmd
                    .current_dir(self.backend_directory_path())
                    .arg("gradlew")
                    .arg("run")
                    .arg(format!("--args='{}'", port))
            },
            FmuBackendImplementationLanguage::Python => {
                backend_process_cmd
                    .arg(self.backend_directory_path().join("backend.py"))
                    .arg(port)
            }
        };

        return backend_process_cmd
    }

    fn start_remote_backend(
        &self,
        port: String,
        stdout_channel_tx: Sender<String>,
        stderr_channel_tx: Sender<String>,
    ) {
        let mut backend_process_cmd = self.get_remote_command(port);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Should be able to start tokio runtime.");

        let mut backend_process = runtime.block_on( async {
            backend_process_cmd
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Should be able to spawn the remote backend.")
        });

        let mut stdout_lines = tokio::io::BufReader::new(
            backend_process.stdout
                .take()
                .unwrap()
        )
            .lines();

        let mut stderr_lines = tokio::io::BufReader::new(
            backend_process.stderr
                .take()
                .unwrap()
        )
            .lines();

        let stdout_future = async {
            while let Some(line) = stdout_lines.next_line()
                .await
                .expect("Should be able to get stdout from process.")
            {
                stdout_channel_tx.send(line)
                    .expect("Should be able to send backend stdout through channel.");
            }
        };

        let stderr_future = async {
            while let Some(line) = stderr_lines.next_line()
                .await
                .expect("Should be able to get stderr from process.")
            {
                stderr_channel_tx.send(line)
                    .expect("Should be able to send backend stderr through channel.");
            }
        };

        runtime.block_on(async {
            select! {
                _ = async {
                    tokio::join!(stdout_future, stderr_future)
                } => {},
                _ = async {
                    backend_process.wait()
                        .await
                        .expect("Should be able to run the remote backend.")
                } => {}
            }
        })
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

static REMOTE_CSHARP_FMI2: LazyLock<RemoteFmu> = LazyLock::new(|| {RemoteFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::CSharp,
)});

static REMOTE_JAVA_FMI2: LazyLock<RemoteFmu> = LazyLock::new(|| {RemoteFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::CSharp,
)});

static REMOTE_PYTHON_FMI2: LazyLock<RemoteFmu> = LazyLock::new(|| {RemoteFmu::new(
    FmiVersion::Fmi2,
    FmuBackendImplementationLanguage::Python,
)});

static REMOTE_PYTHON_FMI3: LazyLock<RemoteFmu> = LazyLock::new(|| {RemoteFmu::new(
    FmiVersion::Fmi3,
    FmuBackendImplementationLanguage::Python,
)});