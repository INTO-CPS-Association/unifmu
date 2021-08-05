use std::fs::File;

use common::Fmi2Status::Fmi2OK;
use rpc::{Fmi2Command, Fmi2Return};
use serde::Serialize;

#[derive(Serialize)]
struct Sample {
    commands: Vec<Fmi2Command>,
    results: Vec<Fmi2Return>,
}

fn main() {
    let s = Sample {
        commands: vec![
            Fmi2Command::Fmi2SetupExperiment {
                start_time: 0.0,
                stop_time: Some(0.0),
                tolerance: Some(1.0),
            },
            Fmi2Command::Fmi2EnterInitializationMode,
            Fmi2Command::Fmi2ExitInitializationMode,
        ],
        results: vec![
            Fmi2Return::Fmi2StatusReturn { status: Fmi2OK },
            Fmi2Return::Fmi2GetRealReturn {
                status: Fmi2OK,
                values: vec![0.0, 0.0, 0.0],
            },
            Fmi2Return::Fmi2GetIntegerReturn {
                status: Fmi2OK,
                values: vec![0, 0, 0],
            },
            Fmi2Return::Fmi2GetBooleanReturn {
                status: Fmi2OK,
                values: vec![false, false, false],
            },
            Fmi2Return::Fmi2GetStringReturn {
                status: Fmi2OK,
                values: vec![String::from("a"), String::from("a"), String::from("a")],
            },
        ],
    };

    // let pickle = serde_pickle::to_vec(&commands, true);

    let file = File::create("sample.json").unwrap();
    serde_json::to_writer_pretty(&file, &s).unwrap();

    let mut file = File::create("sample.pkl").unwrap();
    serde_pickle::to_writer(&mut file, &s, true).unwrap();

    {
        let mut file = File::create("test.pkl").unwrap();
        serde_pickle::to_writer(
            &mut file,
            &Fmi2Return::Fmi2StatusReturn { status: Fmi2OK },
            true,
        )
        .unwrap();
    }

    let mut file = File::open("test.pkl").unwrap();
    let res: Fmi2Return = serde_pickle::from_reader(&file).unwrap();
    println!("{:?}", res);
}

// ('Fmi2StatusReturn', {'status': 0})
