use phroxy::{server, Result};
use std::net::TcpListener;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 0;
    let mut host = "0.0.0.0";

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--version" | "-version" => {
                print_version();
                return Ok(());
            }
            "--help" | "-help" => {
                print_help();
                return Ok(());
            }
            "-p" | "--port" | "-port" => {
                if let Some(p) = iter.next() {
                    port = p.parse().unwrap_or(0);
                }
            }
            "-h" => {
                if let Some(h) = iter.next() {
                    host = h;
                } else {
                    print_help();
                    return Ok(());
                }
            }
            "--host" | "-host" => {
                if let Some(h) = iter.next() {
                    host = h;
                }
            }
            arg => {
                if !arg.is_empty() {
                    if let Some('-') = arg.chars().nth(0) {
                        eprintln!("Unknown option: {}", arg);
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    server::start(TcpListener::bind(format!("{}:{}", host, port))?)
}

fn print_help() {
    println!(
        "Usage:

    phroxy [options]

Options:

    -p, --port NUM    Port to bind to.
    -h, --host NAME   Hostname to bind to.
  
Other flags:  
  
    -h, --help        Print this screen.
    -v, --version     Print phroxy version."
    );
}

fn print_version() {
    println!("phroxy v{}", env!("CARGO_PKG_VERSION"));
}
