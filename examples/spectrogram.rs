use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use main::cpal_enumerate;
use main::fft;
use ringbuf::HeapRb;

const FFT_LEN: usize = 2048;

use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 927;
const HEIGHT: usize = 400;

fn main() -> anyhow::Result<()> {
    let _ = cpal_enumerate::get_devices();
    // cpal setup
    let host = cpal::default_host();

    let input_device = host
        .default_input_device()
        .expect("no output device available");
    let fft_len = FFT_LEN;

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(fft_len);
    let (mut producer, mut consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..fft_len {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            //push sample to the consumer
            if producer.push(sample).is_err() {
                // println!("heap filled, before fft applied");
            }
        }
    };

    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    input_stream.play()?;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Fractal - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            scale_mode: ScaleMode::Stretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to Open Window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.set_background_color(0, 0, 20);
    let mut j = 0;
    while window.is_open() && !window.is_key_down(Key::Q) {
        //when buffer full, process frequencies
        if consumer.len() >= FFT_LEN {
            let mut angle_update: f64;
            let v: Vec<f32> = consumer.pop_iter().collect();
            // let amp = f64::from(v.iter().map(|&x| x.abs()).sum::<f32>() / v.len() as f32);
            let buff_fft = fft::fft(&v, FFT_LEN);
            // number of fft bins is around half the fft_length
            // also, the fft::fft function can limit the range of requencies
            let (_, buff_mel) = fft::spec_to_mels(&buff_fft);
            let num_bins = buff_mel.len();
            // convert fft values to pixel
            let pixels: Vec<u32> = buff_mel.iter().map(|x| (x * 10000.) as u32).collect();

            println!("{},{:?},{}", FFT_LEN, num_bins, j);
            // 2 things happen here, the framebuffer gets shifted by fft_length and
            // the 0..fft_length vectors get replaced by the calculated fft values

            buffer[..num_bins].clone_from_slice(&pixels);

            buffer.rotate_right(WIDTH);
            j += 1;
        }

        // We unwrap here as we want this code to exit if it fails
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
    Ok(())
}

fn map(val: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
    start2 + (stop2 - start2) * ((val - start1) / (stop1 - start1))
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
