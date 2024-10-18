use std::{
    fs::{copy, create_dir, File, read_dir},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    sync::LazyLock
};

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use unifmu::utils::zip_dir;
use walkdir::WalkDir;
use zip::CompressionMethod;

static CSHARP_FMI2: LazyLock<TestFmu> = LazyLock::new(|| {TestFmu::create_new(
    &FmiVersion::Fmi2,
    &FmuBackendImplementationLanguage::CSharp,
)});

static JAVA_FMI2: LazyLock<TestFmu> = LazyLock::new(|| {TestFmu::create_new(
    &FmiVersion::Fmi2,
    &FmuBackendImplementationLanguage::Java,
)});

static PYTHON_FMI2: LazyLock<TestFmu> = LazyLock::new(|| {TestFmu::create_new(
    &FmiVersion::Fmi2,
    &FmuBackendImplementationLanguage::Python,
)});

static PYTHON_FMI3: LazyLock<TestFmu> = LazyLock::new(|| {TestFmu::create_new(
    &FmiVersion::Fmi3,
    &FmuBackendImplementationLanguage::Python,
)});

#[derive(Clone)]
pub enum FmuBackendImplementationLanguage {
    Python,
    Java,
    CSharp
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
            FmuBackendImplementationLanguage::Java => "throw new Exception();",
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

pub struct TestFmu {
    pub directory: TempDir,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
}

impl TestFmu {
    pub fn create_new(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> TestFmu {
        let directory = TempDir::new()
            .expect("Couldn't create temporary Fmu directory.");

        Command::cargo_bin("unifmu")
            .expect("The unifmu binary should be present in this crate.")
            .current_dir(directory.path())
            .args(Self::construct_cmd_args(version, language))
            .assert()
            .success()
            .stderr(contains("the FMU was generated successfully"));

        TestFmu {
            directory,
            language: language.clone(),
            version: version.clone()
        }
    }

    pub fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> TestFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => (&*CSHARP_FMI2).clone(),
            FmuBackendImplementationLanguage::Java => (&*JAVA_FMI2).clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => (&*PYTHON_FMI2).clone(),
                FmiVersion::Fmi3 => (&*PYTHON_FMI3).clone()
            }
        }
    }

    pub fn break_model(&self) {
        inject_line(
            &self.get_model_file_path(),
            self.language.fault_str(),
            1
        ).expect("Should be able to inject fault into model.");
    }

    pub fn break_do_step_function(&self) {
        let injection = format!("        {}", self.language.fault_str()); 
        inject_line(
            &self.get_model_file_path(),
            &injection,
            self.get_do_step_function_line_number()
        ).expect("Should be able to inject fault into model.");
    }

    /// Copies the contents of this FMU and zips them in a new temporary directory.
    pub fn zipped(&self) -> ZippedTestFmu {
        let new_directory = TempDir::new()
            .expect("Should be able to create new temporary FMU directory.");

        let fmu_name = Self::construct_fmu_name(&self.version, &self.language);

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

        ZippedTestFmu {
            directory: new_directory,
            file: File::open(&zip_file_path).expect("Should be able to open newly created zipfile."),
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }

    fn get_model_file_path(&self) -> PathBuf {
        self.directory.path()
            .join(Self::construct_fmu_name(&self.version, &self.language))
            .join("resources")
            .join(self.language.model_str())
    }

    fn get_do_step_function_line_number(&self) -> u64 {
        match self.language {
            FmuBackendImplementationLanguage::CSharp => 48,
            FmuBackendImplementationLanguage::Java => 47,
            FmuBackendImplementationLanguage::Python => {
                match self.version {
                    FmiVersion::Fmi2 => 34,
                    FmiVersion::Fmi3 => 116
                }
            },
        }
    }

    fn construct_cmd_args(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from(language.cmd_str()),
            Self::construct_fmu_name(version, language),
        ];

        match version {
            FmiVersion::Fmi2 => (),
            FmiVersion::Fmi3 => args.push(String::from(version.as_str())),
        };

        args
    }

    fn construct_fmu_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String {
        format!("{}fmu_{}.fmu", language.cmd_str(), version.as_str())
    }
}

impl Clone for TestFmu {
    fn clone(&self) -> Self {
        let new_directory = TempDir::new()
            .expect("Couldn't create temporary FMU directory.");

        copy_directory_recursive(
            &self.directory
                .path()
                .join(Self::construct_fmu_name(&self.version, &self.language)), 
            &new_directory
                .path()
                .join(Self::construct_fmu_name(&self.version, &self.language))
        )
            .expect("Should be able to recursively copy cloned FMU into new FMU path");
        
        Self {
            directory: new_directory,
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}

pub struct ZippedTestFmu {
    pub directory: TempDir,
    pub file: File,
    language: FmuBackendImplementationLanguage,
    version: FmiVersion,
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

    let lines = io::BufReader::new(&file).lines();

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