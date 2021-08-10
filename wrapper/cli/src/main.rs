use std::{
    collections::HashMap,
    option,
    path::{Path, PathBuf},
};

use clap::arg_enum;
use env_logger::Builder;
use fs_extra::dir::CopyOptions;
use lazy_static::lazy_static;
use log::info;
use rust_embed::RustEmbed;
use structopt::StructOpt;
use tempfile::TempDir;

arg_enum! {
    #[derive(Debug)]
enum Language {
    Python,
    CSharp,
    Matlab,
    Java,
}
}

#[derive(Debug, StructOpt)]
#[structopt(name = "UniFMU", about = "An example of StructOpt usage.")]
enum Subcommands {
    /// Create a new FMU using the specified source language
    Generate {
        /// Source language of the generated FMU
        #[structopt(possible_values = &Language::variants(), case_insensitive = true)]
        language: Language,

        outpath: PathBuf,

        /// Compress the generated FMU as a zip-archive and store with '.fmu' extension
        #[structopt(short, long)]
        zipped: bool,

        /// Configure the generated model to deploy and execute code inside a 'Docker' container
        #[structopt(short, long)]
        dockerize: bool,
    },

    Validate {},
}
#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(RustEmbed)]
#[folder = "assets/python"]
struct PythonAssets;

// lazy_static! {
//     static ref HASHMAP: HashMap<Language, &'static str> = {
//         let mut m = HashMap::new();
//         m.insert(Language::Python, ("backend.py", "backend.py"));
//         m.insert(Language::Python, "backend.py");
//         m.insert(2, "baz");
//         m
//     };
// }

fn main() {
    let assets: Vec<String> = Assets::iter().map(|f| String::from(f)).collect();
    println!("I got all these assets: {:?}", assets);

    let opt = Subcommands::from_args();
    println!("{:?}", opt);

    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info);
    b.init();

    match opt {
        Subcommands::Generate {
            language,
            outpath,
            zipped,
            dockerize,
        } => {
            let tmpdir = TempDir::new().unwrap();

            info!(
                "Generating FMU for language '{:?}' with tmpdir {:?} and final output path {:?}",
                language,
                tmpdir.path(),
                outpath
            );

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

            let resources = tmpdir.path().join("resources");

            match language {
                Language::Python => {
                    for f in PythonAssets::iter() {
                        let out = resources.join(&*f);
                        info!("writing resource: {:?} to {:?}", f, out);
                        std::fs::create_dir_all(&out.parent().unwrap()).unwrap();
                        std::fs::write(resources.join(&*f), PythonAssets::get(&*f).unwrap().data)
                            .unwrap();
                    }
                }
                Language::CSharp => todo!(),
                Language::Matlab => todo!(),
                Language::Java => todo!(),
            };

            let mut options = CopyOptions::default();
            options.copy_inside = true;
            options.content_only = true;
            options.overwrite = true;
            fs_extra::dir::move_dir(tmpdir, outpath, &options).unwrap();
        }
        Subcommands::Validate {} => todo!(),
    }
}
