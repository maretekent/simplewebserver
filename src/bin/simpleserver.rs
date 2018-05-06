extern crate simpleserver;
use simpleserver::ThreadPool;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::sync::{mpsc};

enum ServerMessage {
    ShutDown
}

fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<ServerMessage>) {
    let asset_dir = "pages/";
    let mut buf = [0; 512];
    let mut contents = String::new();

    if let Err(_) = stream.read(&mut buf) { println!("Failed to read stream."); }

    let indexpage = b"GET / HTTP/1.1";
    let somepage = b"GET /somepage HTTP/1.1";
    let shutdown_page = b"GET /shutdown HTTP/1.1";

    let (status, file_name) = if buf.starts_with(indexpage) {
        ("200 OK", "index.html")
    } else if buf.starts_with(somepage) {
        thread::sleep(Duration::from_secs(5));
        ("200 OK", "somepage.html")
    } else if buf.starts_with(shutdown_page) {
        if let Err(_) = tx.send(ServerMessage::ShutDown) {
            println!("Failed to send shutdown message.");
        }
        ("200 OK", "shutdown.html")
    } else {
        ("404 NOT FOUND", "404.html")
    };

    if let Ok(mut file) = File::open(format!("{}{}", asset_dir, file_name)) {
        if let Err(_) = file.read_to_string(&mut contents) {
            println!("Failed to read file");
        }
    }

    let response = format!("HTTP/1.1 {}\r\n\r\n{}", status, contents);

    if let Err(_) = stream.write(response.as_bytes()) {
        println!("Failed to write response to stream: {}", response);
    }
    if let Err(_) = stream.flush() {
        println!("Failed to flush stream.");
    }
}

fn main() {
    let address = "0.0.0.0:8005";
    let pool = ThreadPool::new(10);
    let (tx, rx) = mpsc::channel();

    if let Ok(listener) = TcpListener::bind(address) {
        println!("Server started on http://{}", address);

        for tcp_result in listener.incoming() {
            if let Ok(ServerMessage::ShutDown) = rx.try_recv() {
                println!("Shutting Down Server.");
                return;
            }

            if let Ok(stream) = tcp_result {
                let tx = tx.clone();
                pool.execute(|| {
                    handle_client(stream, tx);
                });
            }
            else {
                println!("Failed to get TcpStream");
            }
        }
    }
    else {
        println!("Failed to bind to address: {}", address);
    }
}
