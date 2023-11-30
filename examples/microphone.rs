use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use main::fft;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::sync::mpsc;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("no output device available");
    let (tx, rx) = mpsc::channel();

    let mut buffer: AllocRingBuffer<f32> = AllocRingBuffer::with_capacity(10000);

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            // buffer.push(sample);
            tx.send(sample).unwrap();
        }
    };

    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    input_stream.play()?;

    let fft_len = 2048;
    for _ in 0..fft_len {
        buffer.push(0.0);
    }

    let mut i = 0;
    for received in rx {
        i += 1;
        if i % 3000 == 0 {
            print!("Got: {},{i}", received);
            print!("\r");
            buffer.push(received);
            let buff_fft = fft::fft(&buffer.to_vec(), fft_len);
            let buff_mel = fft::spec_to_mels(&buff_fft);
        }
    }

    drop(input_stream);
    Ok(())
}
fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
