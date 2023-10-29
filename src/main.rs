use std::fmt;
// Uncomment this block to pass the first stage
use std::net::{Shutdown, TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::str::from_utf8;
use chrono::Local;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                handle_connection_neat(_stream);
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

fn handle_connection_neat(mut stream: TcpStream) {
    let mut buffered_reader = BufReader::new(&stream);
    let mut incoming_request_string = String::new();
    buffered_reader.read_line(&mut incoming_request_string).unwrap();
    let request_parts: Vec<&str> = incoming_request_string.split_whitespace().collect();
    request_parts[0];
    let path = request_parts[1];
    println!("[{:?}] {}", get_current_time_str(), incoming_request_string);
    let response = match path {
        p if p.contains("/echo") => {
            let body = p.strip_prefix("/echo/").unwrap();
            prepare_response(ContentType::TextPlain, body)
        }
        "/" => prepare_ok(),
        _ => prepare_404()
    };
    stream.write_all(response.as_bytes()).unwrap();
}

fn prepare_response(content_type: ContentType, body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        content_type,
        body.len(),
        body
    )
}

fn prepare_ok() -> String {
    format!("HTTP/1.1 200 OK\r\n\r\n")
}

fn prepare_404() -> String {
    format!("HTTP/1.1 404 Not Found\r\n\r\n")
}

fn get_current_time_str() -> String {
    format!("{}", Local::now().format("%d/%m/%Y %H:%M:%S"))
}

#[derive(Debug, Clone, Copy)]
enum ContentType {
    TextPlain
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::TextPlain => write!(f, "text/plain")
        }
    }
}