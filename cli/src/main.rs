use clap;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::{error, info};
use std::{path::PathBuf, process::exit};
use unifmu::FmiFmuVersion;
use unifmu::{
    generate,
    generate_virtual,
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
        #[clap(value_enum)]
        language: Language,

        /// Output directory or name of the FMU archive if "--zipped" is passed
        outpath: PathBuf,

        /// Version of the FMI specification to target
        #[clap(value_enum, default_value_t=FmiFmuVersion::FMI2)]
        fmu_version: FmiFmuVersion,

        /// Compress the generated FMU as a zip-archive and store with '.fmu' extension
        #[clap(short, long)]
        zipped: bool,
    },

    /// Generates a pair of FMUs for distributed co-simulation, where one FMU works as a master and the other one as a slave.
    Generate_virtual {
        /// Source language of the generated FMU
        #[clap(value_enum)]
        language: Language,

        /// Output directory or name of the FMU archive if "--zipped" is passed
        outpath: PathBuf,

        /// This argument shall be passed with the host IP address for the virtual FMU
        endpoint: String,

        /// Version of the FMI specification to target
        #[clap(value_enum, default_value_t=FmiFmuVersion::FMI2)]
        fmu_version: FmiFmuVersion,

        /// Compress the generated FMU as a zip-archive and store with '.fmu' extension
        #[clap(short, long)]
        zipped: bool,
    }
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
        }

        Command::Generate_virtual {
            language,
            fmu_version,
            outpath,
            zipped,
            endpoint,
        } => match generate_virtual(&language, &fmu_version, &outpath, zipped, endpoint) {
            Ok(_) => {
                info!("the FMUs were generated successfully");
            }
            Err(e) => {
                error!("an error ocurred during the generation of the FMUs: {:?}", e);
                exit(-1);
            }
        }
    }
}
