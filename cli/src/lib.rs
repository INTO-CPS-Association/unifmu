use clap::ValueEnum;
use fs_extra::dir::CopyOptions;
use lazy_static::lazy_static;
use log::info;
use log::error;
use log::warn;
use rust_embed::RustEmbed;
use std::{fs::File, path::{Path, PathBuf}};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::{result::ZipError, CompressionMethod};
use serde::Deserialize;
use serde::Serialize;
use std::fs;


use crate::utils::zip_dir;

extern crate dlopen_derive;

#[derive(ValueEnum, Clone, Debug)]
pub enum Language {
    Python,
    CSharp,
    Java,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum FmiFmuVersion {
    FMI2,
    FMI3,
}

#[derive(RustEmbed)]
#[folder = "../assets"]
struct Assets;

pub mod utils;

struct LanguageAssets {
    fmi2_resources: Vec<(&'static str, &'static str)>,
    fmi3_resources: Vec<(&'static str, &'static str)>,
}

lazy_static! {
    static ref PYTHONASSETS: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("python/fmi2/backend.py", "backend.py"),
            ("python/fmi2/model.py", "model.py"),
            (
                "auto_generated/fmi2_messages_pb2.py",
                "schemas/fmi2_messages_pb2.py"
            ),
            (
                "auto_generated/unifmu_handshake_pb2.py",
                "schemas/unifmu_handshake_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi2/README.md", "README.md"),
        ],
        fmi3_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("python/fmi3/backend.py", "backend.py"),
            ("python/fmi3/model.py", "model.py"),
            (
                "auto_generated/fmi3_messages_pb2.py",
                "schemas/fmi3_messages_pb2.py"
            ),
            (
                "auto_generated/unifmu_handshake_pb2.py",
                "schemas/unifmu_handshake_pb2.py"
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
            (
                "auto_generated/UnifmuHandshake.cs",
                "schemas/UnifmuHandshake.cs"
            ),
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
            (
                "auto_generated/UnifmuHandshake.java",
                "src/main/java/UnifmuHandshake.java"
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
    static ref PYTHONASSETSREMOTE: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("python/fmi2/backend_remote.py", "backend.py"),
            ("python/fmi2/model.py", "model.py"),
            (
                "auto_generated/fmi2_messages_pb2.py",
                "schemas/fmi2_messages_pb2.py"
            ),
            (
                "auto_generated/unifmu_handshake_pb2.py",
                "schemas/unifmu_handshake_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi2/README.md", "README.md"),
        ],
        fmi3_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("python/fmi3/backend_remote.py", "backend.py"),
            ("python/fmi3/model.py", "model.py"),
            (
                "auto_generated/fmi3_messages_pb2.py",
                "schemas/fmi3_messages_pb2.py"
            ),
            (
                "auto_generated/unifmu_handshake_pb2.py",
                "schemas/unifmu_handshake_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi3/README.md", "README.md"),
        ],
    };

    static ref PYTHONASSETSPROXY: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("python/fmi2/backend_proxy.py", "backend.py"),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi2/README.md", "README.md"),
        ],
        fmi3_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("python/fmi3/backend_proxy.py", "backend.py"),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi3/README.md", "README.md"),
        ],
    };

    static ref JAVAASSETSREMOTE: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            (
                "java/src/main/java/BackendPrivate.java",
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
            (
                "auto_generated/UnifmuHandshake.java",
                "src/main/java/UnifmuHandshake.java"
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
    static ref JAVAASSETSPROXY: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            (
                "java/src/main/java/BackendProxy.java",
                "src/main/java/Backend.java"
            ),
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
        ],
        fmi3_resources: vec![
            ("docker/Dockerfile_csharp", "Dockerfile"),
            ("docker/deploy_csharp.py", "deploy.py"),
            ("docker/docker-compose.yml", "docker-compose.yml"),
            ("docker/launch_csharp.toml", "launch.toml"),
            ("docker/README.md", "README_DOCKER.md"),
        ],
    };

    static ref CSHARPASSETSREMOTE: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("csharp/backend_private.cs", "backend.cs"),
            ("csharp/model.cs", "model.cs"),
            ("csharp/model.csproj", "model.csproj"),
            ("auto_generated/Fmi2Messages.cs", "schemas/Fmi2Messages.cs"),
            (
                "auto_generated/UnifmuHandshake.cs",
                "schemas/UnifmuHandshake.cs"
            ),
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

    static ref CSHARPASSETSPROXY: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("csharp/backend_proxy.cs", "backend.cs"),
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

    static ref ASSETSPROXY: Vec<(&'static str, &'static str)> = vec![
        ("proxy/launch.toml", "launch.toml"),
        ("proxy/README.md", "README.md"),
    ];

    static ref ASSETSREMOTEFMU: LanguageAssets = LanguageAssets {
        fmi2_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("common/fmi2/backend_remote_FMU.py", "backend.py"),
            (
                "auto_generated/fmi2_messages_pb2.py",
                "schemas/fmi2_messages_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi2/README.md", "README.md"),
        ],
        fmi3_resources: vec![
            ("python/requirements.txt", "requirements.txt"),
            ("common/fmi3/backend_remote_FMU.py", "backend.py"),
            (
                "auto_generated/fmi3_messages_pb2.py",
                "schemas/fmi3_messages_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/fmi3/README.md", "README.md"),
        ],
    };
}

#[derive(Debug)]
pub enum GenerateError {
    Error,
    FileExists,
    ZipError(ZipError),
}

//to be updated with keys for secure connection
#[derive(Serialize,Deserialize)]
struct Config {
   ip: String,
}

pub fn generate(
    language: &Language,
    fmu_version: &FmiFmuVersion,
    outpath: &Path,
    zipped: bool,
) -> Result<(), GenerateError> {
    let tmpdir = TempDir::new().unwrap();

    info!(
        "Generating FMU version `{:?}` for language '{:?}' with tmpdir {:?} and final output path {:?}",
        fmu_version,
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

    let unifmu_src_win = "auto_generated/unifmu.dll";
    if Assets::get(unifmu_src_win).is_none() {
        warn!("Could not find unifmu.dll in {:?}", unifmu_src_win);
    } else {
        std::fs::write(
            bin_win,
            Assets::get(unifmu_src_win).unwrap().data,
        )
        .unwrap();
    }

    let unifmu_src_linux = "auto_generated/unifmu.so";
    if Assets::get(unifmu_src_linux).is_none() {
        warn!("Could not find unifmu.so in {:?}", unifmu_src_linux);
    } else {
        std::fs::write(
            bin_linux,
            Assets::get(unifmu_src_linux).unwrap().data,
        )
        .unwrap();
    }

    let unifmu_src_macos = "auto_generated/unifmu.dylib";
    if Assets::get(unifmu_src_macos).is_none() {
        warn!("Could not find unifmu.dylib in {:?}", unifmu_src_macos);
    } else {
        std::fs::write(
            bin_macos,
            Assets::get(unifmu_src_macos).unwrap().data,
        )
        .unwrap();
    }

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

            if Assets::get(src).is_none() {
                error!("File does not exist: {:?}", src);
            }

            std::fs::write(dst_resources, Assets::get(src).unwrap().data).unwrap();
        }
    };

    match language {
        Language::Python => copy_to_resources(&PYTHONASSETS),

        Language::CSharp => copy_to_resources(&CSHARPASSETS),

        Language::Java => copy_to_resources(&JAVAASSETS),
    };

    match zipped {
        // zip to temporary, change extension from 'zip' to 'fmu', then copy to output directory
        true => {
            let mut zipped_fmu_path = PathBuf::from(outpath);
            zipped_fmu_path.set_extension("fmu");
            info!("Compressing contents into archive with path {:?}", zipped_fmu_path);

            let file = match File::create(&zipped_fmu_path) {
                Ok(f) => f,
                Err(e) => {
                    error!("Could not create file: {:?}", e);
                    return Err(GenerateError::Error)
                },
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

pub fn generate_distributed(
    language: &Language,
    fmu_version: &FmiFmuVersion,
    outpath: &Path,
    zipped: bool,
    endpoint: String,
    black_box_fmu: bool,
) -> Result<(), GenerateError>  {
    // creates two FMUs with a master and a slave for distributed co-simulation
    let config = Config {
        ip: endpoint.to_string(),
    };
    let tmpdir_proxy = TempDir::new().unwrap();
    let tmpdir_private = TempDir::new().unwrap();
    let output_string = outpath.to_str();
    let proxy_string : &str = "_proxy";
    let private_string : &str = "_private";
    let output_proxy_string = output_string.clone().unwrap();
    let output_private_string = output_string.clone().unwrap();

    let output_proxy_string = format!("{}{}",output_proxy_string,proxy_string);
    let output_private_string = format!("{}{}",output_private_string,private_string);


    let outpath_proxy = Path::new(&output_proxy_string);
    let outpath_private = Path::new(&output_private_string);
    let toml = toml::to_string(&config).unwrap();
    let endpoint_file = "endpoint.toml";
    let dst_endpoint_file = tmpdir_private.path().join(endpoint_file);

    info!(
        "Generating virtual FMUs version `{:?}` for language '{:?}' with tmpdir (proxy) {:?}/tmpdir (private) {:?}  and final output paths {:?} / {:?}",
        fmu_version,
        language,
        tmpdir_proxy.path(),
        tmpdir_private.path(),
        outpath_proxy,
        outpath_private
    );
    // First FMU (Proxy)
    // copy common files to root directory and binaries
    let (bin_macos, bin_win, bin_linux) = match fmu_version {
        FmiFmuVersion::FMI2 => {
            let bin_macos = tmpdir_proxy
                .path()
                .join("binaries")
                .join("darwin64")
                .join("unifmu.dylib");
            let bin_win = tmpdir_proxy
                .path()
                .join("binaries")
                .join("win64")
                .join("unifmu.dll");
            let bin_linux = tmpdir_proxy
                .path()
                .join("binaries")
                .join("linux64")
                .join("unifmu.so");

            (bin_macos, bin_win, bin_linux)
        }
        FmiFmuVersion::FMI3 => {
            let bin_macos = tmpdir_proxy
                .path()
                .join("binaries")
                .join("x86_64-darwin")
                .join("unifmu.dylib");
            let bin_win = tmpdir_proxy
                .path()
                .join("binaries")
                .join("x86_64-windows")
                .join("unifmu.dll");
            let bin_linux = tmpdir_proxy
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

    let md = tmpdir_proxy.path().join("modelDescription.xml");

    let unifmu_src_win = "auto_generated/unifmu.dll";
    if Assets::get(unifmu_src_win).is_none() {
        warn!("Could not find unifmu.dll in {:?}", unifmu_src_win);
    } else {
        std::fs::write(
            bin_win,
            Assets::get(unifmu_src_win).unwrap().data,
        )
        .unwrap();
    }

    let unifmu_src_linux = "auto_generated/unifmu.so";
    if Assets::get(unifmu_src_linux).is_none() {
        warn!("Could not find unifmu.so in {:?}", unifmu_src_linux);
    } else {
        std::fs::write(
            bin_linux,
            Assets::get(unifmu_src_linux).unwrap().data,
        )
        .unwrap();
    }

    let unifmu_src_macos = "auto_generated/unifmu.dylib";
    if Assets::get(unifmu_src_macos).is_none() {
        warn!("Could not find unifmu.dylib in {:?}", unifmu_src_macos);
    } else {
        std::fs::write(
            bin_macos,
            Assets::get(unifmu_src_macos).unwrap().data,
        )
        .unwrap();
    }

    // copy language specific files to 'resources' directory

    let copy_to_resources_proxy =|assets: &Vec<(&'static str, &'static str)>| {
        let assets_all = assets.to_owned();

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
            let dst_resources = tmpdir_proxy.path().join("resources").join(dst);

            info!("copying resource {:?} to {:?}", src, dst_resources);
            std::fs::create_dir_all(dst_resources.parent().unwrap()).unwrap();

            if Assets::get(src).is_none() {
                error!("File does not exist: {:?}", src);
            }

            std::fs::write(dst_resources, Assets::get(src).unwrap().data).unwrap();
        }
    };

    // copy language specific files to 'resources' directory

    let copy_to_resources = |assets: &LanguageAssets| {
        let assets_all = match fmu_version {
            FmiFmuVersion::FMI2 => assets.fmi2_resources.to_owned(),
            FmiFmuVersion::FMI3 => assets.fmi3_resources.to_owned(),
        };

        for (src, dst) in assets_all {
            let dst_resources = tmpdir_private.path().join(dst);

            info!("copying resource {:?} to {:?}", src, dst_resources);
            std::fs::create_dir_all(dst_resources.parent().unwrap()).unwrap();

            if Assets::get(src).is_none() {
                error!("File does not exist: {:?}", src);
            }

            std::fs::write(dst_resources, Assets::get(src).unwrap().data).unwrap();
        }
    };

    match language {
        Language::Python => {
            if black_box_fmu {
                copy_to_resources(&ASSETSREMOTEFMU);
            } else {
                copy_to_resources(&PYTHONASSETSREMOTE);
            }
        }

        Language::CSharp => {
            if black_box_fmu {
                copy_to_resources(&ASSETSREMOTEFMU);
            } else{
                copy_to_resources(&CSHARPASSETSREMOTE);
            }
        }

        Language::Java => {
            if black_box_fmu {
                copy_to_resources(&ASSETSREMOTEFMU);
            } else {
                copy_to_resources(&JAVAASSETSREMOTE);
            }
        }
    };

    copy_to_resources_proxy(&ASSETSPROXY);

    // Settings for the proxy connection
    fs::write(dst_endpoint_file, toml).expect("Could not write to endpoint.toml file!");

    // Creating the non-zipped private folder
    info!(
        "copying temporary dir (private) from {:?} to output {:?}",
        tmpdir_private.path(),
        outpath_private,
    );
    let mut options = CopyOptions::default();
    options.copy_inside = true;
    options.content_only = true;
    options.overwrite = true;
    fs_extra::dir::move_dir(tmpdir_private, outpath_private, &options).unwrap();

    match zipped {
        // zip to temporary, change extension from 'zip' to 'fmu', then copy to output directory
        true => {
            let mut zipped_fmu_path_proxy = PathBuf::from(outpath_proxy);
            zipped_fmu_path_proxy.set_extension("fmu");
            info!("Compressing proxy contents into archive with path {:?}", zipped_fmu_path_proxy);

            let file_proxy = match File::create(&zipped_fmu_path_proxy) {
                Ok(f) => f,
                Err(e) => {
                    error!("Could not create file: {:?}", e);
                    return Err(GenerateError::Error)
                },
            };

            let walkdir_proxy = WalkDir::new(tmpdir_proxy.path());
            let it_proxy = walkdir_proxy.into_iter();

            let method = CompressionMethod::Deflated;

            match zip_dir(
                &mut it_proxy.filter_map(|e| e.ok()),
                tmpdir_proxy.path().to_str().unwrap(),
                file_proxy,
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
                "copying temporary dir (proxy) from {:?} to output {:?}",
                tmpdir_proxy.path(),
                outpath_proxy,
            );
            fs_extra::dir::move_dir(tmpdir_proxy, outpath_proxy, &options).unwrap();
            Ok(())
        }
    }



}
