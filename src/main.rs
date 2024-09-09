use futures_util::StreamExt;

const BUFFER_SECONDS: usize = 6;
const AVERAGE_BITRATE: usize = 16000;
const APPROX_AUDIO_BUFFER_SIZE: usize = AVERAGE_BITRATE * BUFFER_SECONDS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream_url = "";
    stream_radio(stream_url).await?;
    Ok(())
}

fn play_buffer_mp3(buffer: &mut Vec<u8>, sink: &rodio::Sink) {
    // decode the mp3 audio stream with minimp3
    let mut decoder = minimp3::Decoder::new(std::io::Cursor::new(buffer.clone()));
    while let Ok(minimp3::Frame {
        data,
        sample_rate,
        channels,
        ..
    }) = decoder.next_frame()
    {
        // play the buffer with rodio
        let source = rodio::buffer::SamplesBuffer::new(channels as u16, sample_rate as u32, data);
        sink.append(source);
    }
    buffer.clear();
}

async fn stream_radio(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // send an asynchronous request to the radio stream
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .or(Err(format!("failed GET from '{}'", &url)))?;
    let mut stream = response.bytes_stream();

    // set up audio output
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    // this is a buffer to collect streaming chunks of audio
    let mut buffer: Vec<u8> = Vec::new();

    'outer: loop {
        // Fill the buffer with streaming data
        while buffer.len() < APPROX_AUDIO_BUFFER_SIZE {
            match stream.next().await {
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

        // Play the buffer once it's full or the stream ends
        if !buffer.is_empty() {
            play_buffer_mp3(&mut buffer, &sink);
        }
    }

    sink.sleep_until_end(); // keep playing until stream ends
    Ok(())
}
