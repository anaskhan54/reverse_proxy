use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{
    body::{Body, Incoming},
    server::conn::http1,
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use std::{
    convert::Infallible,
    env,
    io::{prelude::*, BufReader},
    net::SocketAddr,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream;

static mut DOMAIN: Option<String> = None;
static mut LHOST: Option<String> = None;
static mut LPORT: Option<u16> = None;

fn get_command_line_arguments() -> Option<(String, String, u16)> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 7 {
        eprintln!("Error: Missing required arguments.");
        println!("Usage:  -domain <domain> -lhost <lhost> -lport <lport>",);
        return None;
    }

    // let mut domain: Option<String> = None;
    // let mut lhost = None;
    // let mut lport = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-domain" => {
                unsafe { DOMAIN = Some(args[i + 1].clone()) };
                i += 2;
            }
            "-lhost" => {
                unsafe { LHOST = Some(args[i + 1].clone()) };
                i += 2;
            }
            "-lport" => {
                unsafe { LPORT = args[i + 1].parse::<u16>().ok() };
                i += 2;
            }
            _ => {
                eprintln!("Error: Invalid argument '{}'", args[i]);
                return None;
            }
        }
    }
    //check if argument valid, then print help message accordingly

    match (
        unsafe { DOMAIN.clone() },
        unsafe { LHOST.clone() },
        unsafe { LPORT.clone() },
    ) {
        (Some(domain), Some(lhost), Some(lport)) => Some((lhost, domain, lport)),
        _ => {
            eprintln!("Error: Missing required arguments.");
            println!(
                "Usage: {} -domain <domain> -lhost <lhost> -lport <lport>",
                args[0]
            );
            None
        }
    }
}
async fn proxy(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let mut host: String = String::new();
    if let Some(value) = req.headers().get("host") {
        host = value.to_str().unwrap().to_string();
    }
    if let Some(domain) = unsafe { DOMAIN.clone() } {
        println!("Domain:{}", domain);
        if host == domain {
            unsafe {
                println!("Host value matches domain: {}", DOMAIN.as_ref().unwrap());
            }
        }
        // let mut port = LPORT.clone().unwrap();
        // let mut address = format!("http://{}{}", domain, port);

        // let stream = TcpStream::connect(address).await.unwrap();
        // let io = TokioIo::new(stream);

        // let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        // tokio::task::spawn(async move {
        //     if let Err(err) = conn.await {
        //         println!("Connection failed: {:?}", err);
        //     }
        // });
    }

    println!("Request received {:?}", req);
    println!("Request headers: {:?}", req.headers());
    println!("Request host: {:?}", req.headers().get("host").unwrap());
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // get command line arguments
    // let (domain, lhost, lport) = match get_command_line_arguments() {
    //     Some(args) => args,
    //     None => return Ok(()),
    // };

    // println!("Domain:{}", domain);
    // println!("Local Host:{}", lhost);
    // println!("Local Port:{}", lport);
    let addr = SocketAddr::from(([127, 0, 0, 1], 80));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();

    //     handle_connection(
    //         stream,
    //         domain.as_ref().unwrap(),
    //         lhost.as_ref().unwrap(),
    //         lport.unwrap(),
    //     );
    // }

    loop {
        let (stream, _) = listener.accept().await?;
        println!("Accepted connection, {:?}", stream);
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(proxy))
                .await
            {
                eprintln!("Error serving connection:{}", err);
            }
        });
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
    }
}
// fn handle_connection(mut stream: TcpStream, domain: &str, lhost: &str, lport: u16) {
//     // Reading the data in the packet
//     let buf_reader = BufReader::new(&mut stream);
//     let http_request: Vec<_> = buf_reader
//         .lines()
//         .map(|result| result.unwrap())
//         .take_while(|line| !line.is_empty())
//         .collect();

//     println!("Request: {:#?}", http_request);

//     // Extracting Host Header from the request
//     let mut host_value = None;
//     for line in &http_request {
//         if line.starts_with("Host:") {
//             host_value = Some(line.trim_start_matches("Host: ").trim().to_string());
//             break;
//         }
//     }

//     match host_value {
//         Some(value) => {
//             println!("Host value: {}", value);
//             if value == domain {
//                 println!("Host value matches domain: {}", domain);

//                 // Sending the http_request to another http server specified by lhost and lport
//                 let destination = format!("{}:{}", lhost, lport);
//                 let mut destination_stream = TcpStream::connect(destination).unwrap();
//                 for line in &http_request {
//                     destination_stream.write_all(line.as_bytes()).unwrap();
//                     destination_stream.write_all(b"\r\n").unwrap();
//                 }
//                 destination_stream.write_all(b"\r\n").unwrap();

//                 // Forwarding the response from the destination server back to the client
//                 let mut response = String::new();
//                 let mut response_reader = BufReader::new(&mut destination_stream);
//                 response_reader.read_to_string(&mut response).unwrap();

//                 // Sending the response back to the client
//                 stream.write_all(response.as_bytes()).unwrap();
//             } else {
//                 println!("Host value does not match domain: {}", domain);
//             }
//         }
//         None => println!("Host header not found"),
//     }
// }
