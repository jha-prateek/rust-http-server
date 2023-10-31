mod http_request;
mod http_utils;

use std::{env, thread};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*};
use std::ops::Not;
use chrono::Local;
use crate::http_request::RequestContext;
use crate::http_utils::{ContentType, HttpStatus};

fn main() {
    let port = "4222";
    let address = format!("127.0.0.1:{}", port);
    println!("Starting TCP server on {}", address);
    let listener = TcpListener::bind(address).unwrap();

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

        let file = File::create(abs_path);
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
