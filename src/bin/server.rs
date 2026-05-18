use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            message = ws_stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        if let Some(text) = message.as_text() {
                            println!("{addr}: {text}");
                            bcast_tx.send(text.to_string())?;
                        }
                    }
                    Some(Err(error)) => {
                        println!("Error from {addr}: {error}");
                        break;
                    }
                    None => {
                        println!("Connection closed from {addr}");
                        break;
                    }
                }
            }

            message = bcast_rx.recv() => {
                let message = message?;
                ws_stream.send(Message::text(message)).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");

        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}