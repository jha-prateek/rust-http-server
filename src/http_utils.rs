use std::fmt;
use chrono::Local;

#[derive(Debug, Clone, Copy)]
pub enum ContentType {
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
pub enum HttpStatus {
    Ok,
    Created,
    NotFound,
    BadRequest
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpStatus::Ok => write!(f, "200 OK"),
            HttpStatus::Created => write!(f, "201 Created"),
            HttpStatus::NotFound => write!(f, "404 Not Found"),
            HttpStatus::BadRequest => write!(f, "400 Bad Request")
        }
    }
}

pub fn prepare_response(status: HttpStatus, content_type: ContentType, body: &str) -> String {
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

pub fn log_error(msg: &str) {
    eprintln!("[{}] {}", get_current_time_str(), msg)
}

pub fn get_current_time_str() -> String {
    format!("{}", Local::now().format("%d/%m/%Y %H:%M:%S"))
}