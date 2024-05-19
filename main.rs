use async_std::{
    io::{BufReader, Result},
    net::{TcpListener, TcpStream},
    prelude::*,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Write;
use std::io::{self, BufRead};
use std::process::Command;
use std::{env, fs, str};

/*
*kill the instances of the program running on the system
*This function is used to kill all the instances of the program running on the system
*/
#[cfg(target_os = "windows")]
fn kill_instances() {
    Command::new("taskkill")
        .args(&["/F", "/IM", "reverse_proxy.exe"])
        .output()
        .expect("Failed to execute taskkill command");
}

#[cfg(target_os = "linux")]
fn kill_instances() {
    Command::new("pkill")
        .args(&["reverse_proxy"])
        .output()
        .expect("Failed to execute pkill command");
}

#[async_std::main]
async fn main() {
    // get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if args[1] == "start" {
            //check if config file exists

            let file_name = if cfg!(windows) {
                "reverse_proxy.conf"
            } else {
                "./reverse_proxy.conf"
            };

            if fs::metadata(file_name).is_ok() {
                // check if the file is valid, if yes then print what it got
                let mut proxy_map = std::collections::HashMap::new();

                if let Ok(file) = fs::File::open(&file_name) {
                    let lines = io::BufReader::new(file).lines();

                    for line_result in lines {
                        if let Ok(line_contents) = line_result {
                            let parts: Vec<&str> = line_contents.split_whitespace().collect();

                            if parts.len() == 3 {
                                let domain = parts[0].to_string();
                                let lhost = parts[1].to_string();
                                let lport = parts[2].to_string();
                                proxy_map.insert(domain, (lhost, lport));
                            } else {
                                println!("Invalid config file: {}", file_name);
                                return;
                            }
                        }
                    }
                } else {
                    println!("Unable to open config file: {}", file_name);
                    return;
                }
                start_proxy_server(proxy_map).await;
            } else {
                println!("No config file found, please add proxy using the command `reverse_proxy add_proxy`");
            }
        } else if args[1] == "add_proxy" {
            if args.len() < 7 {
                eprintln!("Error: Missing required arguments for 'start' command.");
                println!(
                    "Usage: {} add_proxy -domain <domain> -lhost <lhost> -lport <lport>",
                    args[0]
                );
                return;
            }

            let mut domain = None;
            let mut lhost = None;
            let mut lport = None;

            let mut i = 2;
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

            match (domain.as_ref(), lhost.as_ref(), lport) {
                (Some(domain), Some(lhost), Some(lport)) => {
                    println!("Starting the program...");
                    println!("Domain: {}", domain);
                    println!("Lhost: {}", lhost);
                    println!("port: {}", lport);

                    let content = format!("{}  {}  {}\n", domain, lhost, lport);

                    let file = File::open("reverse_proxy.conf");
                    match file {
                        Ok(file) => file,
                        Err(_) => File::create("reverse_proxy.conf").unwrap(),
                    };
                    println!("\x1b[33m......Appending proxy to the config file.....\x1b[0m");

                    let mut data_file = File::open("reverse_proxy.conf").unwrap();

                    // Create an empty mutable string
                    let mut file_content = String::new();

                    // Copy contents of file to a mutable string
                    data_file.read_to_string(&mut file_content).unwrap();

                    // println!("File content: {:?}", file_content);
                    if file_content.contains(&domain.to_string()) {
                        println!("\x1b[31m.........Proxy already exists.........\x1b[0m");
                        return;
                    }
                    if file_content.contains(&lport.to_string()) {
                        println!("\x1b[31m.........Port already in use.........\x1b[0m");
                        return;
                    }
                    let mut conf_file = OpenOptions::new()
                        .append(true)
                        .open("reverse_proxy.conf")
                        .expect("Cannot open file");

                    conf_file.write_all(content.as_bytes()).unwrap();
                    println!("\x1b[32m.........Proxy added successfully.........\x1b[0m")
                }
                _ => {
                    eprintln!("Error: Missing required arguments.");
                    println!(
                        "Usage: {} add_proxy -domain <domain> -lhost <lhost> -lport <lport>",
                        args[0]
                    );
                }
            }
        } else if args[1] == "stop" {
            // Call a function to kill all instances of this program
            kill_instances();
        } else {
            eprintln!("Error: Invalid command '{}'", args[1]);
            println!("Usage: {} [start|add_proxy|stop]", args[0]);
        }
    } else {
        eprintln!("Error: Missing command.");
        println!("Usage: {} [add_proxy|stop]", args[0]);
    }
}

async fn start_proxy_server(proxy_map: HashMap<String, (String, String)>) {
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("\x1b[33mproxy server started on port 80..\x1b[0m");

    listener
        .incoming()
        .for_each_concurrent(None, |tcp_stream| async {
            if let Ok(tcp_stream) = tcp_stream {
                if let Err(e) = handle_connection(tcp_stream, proxy_map.clone()).await {
                    eprintln!("Error: {:?}", e);
                }
            }
        })
        .await;
}

async fn handle_connection(
    mut stream: TcpStream,
    proxy_map: HashMap<String, (String, String)>,
) -> Result<()> {
    println!("\x1b[32mConnection established\x1b[0m");

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

    println!("\x1b[33mRequest Recieved:{:?}\x1b[0m", req);

    let host_value = headers
        .lines()
        .find(|line| line.starts_with("Host:"))
        .and_then(|line| line.split(":").nth(1).map(|value| value.trim()));

    println!("Host value: {:?}", host_value);

    match host_value {
        Some(value) => match proxy_map.get(value) {
            Some((lhost, lport)) => {
                println!("Host value matches domain: {}", value);

                let addr = format!("{}:{}", lhost, lport);
                println!("Destination server address: {}", addr);

                let mut destination_stream = TcpStream::connect(addr).await.unwrap();
                println!("\x1b[36mConnected to destination server\x1b[1m");

                destination_stream.write_all(req.as_bytes()).await.unwrap();
                destination_stream.flush().await.unwrap();

                println!("\x1b[32mRequest forwarded to destination server\x1b[1m");

                let mut buf = [0u8; 4096];
                loop {
                    let bytes_read = destination_stream.read(&mut buf).await?;
                    if bytes_read == 0 {
                        break;
                    }
                    stream.write_all(&buf[..bytes_read]).await?;
                }

                println!("Response forwarded to client");
            }
            None => println!("Host value exists but not found in proxy map"),
        },
        None => println!("Host header not found"),
    }

    Ok(())
}
