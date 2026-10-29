//! Contains the BackendSubprocess, a struct wrapping a handle to the
//! process containing the backend.

use std::{
    collections::VecDeque,
    error::Error, ffi::OsString, fmt::{Debug, Display}, io, path::Path,
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread::{self, JoinHandle},
    time::Instant,
};

use subprocess::{Communicator, ExitStatus, Popen, PopenConfig, PopenError};
use tokio::time::{Duration, sleep};

const OUTPUT_LIMIT: usize = 64 * 1024;
const OUTPUT_POLL_TIME: Duration = Duration::from_millis(100);

/// Representes the subprocess containing the backend.
/// 
/// Stores the subprocess handle for concurrency reasons.
pub struct BackendSubprocess {
    polling_time: Duration,
    subprocess: Popen,
    output_reader: Option<JoinHandle<io::Result<String>>>,
    stop_output_reader: Arc<AtomicBool>,
    output_dump: String,
}

impl BackendSubprocess {
    pub fn create(
        endpoint: String,
        launch_command: &Vec<String>,
        resource_path: &Path
    ) -> SubprocessResult<Self> {
        Self::create_with_output(endpoint, launch_command, resource_path, io::stdout())
    }

    fn create_with_output(
        endpoint: String,
        launch_command: &Vec<String>,
        resource_path: &Path,
        relay: impl io::Write + Send + 'static,
    ) -> SubprocessResult<Self> {
        let endpoint_port = match endpoint
            .split(":")
            .last() {
                Some(port) => port,
                None => {
                    return Err(SubprocessError::NoPortGiven)
                }
            }
            .to_owned();
        
        let mut env_vars: Vec<(OsString, OsString)> = std::env::vars_os().collect();

        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT"),
            OsString::from(endpoint),
        ));
        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT_PORT"),
            OsString::from(endpoint_port),
        ));

        let mut subprocess = match Popen::create(
            launch_command,
            PopenConfig {
                cwd: Some(resource_path.as_os_str().to_owned()),
                env: Some(env_vars),
                stderr: subprocess::Redirection::Merge,
                stdout: subprocess::Redirection::Pipe,
                ..Default::default()
            },
        ) {
            Ok(subprocess) => subprocess,
            Err(error) => {
                return Err(SubprocessError::UnExecutableCommand(
                    format!("{:?}", launch_command),
                    error
                ))
            }
        };

        let communicator = subprocess.communicate_start(None)
            .limit_size(4096)
            .limit_time(OUTPUT_POLL_TIME);
        let stop_output_reader = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&stop_output_reader);
        let output_reader = match thread::Builder::new()
            .name("unifmu-backend-output".into())
            .spawn(move || capture_output(communicator, &stop, relay))
        {
            Ok(reader) => reader,
            Err(error) => {
                if let Err(kill_error) = subprocess.kill() {
                    tracing::error!("Failed to kill backend after output reader setup failed: {kill_error}");
                    subprocess.detach();
                }
                return Err(SubprocessError::OutputRead(error));
            }
        };

        Ok(
            Self{
                // TODO: This is a magic number. Is there a smarter way to define polling_time?
                polling_time: Duration::from_millis(100),
                subprocess,
                output_reader: Some(output_reader),
                stop_output_reader,
                output_dump: String::new(),
            }
        )
    }

    /// Continously polls the backend subprocess and returns if the subprocess
    /// returns an exit status.
    /// 
    /// Will only ever return an Err.
    pub async fn monitor_subprocess(&mut self) -> SubprocessResult<()> {
        loop {
            if self.output_reader.as_ref().is_some_and(|reader| reader.is_finished()) {
                self.finish_output_reader()?;
            }
            match self.subprocess.poll() {
                Some(exit_status) => {
                    self.finish_output_reader()?;
                    return Err(SubprocessError::UnexpectedExit(
                        exit_status, self.output_dump.clone()
                    ))
                },
                None => {
                    sleep(self.polling_time).await; // Is magic number.
                }
            }
        }
    }

    fn finish_output_reader(&mut self) -> SubprocessResult<()> {
        self.stop_output_reader.store(true, Ordering::Relaxed);
        if let Some(reader) = self.output_reader.take() {
            self.output_dump = reader.join()
                .map_err(|_| SubprocessError::OutputReaderPanicked)?
                .map_err(SubprocessError::OutputRead)?;
        }
        Ok(())
    }
}

impl Drop for BackendSubprocess {
    fn drop(&mut self) {
        // Do not wait indefinitely for a backend that is still running.
        if self.subprocess.poll().is_none() {
            if let Err(error) = self.subprocess.kill() {
                tracing::error!("Failed to kill backend subprocess: {error}");
                self.subprocess.detach();
                if let Err(error) = self.finish_output_reader() {
                    tracing::error!("Failed to finish backend output capture: {error}");
                }
                return;
            }
        }
        if let Err(error) = self.subprocess.wait() {
            tracing::error!("Failed to wait for backend subprocess: {error}");
        }
        if let Err(error) = self.finish_output_reader() {
            tracing::error!("Failed to finish backend output capture: {error}");
        }
    }
}

fn capture_output(
    mut communicator: Communicator,
    stop: &AtomicBool,
    mut relay: impl io::Write,
) -> io::Result<String> {
    let mut output = VecDeque::with_capacity(OUTPUT_LIMIT);
    let mut truncated = false;
    let mut stopping_since = None;
    loop {
        // A descendant may inherit the pipe after the backend exits. Allow a
        // bounded final drain rather than waiting indefinitely for its EOF.
        if stop.load(Ordering::Relaxed) {
            let since = stopping_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= OUTPUT_POLL_TIME {
                break;
            }
        }
        let (capture, eof) = match communicator.read() {
            Ok(capture) => {
                let eof = capture.0.as_ref().is_none_or(|bytes| bytes.is_empty());
                (capture, eof)
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut => (error.capture, false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => (error.capture, false),
            Err(error) => return Err(error.error),
        };
        if let Some(bytes) = capture.0 {
            relay.write_all(&bytes)?;
            relay.flush()?;
            let overflow = (output.len() + bytes.len()).saturating_sub(OUTPUT_LIMIT);
            if overflow > 0 {
                output.drain(..overflow);
                truncated = true;
            }
            output.extend(bytes);
        }
        if eof {
            break;
        }
    }
    let mut message = if truncated {
        String::from("[backend output truncated; showing last 65536 bytes]\n")
    } else {
        String::new()
    };
    message.push_str(&String::from_utf8_lossy(output.make_contiguous()));
    Ok(message)
}

type SubprocessResult<T> = Result<T, SubprocessError>;

#[derive(Debug)]
pub enum SubprocessError {
    UnexpectedExit(ExitStatus, String),
    NoPortGiven,
    UnExecutableCommand(String, PopenError),
    OutputRead(io::Error),
    OutputReaderPanicked,
}

impl Display for SubprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedExit(exit_status, output_dump) => {
                let clarification = match exit_status {
                    ExitStatus::Exited(code) => format!(
                        "with exit status {}", code
                    ),
                    ExitStatus::Signaled(signal) => format!(
                        "because of signal {}", signal
                    ),
                    ExitStatus::Other(status) => format!(
                        "with unknown status {}", status
                    ),
                    ExitStatus::Undetermined => String::from(
                        "for an undeterminable reason"
                    )
                };
                let backend_output_message = if output_dump.is_empty() {
                    String::from(". Backend emitted no output before exit")
                } else {
                    format!(". Backend emitted the following output before exit: \n{}", output_dump)
                };
                write!(f, "backend exited unexpectedly {}{}", clarification, backend_output_message)
            }
            Self::NoPortGiven => {
                write!(f, "the endpoint given to for the backend to connect to didn't have a portnumber")
            }
            Self::UnExecutableCommand(command, popen_error) => {
                write!(f, "unable to start the backend subprocess using the specified command '{}'; {}", command, popen_error)
            }
            Self::OutputRead(error) => write!(f, "unable to capture or relay backend output: {error}"),
            Self::OutputReaderPanicked => write!(f, "backend output reader thread panicked"),
        }
    }
}

impl Error for SubprocessError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::Mutex,
    };

    #[derive(Clone, Default)]
    struct RelayedOutput(Arc<Mutex<Vec<u8>>>);

    impl Write for RelayedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn backend_output_fixture() {
        let Ok(endpoint) = std::env::var("UNIFMU_DISPATCHER_ENDPOINT") else {
            return;
        };
        let Some(endpoint) = endpoint.strip_prefix("output-test://") else {
            return;
        };
        let (mode, address) = endpoint.split_once('/').unwrap();
        let mut output: Box<dyn Write> = if mode == "stderr" {
            Box::new(std::io::stderr())
        } else {
            Box::new(std::io::stdout())
        };
        if mode != "small" {
            for _ in 0..512 {
                output.write_all(&[b'x'; 4096]).unwrap();
            }
        }
        output.write_all(b"\xff\nbackend diagnostic tail\n").unwrap();
        output.flush().unwrap();
        let mut signal = TcpStream::connect(address).unwrap();
        // Keep the backend alive until the parent confirms that output was
        // drained even while no FMI call or process monitor was running.
        signal.write_all(b"done").unwrap();
        let _ = signal.read(&mut [0]);
        std::process::exit(7);
    }

    fn start_backend(
        mode: &str,
        relay: impl Write + Send + 'static,
    ) -> (BackendSubprocess, TcpListener) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let command = vec![
            std::env::current_exe().unwrap().into_os_string().into_string().unwrap(),
            "--exact".into(),
            format!("{}::backend_output_fixture", module_path!().split_once("::").unwrap().1),
            "--nocapture".into(),
        ];
        let backend = BackendSubprocess::create_with_output(
            format!("output-test://{mode}/{}", listener.local_addr().unwrap()),
            &command,
            &std::env::current_dir().unwrap(),
            relay,
        ).unwrap();
        (backend, listener)
    }

    fn await_signal(listener: &TcpListener) -> TcpStream {
        let started = Instant::now();
        let mut signal = loop {
            match listener.accept() {
                Ok((signal, _)) => break signal,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    assert!(started.elapsed() < Duration::from_secs(10),
                        "backend blocked while writing output");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("unable to accept backend signal: {error}"),
            }
        };
        signal.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let mut done = [0; 4];
        signal.read_exact(&mut done).unwrap();
        assert_eq!(&done, b"done");
        signal
    }

    fn check_backend_output(mode: &str) {
        let relay = RelayedOutput::default();
        let (mut backend, listener) = start_backend(mode, relay.clone());
        let signal = await_signal(&listener);
        let started = Instant::now();
        loop {
            if relay.0.lock().unwrap().ends_with(b"\xff\nbackend diagnostic tail\n") {
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(2),
                "backend output was not relayed while the backend was alive");
            thread::sleep(Duration::from_millis(10));
        }
        if mode != "small" {
            assert!(relay.0.lock().unwrap().len() > 2 * 1024 * 1024);
        }
        drop(signal);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all().build().unwrap();
        let error = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), backend.monitor_subprocess()).await
        }).expect("backend monitor hung").unwrap_err();
        let SubprocessError::UnexpectedExit(ExitStatus::Exited(7), output) = error else {
            panic!("unexpected backend error: {error}");
        };
        assert!(output.contains("backend diagnostic tail"));
        assert!(output.contains('\u{fffd}'), "non-UTF8 output should not discard diagnostics");
        if mode == "small" {
            assert!(!output.contains("output truncated"));
        } else {
            assert!(output.starts_with("[backend output truncated;"));
            assert!(output.len() <= OUTPUT_LIMIT + 128);
        }
    }

    #[test]
    fn drains_large_stdout_while_backend_is_running() {
        check_backend_output("stdout");
    }

    #[test]
    fn drains_large_stderr_while_backend_is_running() {
        check_backend_output("stderr");
    }

    #[test]
    fn preserves_small_output_on_unexpected_exit() {
        check_backend_output("small");
    }

    #[test]
    fn reports_output_relay_errors() {
        struct BrokenWriter;
        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "relay closed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let (mut backend, _listener) = start_backend("small", BrokenWriter);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all().build().unwrap();
        let error = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), backend.monitor_subprocess()).await
        }).expect("backend monitor hung").unwrap_err();
        assert!(matches!(error, SubprocessError::OutputRead(error)
            if error.kind() == io::ErrorKind::BrokenPipe));
    }

    #[test]
    fn dropping_backend_stops_child_and_output_reader() {
        let (backend, listener) = start_backend("small", io::sink());
        let mut signal = await_signal(&listener);
        let started = Instant::now();
        drop(backend);
        assert!(started.elapsed() < Duration::from_secs(2));
        signal.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut remaining = Vec::new();
        if let Err(error) = signal.read_to_end(&mut remaining) {
            assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
        }
        assert!(remaining.is_empty());
    }
}