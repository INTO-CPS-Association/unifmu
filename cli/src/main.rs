use clap;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::{error, info};
use std::{path::PathBuf, process::exit};
use unifmu::FmiFmuVersion;
use unifmu::{
    benchmark::{benchmark, BenchmarkConfig},
    generate,
    validation::{validate, ValidationConfig},
    Language,
};

static ABOUT: &'static str = "
Implement Functional Mock-up units (FMUs) in various source languages.

* Source:   https://github.com/INTO-CPS-Association/unifmu
* Examples: https://github.com/INTO-CPS-Association/unifmu_examples";

#[derive(Debug, Parser)]
#[clap(author, version, about = ABOUT)]
struct Arguments {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create a new FMU using the specified source language
    Generate {
        /// Source language of the generated FMU
        #[clap(arg_enum)]
        language: Language,

        /// Version of the FMI specification to target
        #[clap(arg_enum)]
        fmu_version: FmiFmuVersion,

        /// Output directory or name of the FMU archive if "--zipped" is passed
        outpath: PathBuf,

        /// Compress the generated FMU as a zip-archive and store with '.fmu' extension
        #[clap(short, long)]
        zipped: bool,
    },

    /// Run a suite of checks to detect potential defects of the FMU
    Validate {
        /// Path to FMU directory or archive
        path: PathBuf,
    },

    /// Benchmark the performance of the FMU
    Benchmark {
        /// Path to FMU directory or archive
        path: PathBuf,
    },
}

fn main() {
    let opt = Arguments::parse();

    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();

    match opt.cmd {
        Command::Generate {
            language,
            fmu_version,
            outpath,
            zipped,
        } => match generate(&language, &fmu_version, &outpath, zipped) {
            Ok(_) => {
                info!("the FMU was generated successfully");
            }
            Err(e) => {
                error!("an error ocurred during the generation of the FMU: {:?}", e);
                exit(-1);
            }
        },
        Command::Validate { path } => {
            let config = ValidationConfig::default();

            let path = match path.is_absolute() {
                true => path,
                false => std::env::current_dir().unwrap().join(path),
            };

            if !path.exists() {
                error!("Unable to open FMU, the specified path is neither a directory or a file.");
                exit(-1);
            }

            let md_path = path.join("modelDescription.xml");

            info!("Attempting to locate 'modelDescription.xml' at path {}", "");

            if !md_path.exists() {
                error!("Unable to locate 'modelDescription.xml' inside the FMU");
                exit(-1);
            }

            info!(
                "validating the following FMU {:?} with the following checks {:?}",
                path, config
            );

            match validate(&path, &config) {
                Ok(_) => info!("no errors detected during validation of the FMU"),
                Err(e) => {
                    error!(
                        "a defect was detected during the validation of the FMU: {:?} ",
                        e
                    );
                    exit(-1);
                }
            }
        }
        Command::Benchmark { path } => benchmark(&path, &BenchmarkConfig::default()),
    }
}
