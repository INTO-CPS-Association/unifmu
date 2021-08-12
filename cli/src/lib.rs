use clap::arg_enum;
use std::path::Path;

use fs_extra::dir::CopyOptions;
use lazy_static::lazy_static;
use log::info;
use rust_embed::RustEmbed;
use tempfile::TempDir;

arg_enum! {
    #[derive(Debug)]
pub enum Language {
    Python,
    CSharp,
    Matlab,
    Java,
}
}
#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

struct LanguageAssets {
    resources: Vec<(&'static str, &'static str)>,
    docker: Vec<(&'static str, &'static str)>,
}

lazy_static! {
    static ref PYTHONASSETS: LanguageAssets = LanguageAssets {
        resources: vec![
            ("python/backend.py", "backend.py"),
            ("python/model.py", "model.py"),
            (
                "python/schemas/unifmu_fmi2_pb2.py",
                "schemas/unifmu_fmi2_pb2.py"
            ),
            ("python/launch.toml", "launch.toml"),
            ("python/README.md", "README.md"),
        ],
        docker: vec![
            ("docker/Dockerfile_python", "Dockerfile"),
            ("docker/deploy_python.py", "deploy.py"),
            ("docker/docker-compose.yml", "docker-compose.yml"),
            ("docker/launch_python.toml", "launch.toml"),
            ("docker/README.md", "README_DOCKER.md"),
        ],
    };
    static ref CSHARPASSETS: LanguageAssets = LanguageAssets {
        resources: vec![
            ("csharp/backend.cs", "backend.cs"),
            ("csharp/model.cs", "model.cs"),
            ("csharp/model.csproj", "model.csproj"),
            ("csharp/schemas/UnifmuFmi2.cs", "schemas/UnifmuFmi2.cs"),
            ("csharp/launch.toml", "launch.toml"),
            ("csharp/README.md", "README.md"),
        ],
        docker: vec![
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
}

pub fn generate(
    language: &Language,
    outpath: &Path,
    zipped: bool,
    dockerize: bool,
) -> Result<(), GenerateError> {
    let tmpdir = TempDir::new().unwrap();

    info!(
        "Generating FMU for language '{:?}' with tmpdir {:?} and final output path {:?}",
        language,
        tmpdir.path(),
        outpath
    );

    // copy common files to root directory and binaries
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

    for p in vec![&bin_macos, &bin_win, &bin_linux] {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    }

    let md = tmpdir.path().join("modelDescription.xml");

    std::fs::write(
        &md,
        Assets::get("common/modelDescription.xml").unwrap().data,
    )
    .unwrap();

    info!("{:?}", &bin_win);
    std::fs::write(bin_win, Assets::get("common/unifmu.dll").unwrap().data).unwrap();
    std::fs::write(bin_linux, Assets::get("common/unifmu.so").unwrap().data).unwrap();
    std::fs::write(bin_macos, Assets::get("common/unifmu.dylib").unwrap().data).unwrap();

    // copy language specific files to 'resources' directory

    let copy_to_resources = |assets: &LanguageAssets| {
        let mut assets_all = assets.resources.to_owned();

        if dockerize {
            assets_all.extend(assets.docker.to_owned())
        };

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

        Language::Java => todo!(),
    };

    match zipped {
        // zip to temporary, change extension from 'zip' to 'fmu', then copy to output directory
        true => todo!(),

        // copy from temporary directory to output directory
        false => {
            let mut options = CopyOptions::default();
            options.copy_inside = true;
            options.content_only = true;
            options.overwrite = true;
            fs_extra::dir::move_dir(tmpdir, outpath, &options).unwrap();
            Ok(())
        }
    }
}
