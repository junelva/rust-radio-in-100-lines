use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Client;
use rodio::{Decoder, OutputStream, Sink};
use std::thread::sleep;
use std::time::Duration;
use std::{error::Error, io::Cursor, sync::Arc};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream_url = "https://stream.epic-piano.com/chillout-piano?ref=radiobrowser";
    stream_radio(stream_url).await?;
    Ok(())
}

async fn play_buffer(
    sink: &Sink,
    buffer: &mut Vec<Bytes>,
) -> Result<(), rodio::decoder::DecoderError> {
    let b = {
        let mut b = Vec::new();
        let head_len = buffer.len() - ((buffer.len()) % 4);
        println!("buf len: {}, head len: {}", buffer.len(), head_len);
        let head: Vec<Bytes> = buffer.drain(0..head_len).collect();
        for byte in head.iter() {
            b = [b, byte.to_vec()].concat();
        }
        b
    };
    let cursor = Cursor::new(b);
    let source = Decoder::new(cursor)?;
    sink.append(source);
    // }

    Ok(())
}

enum DownloadResult {
    Success,
    Empty,
}

async fn download_ok(
    sent: Option<Result<Bytes, reqwest::Error>>,
    tx: &mut Sender<Bytes>,
) -> Result<DownloadResult, Box<dyn Error>> {
    match sent {
        Some(Ok(chunk)) => {
            tx.send(chunk).await?;
            return Ok(DownloadResult::Success);
        }
        Some(Err(e)) => {
            eprintln!("error reading chunk: {}", e);
            return Err(Box::new(e));
        }
        None => (),
    }
    Ok(DownloadResult::Empty)
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
    let (_output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&output_stream_handle).unwrap();
    let sink_mtx = Arc::new(Mutex::new(sink));
    let sink_mtx_decode = sink_mtx.clone();

    // set up thread communication
    let (mut tx, rx) = mpsc::channel::<Bytes>(4096);
    let rx_mtx = Mutex::new(rx);

    // buffer is only touched in the decoding and playback thread
    let buffer = Arc::new(Mutex::new(Vec::<Bytes>::new()));

    // downloading thread (sender)
    let download_handle = tokio::spawn(async move {
        loop {
            let st = audio_stream.next().await;
            match download_ok(st, &mut tx).await {
                Ok(DownloadResult::Success) => {
                    // println!("downloaded something");
                }
                Ok(DownloadResult::Empty) => {
                    println!("download was empty");
                }
                Err(e) => {
                    eprintln!("error reading chunk: {}", e);
                }
            }
            sleep(Duration::from_nanos(250_000));
        }
    });

    // decoding and playback thread (receiver)
    let decode_handle = tokio::spawn(async move {
        loop {
            let mut buffer = buffer.lock().await;
            {
                let mut rx = rx_mtx.lock().await;
                let recv = rx.recv().await;
                if let Some(recv) = recv {
                    buffer.push(recv);
                }
            }

            if buffer.len() >= 16 {
                let sink = sink_mtx_decode.lock().await;
                match play_buffer(&sink, &mut buffer).await {
                    Ok(_) => (),
                    Err(e) => eprintln!("error playing: {}", e),
                };
            } else if buffer.is_empty() {
                println!("empty buffer; decoder is sleeping...");
                sleep(Duration::from_millis(1));
            }
        }
    });

    let _ = tokio::join!(download_handle, decode_handle);
    let sink = sink_mtx.lock().await;
    sink.stop();
    Ok(())
}
