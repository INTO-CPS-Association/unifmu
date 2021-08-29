use env_logger::Builder;
use log::{error, info};
use std::{path::PathBuf, process::exit};
use structopt::StructOpt;
use unifmu::{
    benchmark::{benchmark, BenchmarkConfig},
    generate,
    validation::{validate, ValidationConfig},
    Language,
};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "UniFMU",
    about = "Implement 'Functional Mock-up units' (FMUs) in various source languages."
)]
struct Arguments {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]

enum Command {
    /// Create a new FMU using the specified source language
    Generate {
        /// Source language of the generated FMU
        #[structopt(possible_values = &Language::variants(), case_insensitive = true)]
        language: Language,

        /// Output directory or name of the FMU archive if "--zipped" is passed
        outpath: PathBuf,

        /// Compress the generated FMU as a zip-archive and store with '.fmu' extension
        #[structopt(short, long)]
        zipped: bool,

        /// Configure the generated model to deploy and execute code inside a 'Docker' container
        #[structopt(short, long)]
        dockerize: bool,
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
    let opt = Arguments::from_args();

    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();

    match opt.cmd {
        Command::Generate {
            language,
            outpath,
            zipped,
            dockerize,
        } => match generate(&language, &outpath, zipped, dockerize) {
            Ok(_) => {
                info!("the FMU was generated succesfully");
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

            // let path = path.canonicalize().unwrap();

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
