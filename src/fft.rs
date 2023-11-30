use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::FrequencySpectrum;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

pub fn fft(samples: &Vec<f32>, size: usize) -> spectrum_analyzer::FrequencySpectrum {
    let hann_window = hann_window(&samples[0..size]);
    // calc spectrum
    let spectrum_hann_window = samples_fft_to_spectrum(
        // (windowed) samples
        &hann_window,
        // sampling rate
        44100,
        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
        FrequencyLimit::Range(30.0, 10000.0),
        // FrequencyLimit::All,
        // optional scale
        Some(&divide_by_N_sqrt),
    )
    .unwrap();
    return spectrum_hann_window;
}
pub fn spec_to_mels(spec: &FrequencySpectrum) -> (Vec<f32>, Vec<f32>) {
    // takes in a spectrogram and return 2 vectors (mels,amplitudes)
    let v1: Vec<f32> = spec.to_mel_map().iter().map(|e| *e.0 as f32).collect();
    let v2: Vec<f32> = spec.to_mel_map().iter().map(|e| *e.1).collect();
    return (v1, v2);
}
