use clap::ArgEnum;
use fs_extra::dir::CopyOptions;
use lazy_static::lazy_static;
use log::info;
use rust_embed::RustEmbed;
use std::{fs::File, path::Path};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::{result::ZipError, CompressionMethod};

use crate::utils::zip_dir;

#[macro_use]
extern crate dlopen_derive;

#[derive(ArgEnum, Clone, Debug)]
pub enum Language {
    Python,
    CSharp,
    Matlab,
    Java,
}

#[derive(ArgEnum, Clone, Debug)]
pub enum FmiFmuVersion {
    FMI2,
    FMI3,
}

#[derive(RustEmbed)]
#[folder = "../assets"]
struct Assets;

pub mod benchmark;
pub mod utils;
pub mod validation;

struct LanguageAssets {
    fmi2_resources: Vec<(&'static str, &'static str)>,
    fmi3_resources: Vec<(&'static str, &'static str)>,
}

lazy_static! {
    static ref PYTHONASSETS: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("python/fmi2/backend.py", "backend.py"),
            ("python/fmi2/model.py", "model.py"),
            (
                "auto_generated/fmi2_messages_pb2.py",
                "schemas/fmi2_messages_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi2/README.md", "README.md"),
        ],
        fmi3_resources: vec![
            ("python/fmi3/backend.py", "backend.py"),
            ("python/fmi3/model.py", "model.py"),
            (
                "auto_generated/fmi3_messages_pb2.py",
                "schemas/fmi3_messages_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi3/README.md", "README.md"),
        ],
    };
    static ref CSHARPASSETS: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("csharp/backend.cs", "backend.cs"),
            ("csharp/model.cs", "model.cs"),
            ("csharp/model.csproj", "model.csproj"),
            ("auto_generated/Fmi2Messages.cs", "schemas/Fmi2Messages.cs"),
            ("csharp/launch.toml", "launch.toml"),
            ("csharp/README.md", "README.md"),
        ],
        fmi3_resources: vec![
            ("docker/Dockerfile_csharp", "Dockerfile"),
            ("docker/deploy_csharp.py", "deploy.py"),
            ("docker/docker-compose.yml", "docker-compose.yml"),
            ("docker/launch_csharp.toml", "launch.toml"),
            ("docker/README.md", "README_DOCKER.md"),
        ],
    };
    static ref JAVAASSETS: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            (
                "java/src/main/java/Backend.java",
                "src/main/java/Backend.java"
            ),
            ("java/src/main/java/Model.java", "src/main/java/Model.java"),
            ("java/build.gradle", "build.gradle"),
            ("java/gradlew", "gradlew"),
            (
                "java/gradle/wrapper/gradle-wrapper.jar",
                "gradle/wrapper/gradle-wrapper.jar"
            ),
            (
                "java/gradle/wrapper/gradle-wrapper.properties",
                "gradle/wrapper/gradle-wrapper.properties"
            ),
            ("java/gradlew.bat", "gradlew.bat"),
            ("java/launch.toml", "launch.toml"),
            ("java/README.md", "README.md"),
            (
                "auto_generated/Fmi2Messages.java",
                "src/main/java/Fmi2Messages.java"
            ),
        ],
        fmi3_resources: vec![
            ("docker/Dockerfile_csharp", "Dockerfile"),
            ("docker/deploy_csharp.py", "deploy.py"),
            ("docker/docker-compose.yml", "docker-compose.yml"),
            ("docker/launch_csharp.toml", "launch.toml"),
            ("docker/README.md", "README_DOCKER.md"),
        ],
    };
}

#[derive(Debug)]
pub enum GenerateError {
    Error,
    FileExists,
    ZipError(ZipError),
}

pub fn generate(
    language: &Language,
    fmu_version: &FmiFmuVersion,
    outpath: &Path,
    zipped: bool,
) -> Result<(), GenerateError> {
    let tmpdir = TempDir::new().unwrap();

    info!(
        "Generating FMU for language '{:?}' with tmpdir {:?} and final output path {:?}",
        language,
        tmpdir.path(),
        outpath
    );

    // copy common files to root directory and binaries
    let (bin_macos, bin_win, bin_linux) = match fmu_version {
        FmiFmuVersion::FMI2 => {
            let bin_macos = tmpdir
                .path()
                .join("binaries")
                .join("darwin64")
                .join("unifmu.dylib");
            let bin_win = tmpdir
                .path()
                .join("binaries")
                .join("win64")
                .join("unifmu.dll");
            let bin_linux = tmpdir
                .path()
                .join("binaries")
                .join("linux64")
                .join("unifmu.so");

            (bin_macos, bin_win, bin_linux)
        }
        FmiFmuVersion::FMI3 => {
            let bin_macos = tmpdir
                .path()
                .join("binaries")
                .join("x86_64-darwin")
                .join("unifmu.dylib");
            let bin_win = tmpdir
                .path()
                .join("binaries")
                .join("x86_64-windows")
                .join("unifmu.dll");
            let bin_linux = tmpdir
                .path()
                .join("binaries")
                .join("x86_64-linux")
                .join("unifmu.so");

            (bin_macos, bin_win, bin_linux)
        }
    };

    for p in vec![&bin_macos, &bin_win, &bin_linux] {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    }

    let md = tmpdir.path().join("modelDescription.xml");

    info!("{:?}", &bin_win);
    std::fs::write(
        bin_win,
        Assets::get("auto_generated/unifmu.dll").unwrap().data,
    )
    .unwrap();
    std::fs::write(
        bin_linux,
        Assets::get("auto_generated/unifmu.so").unwrap().data,
    )
    .unwrap();
    std::fs::write(
        bin_macos,
        Assets::get("auto_generated/unifmu.dylib").unwrap().data,
    )
    .unwrap();

    // copy language specific files to 'resources' directory

    let copy_to_resources = |assets: &LanguageAssets| {
        let assets_all = match fmu_version {
            FmiFmuVersion::FMI2 => assets.fmi2_resources.to_owned(),
            FmiFmuVersion::FMI3 => assets.fmi3_resources.to_owned(),
        };

        match fmu_version {
            FmiFmuVersion::FMI2 => {
                std::fs::write(
                    &md,
                    Assets::get("common/fmi2/modelDescription.xml")
                        .unwrap()
                        .data,
                )
                .unwrap();
            }
            FmiFmuVersion::FMI3 => {
                std::fs::write(
                    &md,
                    Assets::get("common/fmi3/modelDescription.xml")
                        .unwrap()
                        .data,
                )
                .unwrap();
            }
        }

        for (src, dst) in assets_all {
            let dst_resources = tmpdir.path().join("resources").join(dst);

            info!("copying resource {:?} to {:?}", src, dst_resources);
            std::fs::create_dir_all(dst_resources.parent().unwrap()).unwrap();
            std::fs::write(dst_resources, Assets::get(src).unwrap().data).unwrap();
        }
    };

    match language {
        Language::Python => copy_to_resources(&PYTHONASSETS),

        Language::CSharp => copy_to_resources(&CSHARPASSETS),

        Language::Matlab => todo!(),

        Language::Java => copy_to_resources(&JAVAASSETS),
    };

    match zipped {
        // zip to temporary, change extension from 'zip' to 'fmu', then copy to output directory
        true => {
            info!("Compressing contents into archive with path {:?}", outpath);

            let file = match File::create(&outpath) {
                Ok(f) => f,
                Err(_) => return Err(GenerateError::FileExists),
            };

            let walkdir = WalkDir::new(tmpdir.path());
            let it = walkdir.into_iter();

            let method = CompressionMethod::Deflated;

            match zip_dir(
                &mut it.filter_map(|e| e.ok()),
                tmpdir.path().to_str().unwrap(),
                file,
                method,
            ) {
                Ok(_) => (),
                Err(e) => return Err(GenerateError::ZipError(e)),
            }
            Ok(())
        }

        // copy from temporary directory to output directory
        false => {
            info!(
                "copying temporary dir from {:?} to output {:?}",
                tmpdir.path(),
                outpath,
            );
            let mut options = CopyOptions::default();
            options.copy_inside = true;
            options.content_only = true;
            options.overwrite = true;
            fs_extra::dir::move_dir(tmpdir, outpath, &options).unwrap();
            Ok(())
        }
    }
}
