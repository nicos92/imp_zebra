//! Mock Zebra printer for local development and end-to-end testing.
//!
//! Listens on a TCP port (default `127.0.0.1:9100`, the ZPL default),
//! receives raw ZPL bytes and appends them to an output file so the
//! payload of each print job can be inspected.
//!
//! Usage:
//!   cargo run --example mock_zebra [-- <host> <port> <output-file>]
//!
//! Defaults: host `127.0.0.1`, port `9100`, output `mock_zebra.zpl`.
//! Pass `0` as port to let the OS pick a free one (recommended).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 9100;
const DEFAULT_OUTPUT: &str = "mock_zebra.zpl";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let host = args.first().map(|s| s.as_str()).unwrap_or(DEFAULT_HOST);
    let port = args
        .get(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);
    let output = PathBuf::from(args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_OUTPUT));

    let listener = match TcpListener::bind((host, port)) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!(
                "mock_zebra: could not bind {host}:{port}: {e}\n\
                 hint: pass 0 as port to let the OS pick a free one"
            );
            return ExitCode::FAILURE;
        }
    };
    let bound = listener.local_addr().expect("bound address");

    println!("mock_zebra: listening on {bound}");
    println!(
        "mock_zebra: appending received ZPL to '{}'",
        output.display()
    );
    println!("mock_zebra: press Ctrl+C to stop");

    let mut job_count: u64 = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match handle_client(stream, &output, &mut job_count) {
                Ok(bytes) => {
                    job_count += 1;
                    println!(
                        "mock_zebra: job #{job_count} captured {bytes} bytes -> {}",
                        output.display()
                    );
                }
                Err(e) => eprintln!("mock_zebra: client error: {e}"),
            },
            Err(e) => eprintln!("mock_zebra: accept error: {e}"),
        }
    }

    ExitCode::SUCCESS
}

fn handle_client(
    stream: TcpStream,
    output: &PathBuf,
    job_count: &mut u64,
) -> std::io::Result<usize> {
    let mut buf = Vec::new();
    {
        let mut stream = stream;
        stream.read_to_end(&mut buf)?;
    }

    let label = format!(
        "\n=== Zebra mock print #{job_number} — {timestamp} ===",
        job_number = *job_count + 1,
        timestamp = timestamp(),
    );
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output)?;
    file.write_all(label.as_bytes())?;
    file.write_all(&buf)?;
    file.write_all(b"\n")?;
    Ok(buf.len())
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix {secs}")
}
