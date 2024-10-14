use std::{
    ffi::OsStr,
    fs::{copy, File},
    io::{self, BufRead, Read, Write},
    sync::LazyLock
};

use assert_cmd::Command;
use predicates::str::contains;
use tempdir::TempDir;
use walkdir::WalkDir;
use zip::{CompressionMethod, write::FileOptions, ZipArchive, ZipWriter};

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
            FmuBackendImplementationLanguage::Java => "Model.java",
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
    pub file: File,
    pub language: FmuBackendImplementationLanguage,
    pub version: FmiVersion,
}

impl TestFmu {
    pub fn create_new(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> TestFmu {
        let directory = TempDir::new("FmuTestDir")
            .expect("Couldn't create temporary Fmu directory.");

        Command::cargo_bin("unifmu")
            .expect("The unifmu binary should be present in this crate.")
            .current_dir(directory.path())
            .args(Self::construct_cmd_args(version, language))
            .assert()
            .success()
            .stderr(contains("the FMU was generated successfully"));

        let file_path = directory.path().join(
            Self::construct_file_name(version, language)
        );

        TestFmu {
            directory,
            file: File::open(file_path)
                .expect("Fmu file should exist in temporary directory"),
            language: language.clone(),
            version: version.clone()
        }
    }

    pub fn get_clone(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> TestFmu {
        match language {
            FmuBackendImplementationLanguage::CSharp => &*CSHARP_FMI2.clone(),
            FmuBackendImplementationLanguage::Java => &*JAVA_FMI2.clone(),
            FmuBackendImplementationLanguage::Python => match version {
                FmiVersion::Fmi2 => &*PYTHON_FMI2.clone(),
                FmiVersion::Fmi3 => &*PYTHON_FMI3.clone()
            }
        }
    }

    pub fn inject_fault_in_model(&mut self, line_number: u64) {
        todo!("Open FMU dir, find correct file, call correct inject function.");
    }

    fn inject_in_csharp<T>(file: &mut File, writer: T, line_number: u64) 
    where
        T: Write,
    {
        todo!("Write a line of reasonable code at the target line.");
    }

    fn inject_in_java<T>(file: &mut File, writer: T, line_number: u64) 
    where
        T: Write,
    {
        todo!("Write a line of reasonable code at the target line.");
    }

    fn inject_in_python<T>(file: &mut File, writer: &mut T, line_number: u64) 
    where
        T: Write,
    {
        todo!("Write a line of reasonable code at the target line.");
    }

    fn construct_cmd_args(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from(language.cmd_str()),
            Self::construct_file_name(version, language),
        ];

        match version {
            FmiVersion::Fmi2 => args,
            FmiVersion::Fmi3 => args.push(String::from(version.as_str())),
        }
    }

    fn construct_file_name(
        version: &FmiVersion,
        language: &FmuBackendImplementationLanguage
    ) -> String {
        format!("{}fmu_{}.fmu", language.cmd_str(), version.as_str())
    }
}

impl Clone for TestFmu {
    fn clone(&self) -> Self {
        let new_directory = TempDir::new("FmuTestDir")
            .expect("Couldn't create temporary Fmu directory.");

        let old_file_path = self.directory.path().join(
            Self::construct_file_name(&self.version, &self.language)
        );

        let new_file_path = new_directory.path().join(
            Self::construct_file_name(&self.version, &self.language)
        );

        let new_file = File::create_new(
            new_directory
                .path()
                .join(Self::construct_file_name(
                    &self.version, &self.language
                ))
        ).expect("Should have been able to create new file in new temporary directory.");

        copy(old_file_path, new_file_path)
            .expect("Should have been able to copy files between test directories.");

        Self {
            directory: new_directory,
            file: new_file,
            language: self.language.clone(),
            version: self.version.clone(),
        }
    }
}