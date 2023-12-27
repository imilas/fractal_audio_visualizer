use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use main::cpal_enumerate;
use main::fft;
use ringbuf::HeapRb;

const FFT_LEN: usize = 2048;

use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const FRACTAL_DEPTH: u32 = 16;
const GENERATION_INFINITY: f64 = 8.;
const MAX_EXPONENT: f64 = 7.;

fn main() -> anyhow::Result<()> {
    let _ = cpal_enumerate::get_devices();
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
    let mut low: f64 = 0.; // low freqs
    let mut high: f64 = 0.; // high freqs
    let mut ratio = 1.;
    let eps = 0.1; // rate of change
    let range = 2.0; //z coordinate range
    let x_min = 0. - range;
    let y_min = 0. - range;
    let x_max = 0. + range;
    let y_max = 0. + range;
    let mut angle: f64 = 1.57;
    let mut exponent: f64 = 2.; // julia set exponent
    window.set_background_color(0, 0, 20);

    //
    while window.is_open() && !window.is_key_down(Key::Q) {
        for (i, pixel) in buffer.iter_mut().enumerate() {
            let mut real = map((i % WIDTH) as f64, 0., WIDTH as f64, x_min, x_max);
            let mut imag = map((i / HEIGHT) as f64, 0., HEIGHT as f64, y_min, y_max);

            let mut n = 0;

            let mut z = num::complex::Complex::new(real, imag);

            while n < FRACTAL_DEPTH {
                z = z.powf(exponent);
                real = z.re + angle.cos();
                imag = z.im + angle.sin();

                z = num::complex::Complex::new(real, imag);
                if (real + imag).abs() > GENERATION_INFINITY {
                    break;
                }

                n += 1;
            }

            *pixel = fill(n, low, high);
        }

        //when buffer full, process frequencies
        if consumer.len() > FFT_LEN {
            let mut angle_update: f64;
            let v: Vec<f32> = consumer.pop_iter().collect();
            // let amp = f64::from(v.iter().map(|&x| x.abs()).sum::<f32>() / v.len() as f32);
            let buff_fft = fft::fft(&v, FFT_LEN);
            // number of fft bins is around half the fft_length
            // also, the fft::fft function can limit the range of requencies
            let (_, buff_mel) = fft::spec_to_mels(&buff_fft);
            let num_bins = buff_mel.len();
            // let normalizer = num_bins as f64;
            let low_cut = num_bins / 3 as usize;
            //update angle based on frequency content
            low = buff_mel[0..low_cut].iter().sum::<f32>() as f64 / low_cut as f64;
            high = 3. * buff_mel[low_cut..num_bins].iter().sum::<f32>() as f64
                / (num_bins - low_cut) as f64;

            if high > low {
                angle_update = f64::min(0.1, 7. * high);
            } else {
                angle_update = -1. * f64::min(0.1, 7. * low);
            }
            ratio = (1. - eps) * ratio + eps * ((low + 0.001) / (0.0001 + high));

            if exponent <= -1. {
                angle_update = angle_update * 0.5; // negative exponents more sensetive
            }
            angle += angle_update;
        }

        if window.is_key_pressed(Key::K, KeyRepeat::No) {
            exponent = exponent.floor() % MAX_EXPONENT + 1.;
            println!("New Exponent: {exponent}");
        }
        if window.is_key_pressed(Key::J, KeyRepeat::No) {
            exponent = exponent % MAX_EXPONENT - 1.;
            println!("New Exponent: {exponent}");
        }

        // We unwrap here as we want this code to exit if it fails
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
    Ok(())
}

fn map(val: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
    start2 + (stop2 - start2) * ((val - start1) / (stop1 - start1))
}

fn fill(n: u32, low: f64, high: f64) -> u32 {
    //fills pixel based on fractal depth and frequency content
    if FRACTAL_DEPTH == n {
        ((high * 20. * 255.) as u32) << 16 //red at out of bound set
    } else if n % 7 == 0 {
        255 + ((n * 100 % 255) << 8) + (((low * 20. * 255.) as u32) << 16) // every 7th layer gets some red and green
    } else if n > FRACTAL_DEPTH - 10 {
        ((high * 20. % 255.) as u32) << 16 // end layers get red
    } else {
        200 + ((n * 100 % 255) << 8)
    }
}
fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
