use std::collections::HashMap;
use std::{env, fmt, thread};
use std::fs::File;
// Uncomment this block to pass the first stage
use std::net::{Shutdown, TcpListener, TcpStream};
use std::io::{prelude::*};
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
    let request_context = RequestContext::prepare_request(&mut stream);

    println!("[{:?}] {}", get_current_time_str(), request_context.request_info());

    let response = match request_context.path.as_str() {
        p if p.contains("/echo") => handle_echo(request_context),
        p if p.contains("/user-agent") => handle_user_agent(request_context),
        p if p.contains("/files") && request_context.method == "GET" => file_get(request_context),
        p if p.contains("/files") && request_context.method == "POST" => file_post(request_context),
        "/" => prepare_response(HttpStatus::Ok, ContentType::Unknown, ""),
        _ => prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
    };
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn file_post(request: RequestContext) -> String {
    let directory = get_args_value("directory");

    if directory.is_empty().not()
        && request.path_params.is_empty().not()
        && request.path_params[0].to_string().is_empty().not() {

        let abs_path = format!("{}/{}", directory, request.path_params[0]);

        let mut file = File::create(abs_path);
        match file {
            Ok(mut f) => {
                match f.write_all(request.body.as_bytes()) {
                    Ok(_) => prepare_response(HttpStatus::Created, ContentType::Unknown, ""),
                    Err(_) => prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
                }
            }
            Err(_) => prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
        }
    } else {
        prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
    }
}

fn file_get(request: RequestContext) -> String {

    let directory = get_args_value("directory");

    if directory.is_empty().not()
        && request.path_params.is_empty().not()
        && request.path_params[0].to_string().is_empty().not() {
        match File::open(format!("{}/{}", directory, request.path_params[0])) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                prepare_response(HttpStatus::Ok, ContentType::OctetStream, contents.as_str())
            }
            Err(_) => prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
        }
    } else {
        prepare_response(HttpStatus::NotFound, ContentType::Unknown, "")
    }
}

fn handle_user_agent(request: RequestContext) -> String {
    prepare_response(HttpStatus::Ok, ContentType::TextPlain, request.headers.get("User-Agent").unwrap())
}

fn handle_echo(request: RequestContext) -> String {
    let body = request.path.strip_prefix("/echo/").unwrap();
    prepare_response(HttpStatus::Ok, ContentType::TextPlain, body)
}

fn prepare_response(status: HttpStatus, content_type: ContentType, body: &str) -> String {
    match content_type {
        ContentType::TextPlain | ContentType::OctetStream => {
            format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                status,
                content_type,
                body.len(),
                body
            )
        }
        ContentType::Unknown => format!("HTTP/1.1 {}\r\n\r\n", status)
    }
}

fn get_args_value(arg_label: &str) -> String {
    let args = env::args().collect::<Vec<String>>();
    let arg_value = args.iter()
        .skip(1)
        .skip_while(|a| a.contains(format!("--{}", arg_label).as_str()))
        .next();

    match arg_value {
        None => "".to_string(),
        Some(a) => a.to_string()
    }
}

fn get_current_time_str() -> String {
    format!("{}", Local::now().format("%d/%m/%Y %H:%M:%S"))
}

struct RequestContext {
    method: String,
    path: String,
    path_params: Vec<String>,
    version: String,
    headers: HashMap<String, String>,
    body: String
}

impl RequestContext {
    fn prepare_request(mut stream: &TcpStream) -> RequestContext {
        // let mut buffered_reader = BufReader::new(stream);
        // let incoming_request_vec = buffered_reader.lines()
        //     .map(|line| line.unwrap())
        //     .take_while(|line| line.is_empty().not())
        //     .collect::<Vec<_>>();

        let read_buffer = &mut [0u8; 1024];
        stream.read(read_buffer).expect("Unable to read stream");
        let request = from_utf8(read_buffer).unwrap().trim_matches(char::from(0));
        let request_parts= request
            .split("\r\n\r\n").map(|a| a.lines().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let header_parts = request_parts[0]
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>();

        let body = if request_parts[1].is_empty().not() {
            request_parts[1][0].to_string()
        } else { String::from("") };

        let start_line = header_parts[0].split_whitespace().collect::<Vec<&str>>();
        let method = start_line[0].to_string();
        let path = start_line[1].to_string();
        let version = start_line[2].to_string();

        let path_params = path.strip_prefix("/").unwrap()
            .split("/")
            .skip(1)
            .map(|p| p.to_string())
            .collect::<Vec<String>>();

        let mut headers: HashMap<String, String> = HashMap::new();
        header_parts
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
            path_params,
            version,
            headers,
            body
        }
    }

    fn request_info(&self) -> String {
        format!("{} {} {}", self.method, self.path, self.version)
    }
}

#[derive(Debug, Clone, Copy)]
enum ContentType {
    TextPlain,
    OctetStream,
    Unknown
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::TextPlain => write!(f, "text/plain"),
            ContentType::OctetStream => write!(f, "application/octet-stream"),
            ContentType::Unknown => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HttpStatus {
    Ok,
    Created,
    NotFound
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpStatus::Ok => write!(f, "200 OK"),
            HttpStatus::Created => write!(f, "201 Created"),
            HttpStatus::NotFound => write!(f, "404 Not Found")
        }
    }
}