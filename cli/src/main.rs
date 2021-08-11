use env_logger::Builder;
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use unifmu::{generate, Language};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "UniFMU",
    about = "Implement 'Functional Mock-up units' (FMUs) in various source languages."
)]
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

fn main() {
    let opt = Subcommands::from_args();

    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info);
    b.init();

    match opt {
        Subcommands::Generate {
            language,
            outpath,
            zipped,
            dockerize,
        } => match generate(&language, &outpath, zipped, dockerize) {
            Ok(_) => {
                info!("The FMU was generated succesfully");
            }
            Err(_) => todo!(),
        },
        Subcommands::Validate {} => todo!(),
    }
}
