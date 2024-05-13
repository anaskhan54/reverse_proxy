use async_std::{
    io::{BufReader, Result},
    net::{TcpListener, TcpStream},
    prelude::*,
};
use futures::StreamExt;
use std::{env, str};

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
    println!("\x1b[33mproxy server started on port 80..\x1b[0m");

    listener
        .incoming()
        .for_each_concurrent(None, |tcp_stream| async {
            if let Ok(tcp_stream) = tcp_stream {
                if let Err(e) = handle_connection(
                    tcp_stream,
                    domain.as_ref().unwrap(),
                    lhost.as_ref().unwrap(),
                    lport.unwrap(),
                )
                .await
                {
                    eprintln!("Error: {:?}", e);
                }
            }
        })
        .await;
}

async fn handle_connection(
    mut stream: TcpStream,
    domain: &str,
    lhost: &str,
    lport: u16,
) -> Result<()> {
    println!("Connection established");

    let mut request = String::new();
    let mut headers = String::new();

    //creating a buffer to store the request
    let mut buf_reader = BufReader::new(&mut stream);

    // Read the HTTP request line
    buf_reader.read_line(&mut request).await?;

    println!("Request: {:?}", request);

    // Read the HTTP request headers until an empty line is encountered
    loop {
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            break;
        }

        headers.push_str(&line);
    }
    println!("Headers: {:?}", headers);

    let req = request + headers.as_str() + "\r\n";

    println!("Request Recieved:\n{:?}", req);

    let host_value = headers
        .lines()
        .find(|line| line.starts_with("Host:"))
        .and_then(|line| line.split(":").nth(1).map(|value| value.trim()));

    println!("Host value: {:?}", host_value);

    match host_value {
        Some(value) if value == domain => {
            println!("Host value matches domain: {}", domain);

            let addr = format!("{}:{}", lhost, lport);
            println!("Destination server address: {}", addr);

            let mut destination_stream = TcpStream::connect(addr).await.unwrap();
            println!("Connected to destination server");

            destination_stream.write_all(req.as_bytes()).await.unwrap();
            destination_stream.flush().await.unwrap();

            println!("Request forwarded to destination server");

            let mut response = String::new();

            let mut response_reader = BufReader::new(&mut destination_stream);

            response_reader.read_line(&mut response).await.unwrap();

            stream.write_all(response.as_bytes()).await.unwrap();
            println!("{:?}", response);
            println!("Response forwarded to client");
        }
        Some(_) => println!("Host value does not match domain: {}", domain),
        None => println!("Host header not found"),
    }

    Ok(())
}
