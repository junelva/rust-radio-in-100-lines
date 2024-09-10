use futures_util::StreamExt;
use reqwest::Client;
use ringbuf::{traits::*, SharedRb};
use rodio::{Decoder, OutputStream, Sink};
use std::{error::Error, io::Cursor, sync::Arc};
use tokio::sync::{Mutex, Notify};

const BUFFER_SIZE: usize = 1024 * 512;
const BUFFER_ALMOST_FULL: usize = BUFFER_SIZE - BUFFER_SIZE / 4;
const RUNTIME_READ_SIZE: usize = BUFFER_SIZE / 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream_url = "http://jking.cdnstream1.com/b22139_128mp3";
    let response = Client::new()
        .get(stream_url)
        .send()
        .await
        .or(Err(format!("failed GET from '{}'", &stream_url)))?;
    let mut audio_stream = response.bytes_stream();

    let (_output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&output_stream_handle).unwrap();
    let sink_mtx = Arc::new(Mutex::new(sink));
    let sink_mtx_decode = sink_mtx.clone();

    let rb = SharedRb::new(BUFFER_SIZE);
    let (mut writer, mut reader) = rb.split();
    let write_notify = Arc::new(Notify::new());
    let read_notify = write_notify.clone();
    let read_size = Arc::new(Mutex::new(BUFFER_ALMOST_FULL));
    let mut initializing_playback = true;

    let download_handle = tokio::spawn(async move {
        println!("buffering...");
        while let Some(Ok(chunk)) = audio_stream.next().await {
            for byte in chunk {
                loop {
                    if writer.occupied_len() >= BUFFER_ALMOST_FULL {
                        println!("writer is almost full, waking up decoder");
                        initializing_playback = false;
                        write_notify.notify_one();
                        write_notify.notified().await;
                        continue;
                    }
                    match writer.try_push(byte) {
                        Ok(_) => break,
                        Err(e) => eprintln!("error writing byte: {}", e),
                    };
                }
            }
            if !initializing_playback && writer.occupied_len() >= RUNTIME_READ_SIZE {
                write_notify.notify_one();
                write_notify.notified().await;
            }
        }
    });

    let decode_handle = tokio::spawn(async move {
        loop {
            read_notify.notified().await;
            print!("reading... ");
            let out_buffer = {
                let read_size = read_size.lock().await;
                let mut out_buffer: Vec<u8> = Vec::with_capacity(*read_size);
                while out_buffer.len() < *read_size {
                    match reader.try_pop() {
                        Some(value) => out_buffer.push(value),
                        None => continue,
                    }
                }
                out_buffer
            };
            print!("decoding 32 x {}; ", out_buffer.len() as f32 / 4.0);
            let sink_mtx_decode = sink_mtx_decode.clone();
            let cursor = Cursor::new(out_buffer);
            let decoder = Decoder::new_mp3(cursor);
            match decoder {
                Ok(source) => {
                    println!("appending to output sink");
                    let sink = sink_mtx_decode.lock().await;
                    if sink.empty() {
                        println!("sink was empty; filling it up");
                        let mut read_size = read_size.lock().await;
                        *read_size = RUNTIME_READ_SIZE;
                    }
                    sink.append(source)
                }
                Err(e) => eprintln!("error playing: {}", e),
            }
            read_notify.notify_one();
        }
    });

    decode_handle.await?;
    download_handle.await?;
    let sink = sink_mtx.lock().await;
    sink.stop();
    Ok(())
}
