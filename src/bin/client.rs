use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
            .connect()
            .await?;

    println!("Connected to ws://127.0.0.1:8080");

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        ws_stream.send(Message::text(line)).await?;
                    }
                    Ok(None) => break,
                    Err(error) => {
                        println!("Error reading stdin: {error}");
                        break;
                    }
                }
            }

            message = ws_stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        if let Some(text) = message.as_text() {
                            println!("{text}");
                        }
                    }
                    Some(Err(error)) => return Err(error),
                    None => break,
                }
            }
        }
    }

    Ok(())
}