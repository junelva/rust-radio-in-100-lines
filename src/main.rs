use std::{error::Error, io::Cursor};

use futures_util::StreamExt;
use reqwest::Client;
use rodio::{Decoder, OutputStream, Sink};

const BUFFER_SECONDS: usize = 6;
const AVERAGE_BITRATE: usize = 16000;
const APPROX_AUDIO_BUFFER_SIZE: usize = AVERAGE_BITRATE * BUFFER_SECONDS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream_url = "";
    stream_radio(stream_url).await?;
    Ok(())
}

async fn stream_radio(url: &str) -> Result<(), Box<dyn Error>> {
    // send an asynchronous request to the radio stream
    let response = Client::new()
        .get(url)
        .send()
        .await
        .or(Err(format!("failed GET from '{}'", &url)))?;
    let mut audio_stream = response.bytes_stream();

    // set up audio output
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // this is a buffer to collect streaming chunks of audio
    let mut buffer: Vec<u8> = Vec::new();

    'outer: loop {
        // fill the buffer with streaming data
        while buffer.len() < APPROX_AUDIO_BUFFER_SIZE {
            match audio_stream.next().await {
                Some(Ok(chunk)) => {
                    buffer.extend_from_slice(&chunk);
                }
                Some(Err(e)) => {
                    eprintln!("Error reading chunk: {}", e);
                    break;
                }
                None => {
                    if buffer.is_empty() {
                        break 'outer;
                    } else {
                        break;
                    }
                }
            }
        }

        if !buffer.is_empty() {
            sink.append(Decoder::new(Cursor::new(buffer.clone()))?);
            buffer.clear();
        }
    }

    sink.stop();
    Ok(())
}
