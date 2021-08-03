use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

fn setup(
    context: &zmq::Context,
    format: &str,
) -> (zmq::Socket, zmq::Socket, zmq::Message, zmq::Message) {
    // let context = zmq::Context::new();
    let client = context.socket(zmq::SocketType::REQ).unwrap();
    let server = context.socket(zmq::SocketType::REP).unwrap();

    let endpoint = format!("{}://*:0", format);
    loop {
        match server.bind(&endpoint) {
            Ok(_) => break,
            Err(_) => (),
        };
    }

    let endpoint = server.get_last_endpoint().unwrap().unwrap();
    println!("connected client to server on {:?}", endpoint);

    client.connect(&endpoint).unwrap();
    let client_msg = zmq::Message::new();
    let server_msg = zmq::Message::new();
    (client, server, client_msg, server_msg)
}

fn sequential_read_write(
    client: &zmq::Socket,
    server: &zmq::Socket,
    client_msg: &mut zmq::Message,
    server_msg: &mut zmq::Message,
) {
    let data = vec![40; 0];

    // for i in 0..100 {
    client.send(&data, 0).unwrap();
    server.recv(server_msg, 0).unwrap();
    server.send(&data, 0).unwrap();
    client.recv(client_msg, 0).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let context = zmq::Context::new();

    let mut g = c.benchmark_group("zmq");
    // group.sample_size(n)

    g.bench_function("no-op", |b| {
        b.iter_batched(
            || setup(&context, "inproc"),
            |_| black_box(0),
            BatchSize::LargeInput,
        )
    });

    g.bench_function("zmq inproc", |b| {
        b.iter_batched(
            || setup(&context, "inproc"),
            |(client, server, mut client_msg, mut server_msg)| {
                sequential_read_write(&client, &server, &mut client_msg, &mut server_msg)
            },
            BatchSize::LargeInput,
        )
    });

    // c.bench_function("zmq tcp", |b| {
    //     b.iter_batched(
    //         || setup(&context, "tcp"),
    //         |(client, server, mut client_msg, mut server_msg)| {
    //             sequential_read_write(&client, &server, &mut client_msg, &mut server_msg)
    //         },
    //         BatchSize::PerIteration,
    //     )
    // });

    println!("dropping sockets, this may take a while");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
