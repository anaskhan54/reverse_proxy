use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    env
};

fn main() {

    // get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 7 {
        eprintln!("Error: Missing required arguments.");
        println!("Usage: {} -domain <domain> -lhost <lhost> -lport <lport>", args[0]);
        return;
    }

    let mut domain = None;
    let mut lhost = None;
    let mut lport = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-domain" => {
                domain = Some(args[i + 1].clone());
                i += 2;
            }
            "-lhost" => {
                lhost = Some(args[i + 1].clone());
                i += 2;
            }
            "-lport" => {
                lport = args[i + 1].parse::<u16>().ok();
                i += 2;
            }
            _ => {
                eprintln!("Error: Invalid argument '{}'", args[i]);
                return;
            }
        }
    }
//check if argument valid, then print help message accordingly

    match (domain.as_ref(), lhost.as_ref(), lport) {
        (Some(domain), Some(lhost), Some(lport)) => {
            println!("Domain: {}", domain);
            println!("Lhost: {}", lhost);
            println!("Lport: {}", lport);
        }
        _ => {
            eprintln!("Error: Missing required arguments.");
            println!("Usage: {} -domain <domain> -lhost <lhost> -lport <lport>", args[0]);
        }
    }
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, domain.as_ref().unwrap(), lhost.as_ref().unwrap(), lport.unwrap());
    }
}



fn handle_connection(mut stream: TcpStream, domain: &str, lhost: &str, lport: u16) {
    // Reading the data in the packet
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    // Extracting Host Header from the request
    let mut host_value = None;
    for line in &http_request {
        if line.starts_with("Host:") {
            host_value = Some(line.trim_start_matches("Host: ").trim().to_string());
            break;
        }
    }

    match host_value {
        Some(value) => {
            println!("Host value: {}", value);
            if value == domain {
                println!("Host value matches domain: {}", domain);

                // Sending the http_request to another http server specified by lhost and lport
                let destination = format!("{}:{}", lhost, lport);
                let mut destination_stream = TcpStream::connect(destination).unwrap();
                for line in &http_request {
                    destination_stream.write_all(line.as_bytes()).unwrap();
                    destination_stream.write_all(b"\r\n").unwrap();
                }
                destination_stream.write_all(b"\r\n").unwrap();

                // Forwarding the response from the destination server back to the client
                let mut response = String::new();
                let mut response_reader = BufReader::new(&mut destination_stream);
                response_reader.read_to_string(&mut response).unwrap();

                // Sending the response back to the client
                stream.write_all(response.as_bytes()).unwrap();
            } else {
                println!("Host value does not match domain: {}", domain);
            }
        }
        None => println!("Host header not found"),
    }
}


