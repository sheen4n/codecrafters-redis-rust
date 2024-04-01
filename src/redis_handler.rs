use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{
    request::{ParseError, RedisCommand},
    response::Response,
    server::Server,
};

fn set_db(
    db: &Mutex<HashMap<String, (String, Instant)>>,
    key: &str,
    value: &str,
    expiry: Option<u64>,
) -> String {
    let now = Instant::now();
    let expiry_time = match expiry {
        Some(expiry) => now + Duration::from_millis(expiry),
        None => now + Duration::from_millis(u64::MAX),
    };

    if let Ok(mut db) = db.lock() {
        db.insert(key.to_string(), (value.to_string(), expiry_time));
    }

    "+OK\r\n".to_string()
}

fn get_db(db: &Mutex<HashMap<String, (String, Instant)>>, key: &str) -> String {
    if let Ok(db) = db.lock() {
        if let Some(value) = db.get(&key.to_string()) {
            if value.1 < Instant::now() {
                return "$-1\r\n".to_string();
            }
            let result = &value.0;
            return format!("${}\r\n{}\r\n", result.len(), result);
        }
    }
    "$-1\r\n".to_string()
}

pub struct RedisHandler {}

impl RedisHandler {
    pub fn new() -> Self {
        Self {}
    }
    // "get" => get_db(db, &args[1]),
    // "info" => "+role:master\r\n".to_string(),
    pub fn handle_request(
        &mut self,
        request: &crate::request::Request,
        server: &Server,
    ) -> crate::response::Response {
        println!("{:?}", request);
        let db = &server.db;
        match request.command() {
            &RedisCommand::PING => Response::new("+PONG\r\n".to_string()),
            &RedisCommand::ECHO => Response::new(format!("+{}\r\n", request.payload()[1])),

            &RedisCommand::SET => Response::new(set_db(
                db,
                &request.payload()[1],
                &request.payload()[2],
                request.expiry(),
            )),
            &RedisCommand::GET => Response::new(get_db(db, &request.payload()[1])),
            &RedisCommand::INFO => match &server.master_host {
                Some(_) => Response::new("+role:slave\r\n".to_string()),
                None => Response::new("+role:master\r\n".to_string()),
            },
            // _ => Response::new("-not_supported command".to_string()),
        }
    }

    pub fn handle_bad_request(&mut self, e: &ParseError) -> crate::response::Response {
        println!("{:?}", e);
        Response::new("invalid_request\r\n".to_string())
    }
}
