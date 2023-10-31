mod http_request;
mod http_utils;

use std::{env, thread};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*};
use crate::http_request::RequestContext;
use crate::http_utils::{ContentType, get_current_time_str, HttpStatus, log_error, prepare_response};

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
    println!("[{}] {}", get_current_time_str(), request_context.request_info());

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
    let directory = match get_args_value("directory") {
        Ok(d) => d,
        Err(e) => {
            return prepare_response(HttpStatus::BadRequest, ContentType::TextPlain, e.as_str())
        }
    };

    let response: Result<_, String> = if directory.is_empty() {
        let e_msg = format!("argument --directory not found");
        log_error(&e_msg);
        Err(e_msg)
    } else if request.path_params.is_empty() || request.path_params[0].to_string().is_empty() {
        let e_msg = format!("file name not found");
        log_error(&e_msg);
        Err(e_msg)
    } else {
        let abs_path = format!("{}/{}", directory, request.path_params[0]);
        let file = File::create(abs_path);
        match file {
            Ok(mut f) => {
                match f.write_all(request.body.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let e_msg = format!("Error while writing file: {}", e);
                        log_error(&e_msg);
                        Err(e_msg)
                    }
                }
            }
            Err(e) => {
                let e_msg = format!("Error: {}", e);
                log_error(&e_msg);
                Err(e_msg)
            }
        }
    };

    match response {
        Ok(_) => prepare_response(HttpStatus::Created, ContentType::Unknown, ""),
        Err(e) => prepare_response(HttpStatus::NotFound, ContentType::TextPlain, e.as_str())
    }
}

fn file_get(request: RequestContext) -> String {
    let directory = match get_args_value("directory") {
        Ok(d) => d,
        Err(e) => {
            return prepare_response(HttpStatus::BadRequest, ContentType::TextPlain, e.as_str())
        }
    };

    let response: Result<String, String> = if directory.is_empty() {
        let e_msg = format!("argument --directory not found");
        log_error(&e_msg);
        Err(e_msg)
    } else if request.path_params.is_empty() || request.path_params[0].to_string().is_empty() {
        let e_msg = format!("file name not found");
        log_error(&e_msg);
        Err(e_msg)
    } else {
        match File::open(format!("{}/{}", directory, request.path_params[0])) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                Ok(contents.clone())
            }
            Err(e) => {
                let e_msg = format!("Error: {}", e);
                log_error(&e_msg);
                Err(e_msg)
            }
        }
    };

    match response {
        Ok(contents) => prepare_response(HttpStatus::Ok, ContentType::OctetStream, contents.as_str()),
        Err(e) => prepare_response(HttpStatus::NotFound, ContentType::TextPlain, e.as_str())
    }
}

fn handle_user_agent(request: RequestContext) -> String {
    match request.headers.get("User-Agent") {
        None => prepare_response(HttpStatus::BadRequest, ContentType::TextPlain, "header User-Agent not found"),
        Some(_) => prepare_response(HttpStatus::Ok, ContentType::TextPlain, request.headers.get("User-Agent").unwrap())
    }
}

fn handle_echo(request: RequestContext) -> String {
    if request.path_params.is_empty() {
        prepare_response(HttpStatus::BadRequest, ContentType::TextPlain, "no path params found")
    } else {
        prepare_response(HttpStatus::Ok, ContentType::TextPlain, request.path_params.join(" ").as_str())
    }
}

fn get_args_value(arg_label: &str) -> Result<String, String> {
    let args = env::args().collect::<Vec<String>>();
    let arg_value = args.iter()
        .skip(1)
        .skip_while(|a| a.contains(format!("--{}", arg_label).as_str()))
        .next();

    match arg_value {
        None => {
            let e_msg = format!("argument --directory not found");
            log_error(&e_msg);
            Err(e_msg)
        }
        Some(a) => Ok(a.to_string())
    }
}
