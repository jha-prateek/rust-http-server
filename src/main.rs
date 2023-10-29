use std::collections::HashMap;
use std::{fmt, thread};
// Uncomment this block to pass the first stage
use std::net::{Shutdown, TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::ops::Not;
use std::str::from_utf8;
use chrono::Local;
use itertools::Itertools;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                thread::spawn(|| {
                    handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection_raw(mut stream: TcpStream) {
    let current_time = format!("{}", Local::now().format("%d/%m/%Y %H:%M:%S"));
    let read_buffer = &mut [0u8; 1024];
    stream.read(read_buffer).expect("TODO: panic message");
    let request = from_utf8(read_buffer).unwrap();
    let request_parts: Vec<&str> = request.split(" ").collect();
    let method = request_parts[0];
    let path = request_parts[1];
    println!("[{:?}] {} {}", current_time, method, path);
    let response_status = match path {
        "/" => "200 OK",
        _ => "404 Not Found"
    };
    let response = format!("HTTP/1.1 {}\r\n\r\n", response_status);
    stream.write_all(response.as_bytes()).unwrap();
    stream.shutdown(Shutdown::Both).expect("TODO: panic message");
}

fn handle_connection(mut stream: TcpStream) {
    let request_context = RequestContext::prepare_request(&stream);

    println!("[{:?}] {}", get_current_time_str(), request_context.request_info());

    let response = match request_context.path.as_str() {
        p if p.contains("/echo") => handle_echo(p),
        p if p.contains("/user-agent") => handle_user_agent(request_context.headers),
        "/" => prepare_response(HttpStatus::Ok, ContentType::Unknown, ""),
        _ => prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
    };
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_user_agent(headers: HashMap<String, String>) -> String {
    prepare_response(HttpStatus::Ok, ContentType::TextPlain, headers.get("User-Agent").unwrap())
}

fn handle_echo(path: &str) -> String {
    let body = path.strip_prefix("/echo/").unwrap();
    prepare_response(HttpStatus::Ok, ContentType::TextPlain, body)
}

fn prepare_response(status: HttpStatus, content_type: ContentType, body: &str) -> String {
    match content_type {
        ContentType::TextPlain => {
            format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                status,
                content_type,
                body.len(),
                body
            )
        }
        ContentType::Unknown => {
            match status {
                HttpStatus::Ok => format!("HTTP/1.1 200 OK\r\n\r\n"),
                HttpStatus::NotFound => format!("HTTP/1.1 404 Not Found\r\n\r\n")
            }
        }
    }
}

fn get_current_time_str() -> String {
    format!("{}", Local::now().format("%d/%m/%Y %H:%M:%S"))
}

struct RequestContext {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>
}

impl RequestContext {
    fn prepare_request(stream: &TcpStream) -> RequestContext {
        let mut buffered_reader = BufReader::new(stream);
        let incoming_request_vec = buffered_reader.lines()
            .map(|line| line.unwrap())
            .take_while(|line| line.is_empty().not())
            .collect::<Vec<_>>();

        let start_line = incoming_request_vec[0].split_whitespace().collect::<Vec<&str>>();
        let method = start_line[0].to_string();
        let path = start_line[1].to_string();
        let version = start_line[2].to_string();

        let mut headers: HashMap<String, String> = HashMap::new();
        incoming_request_vec
            .iter()
            .skip(1)
            .for_each(|line| {
                let parts = line.split(": ").map(|p| p.trim()).collect::<Vec<&str>>();
                let key = parts[0];
                let value = parts[1];
                headers.insert(key.to_string(), value.to_string());
            });

        RequestContext {
            method,
            path,
            version,
            headers
        }
    }

    fn request_info(&self) -> String {
        format!("{} {} {}", self.method, self.path, self.version)
    }
}

#[derive(Debug, Clone, Copy)]
enum ContentType {
    TextPlain,
    Unknown
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::TextPlain => write!(f, "text/plain"),
            ContentType::Unknown => write!(f, "")
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HttpStatus {
    Ok,
    NotFound
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpStatus::Ok => write!(f, "200 OK"),
            HttpStatus::NotFound => write!(f, "404 Not Found")
        }
    }
}