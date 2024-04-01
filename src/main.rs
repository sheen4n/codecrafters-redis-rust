mod redis_handler;
mod request;
mod response;
mod server;
use std::env::args;

use server::Server;

fn main() {
    println!("Logs from your program will appear here!");
    let mut port = "6379";

    let args: Vec<_> = args().collect();
    println!("args: {:?}", args);
    if args.len() > 1 && args[1] == "--port" {
        port = &args[2]
    }

    let mut master_host: Option<&str> = None;
    let mut master_port: Option<&str> = None;
    if args.len() > 3 && args[3] == "--replicaof" {
        master_host = Some(&args[4]);
        master_port = Some(&args[5]);
    }

    let server = Server::new("127.0.0.1", port, master_host, master_port);
    server.run();
}
