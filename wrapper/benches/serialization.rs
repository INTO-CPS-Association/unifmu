use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use static_dispatch::Fmi2Command;

fn json_alloc() {
    serde_json::to_vec(&Fmi2Command::Fmi2DoStep {
        current_time: 0.0,
        step_size: 1.0,
        no_step_prior: false,
    })
    .unwrap();
}

fn pickle_alloc() {
    serde_pickle::to_vec(
        &Fmi2Command::Fmi2DoStep {
            current_time: 0.0,
            step_size: 1.0,
            no_step_prior: false,
        },
        true,
    )
    .unwrap();
}

fn json_no_alloc(buf: &mut Vec<u8>) {
    buf.clear();
    serde_json::to_writer(
        buf,
        &Fmi2Command::Fmi2DoStep {
            current_time: 0.0,
            step_size: 1.0,
            no_step_prior: false,
        },
    )
    .unwrap();
}

fn pickle_no_alloc(buf: &mut Vec<u8>) {
    buf.clear();
    serde_pickle::to_writer(
        buf,
        &&Fmi2Command::Fmi2DoStep {
            current_time: 0.0,
            step_size: 1.0,
            no_step_prior: false,
        },
        true,
    )
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pickle serialize (allocation)", |b| {
        b.iter(|| pickle_alloc())
    });

    c.bench_function("pickle serialize (zero allocation)", |b| {
        b.iter_batched(
            || {
                let v: Vec<u8> = Vec::with_capacity(128);
                v
            },
            |mut b| pickle_no_alloc(&mut b),
            BatchSize::SmallInput,
        )
    });

    c.bench_function("json serialize (allocation)", |b| b.iter(|| json_alloc()));

    c.bench_function("json serialize (zero allocation)", |b| {
        b.iter_batched(
            || {
                let v: Vec<u8> = Vec::with_capacity(128);
                v
            },
            |mut b| json_no_alloc(&mut b),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
