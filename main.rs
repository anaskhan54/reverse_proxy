use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
};
use futures::StreamExt;
use std::{env, io::prelude::*, str, time::Duration};

#[async_std::main]
async fn main() {
    // get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 7 {
        eprintln!("Error: Missing required arguments.");
        println!(
            "Usage: {} -domain <domain> -lhost <lhost> -lport <lport>",
            args[0]
        );
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
            println!(
                "Usage: {} -domain <domain> -lhost <lhost> -lport <lport>",
                args[0]
            );
        }
    }
    let listener = TcpListener::bind("127.0.0.1:80").await.unwrap();

    listener
        .incoming()
        .for_each_concurrent(None, |tcp_stream| async {
            let tcp_stream = tcp_stream.unwrap();
            handle_connection(
                tcp_stream,
                domain.as_ref().unwrap(),
                lhost.as_ref().unwrap(),
                lport.unwrap(),
            )
            .await;
        })
        .await;
}

async fn handle_connection(mut stream: TcpStream, domain: &str, lhost: &str, lport: u16) {
    //store the tcp stream in a buffer reader to imporve efficiency
    println!("Connection established");
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    async_std::task::sleep(Duration::from_secs(15)).await;
    buf_reader.read_line(&mut request_line).await.unwrap();

    // Parse the HTTP request line
    println!("Request Line: {}", request_line);

    let mut parts = request_line.trim().split_whitespace();

    let method = parts.next().unwrap_or("UNKNOWN_METHOD");
    let path = parts.next().unwrap_or("UNKNOWN_PATH");
    let http_version = parts.next().unwrap_or("UNKNOWN_HTTP_VERSION");
    println!(
        "Method: {}, Path: {}, HTTP Version: {}",
        method, path, http_version
    );

    // Read the HTTP headers until an empty line is encountered
    let mut headers = String::new();
    loop {
        let mut line = String::new();

        buf_reader.read_line(&mut line).await.unwrap();
        if line.trim().is_empty() {
            break;
        }
        println!("header line: {}", line.trim());
        headers.push_str(&line);
    }
    for line in headers.lines() {
        if line.trim().starts_with("Host:") {
            println!("Host header found");
            let host_value = line.trim_start_matches("Host: ").trim();
            println!("Host value: {}", host_value);

            if host_value == domain {
                println!("Host value matches domain: {}", domain);
                let addr = format!("{}:{}", lhost, lport);
                let mut destination_stream = TcpStream::connect(addr).await.unwrap();

                println!("Request Line: {:?}", destination_stream);

                destination_stream
                    .write_all(request_line.as_bytes())
                    .await
                    .unwrap();
            } else {
                println!("Host value does not match domain: {}", domain)
            }
        }
    }

    // Read the HTTP request body if present
    let mut body = String::new();
    buf_reader.read_to_string(&mut body).await.unwrap();
    println!("Body:\n{}", body);
}
// //&need better error handling here
// let http_request: Vec<_> = buf_reader
//     .lines()
//     .map(|result| result.unwrap())
//     .take_while(|line| !line.is_empty())
//     .collect();

// println!("Request: {:#?}", http_request);
// println!("Request headers: {:#?}", http_request[1].trim());
// async_std::task::sleep(Duration::from_secs(10)).await;
// // Extracting Host Header from the request
// let mut host_value = None;
// for line in &http_request {
//     if line.starts_with("Host:") {
//         host_value = Some(line.trim_start_matches("Host: ").trim().to_string());
//         break;
//     }
// }

// match host_value {
//     Some(value) => {
//         println!("Host value: {}", value);
//         if value == domain {
//             println!("Host value matches domain: {}", domain);

//             // Sending the http_request to another http server specified by lhost and lport
//             let destination = format!("{}:{}", lhost, lport);
//             let mut destination_stream = TcpStream::connect(destination).unwrap();
//             for line in &http_request {
//                 destination_stream.write_all(line.as_bytes()).unwrap();
//                 destination_stream.write_all(b"\r\n").unwrap();
//             }
//             destination_stream.write_all(b"\r\n").unwrap();

//             // Forwarding the response from the destination server back to the client
//             let mut response = String::new();
//             let mut response_reader = BufReader::new(&mut destination_stream);
//             response_reader.read_to_string(&mut response).unwrap();

//             // Sending the response back to the client
//             stream.write_all(response.as_bytes()).unwrap();
//         } else {
//             println!("Host value does not match domain: {}", domain);
//         }
//     }
//     None => println!("Host header not found"),
// }
