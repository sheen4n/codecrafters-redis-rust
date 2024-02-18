use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    thread,
};

struct RESPDataType;

impl RESPDataType {
    const ARRAY: u8 = b'*';
    const BULK: u8 = b'$';
}

fn set_db(db: &Mutex<HashMap<String, String>>, key: &str, value: &str) -> String {
    if let Ok(mut db) = db.lock() {
        db.insert(key.to_string(), value.to_string());
    }

    "+OK\r\n".to_string()
}

fn get_db(db: &Mutex<HashMap<String, String>>, key: &str) -> String {
    if let Ok(db) = db.lock() {
        if let Some(value) = db.get(&key.to_string()) {
            return format!("${}\r\n{}\r\n", value.len(), value);
        }
        return "$-1\r\n".to_string();
    }
    "".to_string()
}

fn evaluate_resp(mut cmd: &[u8], db: &Mutex<HashMap<String, String>>) -> String {
    //  *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n

    let mut len: u8 = 0;
    if cmd[0] == RESPDataType::ARRAY {
        len = cmd[1] - b'0';
        //  $4\r\nECHO\r\n$3\r\nhey\r\n
        cmd = &cmd[4..];
    }

    match cmd[0] {
        RESPDataType::BULK => {
            let args = get_args(cmd, len);
            match &args[0][..] {
                "ping" => "+PONG\r\n".to_string(),
                "echo" => format!("+{}\r\n", args[1]),
                "set" => set_db(db, &args[1], &args[2]),
                "get" => get_db(db, &args[1]),
                _ => "-not_supported command".to_string(),
            }
        }
        _ => "-not_supported data type\r\n".to_string(),
    }
}

fn get_args(mut cmd: &[u8], mut len: u8) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    while len > 0 {
        // $4\r\nECHO\r\n$3\r\nhey\r\n
        let arg_len: usize = (cmd[1] - b'0') as usize;
        let arg = String::from_utf8_lossy(&cmd[4..arg_len + 4]);
        args.push(arg.into());
        cmd = &cmd[arg_len + 4 + 2..];
        len -= 1;
    }
    args
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    let db: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    println!("Logs from your program will appear here!");
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").expect("could not bind");
    thread::scope(|s| {
        for stream in listener.incoming() {
            let db = &db;
            s.spawn(move || match stream {
                Ok(mut stream) => {
                    println!("Accepted new connection");
                    handle_stream(&mut stream, &db);
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            });
        }
    });
}

fn handle_stream(stream: &mut TcpStream, db: &Mutex<HashMap<String, String>>) {
    let mut buffer = [0u8; 120];

    while let Ok(_) = stream.read(&mut buffer) {
        let val: String = evaluate_resp(&mut buffer, &db);
        _ = stream.write(&val.as_bytes())
    }
}
