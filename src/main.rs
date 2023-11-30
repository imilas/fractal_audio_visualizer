use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use main::fft;
use ringbuf::HeapRb;

const FFT_LEN: usize = 2048;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;
const FRACTAL_DEPTH: u32 = 32;
const GENERATION_INFINITY: f64 = 8.;

fn main() -> anyhow::Result<()> {
    // cpal setup
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("no output device available");

    let fft_len = FFT_LEN;

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(fft_len * 2);
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
            resize: true,
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to Open Window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let range = 2.0;
    let x_min = 0. - range;
    let y_min = 0. - range;

    let x_max = 0. + range;
    let y_max = 0. + range;

    let mut angle: f64 = 1.0;

    window.set_background_color(0, 0, 20);

    while window.is_open() && !window.is_key_down(Key::Q) {
        for (i, pixel) in buffer.iter_mut().enumerate() {
            let mut real = map((i % WIDTH) as f64, 0., WIDTH as f64, x_min, x_max);
            let mut imag = map((i / HEIGHT) as f64, 0., HEIGHT as f64, y_min, y_max);

            let mut n = 0;

            while n < FRACTAL_DEPTH {
                let re = real.powf(2.) - imag.powf(2.);
                let im = 2. * real * imag;

                real = re + angle.cos();
                imag = im + angle.sin();

                if (real + imag).abs() > GENERATION_INFINITY {
                    break; // Leave when achieve infinity
                }
                n += 1;
            }

            *pixel = fill(n);
        }

        if consumer.len() > FFT_LEN {
            let v: Vec<f32> = consumer.pop_iter().collect();
            // let amp = f64::from(v.iter().map(|&x| x.abs()).sum::<f32>() / v.len() as f32);
            let buff_fft = fft::fft(&v, FFT_LEN);
            // number of fft bins is around half the fft_length
            // also, the fft::fft function can limit the range of requencies
            let (_, buff_mel) = fft::spec_to_mels(&buff_fft);
            let num_bins = buff_mel.len();
            let low: f64 =
                buff_mel[0..num_bins / 10 as usize].iter().sum::<f32>() as f64 / num_bins as f64;
            let med: f64 = buff_mel[num_bins / 10..num_bins / 3 as usize]
                .iter()
                .sum::<f32>() as f64
                / num_bins as f64;
            let high: f64 =
                buff_mel[num_bins / 3..num_bins].iter().sum::<f32>() as f64 / num_bins as f64;

            angle += high * 10.0;
            angle -= (low + med) * 13.0;
        }

        // We unwrap here as we want this code to exit if it fails
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
    Ok(())
}

fn map(val: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
    start2 + (stop2 - start2) * ((val - start1) / (stop1 - start1))
}

fn fill(n: u32) -> u32 {
    if FRACTAL_DEPTH == n {
        0x00
    } else {
        // can bit shift here by 8 to make red and green
        n * 32 % 255
    }
}
fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
