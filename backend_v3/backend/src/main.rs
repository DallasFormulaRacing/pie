use futures_util::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsFrame;

mod can;
mod websocket;
use websocket::handle_ws_message;

const SERVER_ADDR: &str = "127.0.0.1:9002";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;
    println!("pi websocket server listening on ws://{SERVER_ADDR}");
    println!(
        r#"try sending: {{"system":"daq","message":{{"frame":"wheelSpeed","source":"nodefl","rpm":42.0}}}}"#
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("client connected: {addr}");

        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, addr).await {
                eprintln!("client {addr} error: {error}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let ws = accept_async(stream).await?;
    let (_ws_tx, mut ws_rx) = ws.split();

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(WsFrame::Text(text)) => {
                    //println!("device -> pi raw: {text}");

                    match websocket::Message::decode_json(&text) {
                        Ok(ws_message) => {
                            /*
                            println!(
                                "device -> pi deserialized:\n{}",
                                ws_message.to_pretty_json()
                            );
                            */
                            handle_ws_message(&ws_message);
                        }
                        Err(_) => println!("device -> pi did not match Message"),
                    }
                }
                Ok(WsFrame::Close(_)) => break,
                Ok(_) => {}
                Err(error) => {
                    eprintln!("client {addr} rx error: {error}");
                    break;
                }
            }
        }
    });

    let _ = rx_thread.await;

    println!("client disconnected: {addr}");
    Ok(())
}
