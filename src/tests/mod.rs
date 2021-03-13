/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use crate::tests::sine::{sine_wave_audio_data, sine_wave_audio_data_multiple};
use alloc::vec::Vec;
use crate::{hann_window, samples_fft_to_spectrum};
// use std::prelude::*;

mod sine;

#[test]
fn test_output_frequency_spectrum_sine_50hz() {
    let sine_audio = sine_wave_audio_data(
        // 1000Hz in 100ms => sin wave will have 100 time periods
        50.0,
        44100,
        1000
    );

    let sine_audio = sine_audio.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // FFT frequency accuracy is: sample_rate / (N / 2)
    // 44100/(4096/2) = 21.53Hz

    // get a window that we want to analyze
    // 1/44100 * 4096 => 0.093s
    let window = &sine_audio[0..1024];

    let hann_window = hann_window(window);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );
    for (fr, vol) in spectrum.iter() {
        // you will experience inaccuracies here
        // TODO add further smoothing / noise reduction
        if *vol > 120.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }
}

#[test]
fn test_output_frequency_spectrum_sine_1000hz() {
    let sine_audio = sine_wave_audio_data(
        // 1000Hz in 100ms => sin wave will have 100 time periods
        1000.0,
        44100,
        1000
    );

    let sine_audio = sine_audio.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // FFT frequency accuracy is: sample_rate / (N / 2)
    // 44100/(16384/2) = 5.383Hz

    // get a window that we want to analyze
    // 1/44100 * 16384 => 0.3715
    let window = &sine_audio[0..16384];

    let hann_window = hann_window(window);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );
    for (fr, vol) in spectrum.iter() {
        // you will experience inaccuracies here
        // TODO add further smoothing / noise reduction
        if *vol > 130.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }
}

#[test]
fn test_output_frequency_spectrum_sine_50_plus_1000_plus_3777hz() {
    let sine_audio = sine_wave_audio_data_multiple(
        // 1000Hz in 100ms => sin wave will have 100 time periods
        &[50.0, 1000.0, 3777.0],
        44100,
        1000
    );

    let sine_audio = sine_audio.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // FFT frequency accuracy is: sample_rate / (N / 2)
    // 44100/(16384/2) = 5.383Hz

    // get a window that we want to analyze
    // 1/44100 * 16384 => 0.3715
    let window = &sine_audio[0..16384];

    let hann_window = hann_window(window);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );
    for (fr, vol) in spectrum.iter() {
        // you will experience inaccuracies here
        // TODO add further smoothing / noise reduction
        if *vol > 130.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }
}

#[test]
fn test_spectrum_mp3_sample_bass_drum() {
    let sine_audio = sine_wave_audio_data_multiple(
        // 1000Hz in 100ms => sin wave will have 100 time periods
        &[50.0, 1000.0, 3777.0],
        44100,
        1000
    );

    let sine_audio = sine_audio.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // FFT frequency accuracy is: sample_rate / (N / 2)
    // 44100/(16384/2) = 5.383Hz

    // get a window that we want to analyze
    // 1/44100 * 16384 => 0.3715
    let window = &sine_audio[0..16384];

    let hann_window = hann_window(window);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );
    for (fr, vol) in spectrum.iter() {
        // you will experience inaccuracies here
        // TODO add further smoothing / noise reduction
        if *vol > 130.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }
}
