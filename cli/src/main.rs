use env_logger::Builder;
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use unifmu::{generate, Language};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "UniFMU",
    about = "Generate FMUs in one of several source languages."
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

// struct LanguageAssets {
//     resources: Vec<(&'static str, &'static str)>,
//     docker: Vec<(&'static str, &'static str)>,
// }

// lazy_static! {
//     static ref PYTHONASSETS: LanguageAssets = LanguageAssets {
//         resources: vec![
//             ("python/backend.py", "backend.py"),
//             ("python/model.py", "model.py"),
//             (
//                 "python/schemas/unifmu_fmi2_pb2.py",
//                 "schemas/unifmu_fmi2_pb2.py"
//             ),
//             ("python/launch.toml", "launch.toml"),
//             ("python/README.md", "README.md"),
//         ],
//         docker: vec![
//             ("docker/Dockerfile_python", "Dockerfile"),
//             ("docker/compose_python.yml", "compose.yml"),
//             ("docker/launch.toml", "launch.toml"),
//             ("docker/README.md", "README_DOCKER.md"),
//         ],
//     };
//     static ref CSHARPASSETS: LanguageAssets = LanguageAssets {
//         resources: vec![
//             ("csharp/backend.cs", "backend.cs"),
//             ("csharp/model.cs", "model.cs"),
//             ("csharp/model.csproj", "model.csproj"),
//             ("csharp/schemas/UnifmuFmi2.cs", "schemas/UnifmuFmi2.cs"),
//             ("csharp/launch.toml", "launch.toml"),
//             ("csharp/README.md", "README.md"),
//         ],
//         docker: vec![
//             ("docker/Dockerfile_csharp", "Dockerfile"),
//             ("docker/compose_csharp.yml", "compose.yml"),
//             ("docker/launch.toml", "launch.toml"),
//             ("docker/README.md", "README_DOCKER.md"),
//         ],
//     };
// }

fn main() {
    let opt = Subcommands::from_args();
    // println!("{:?}", opt);

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
