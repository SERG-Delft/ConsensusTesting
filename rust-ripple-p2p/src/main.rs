use std::process::Command;
use powershell_script;
use std::fs;
use std::env;

mod app;
mod protos;
mod message_handler;
mod client;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .core_threads(num_cpus::get())
        .enable_io()
        .enable_time()
        .threaded_scheduler()
        .build()
        .expect("error on building runtime");

    let args: Vec<String> = env::args().collect();
    let n: u16 = (&args[1]).parse().unwrap();

    let app = app::App::new(n);

    if let Err(error) = runtime.block_on(app.start()) {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }

    std::process::exit(0);
}
