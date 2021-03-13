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
//! Simple `no_std` spectrum analysis library that follows the KISS (keep it simple, stupid)
//! principle. The main goal of this crate is to be educational to the world and myself. This
//! is not a bullet-proof or ideal solution! Feel free to contribute and point out possible
//! errors/bugs/wrong assumptions or improvements!

#![no_std]

// use alloc crate, because this is no_std
// #[macro_use]
extern crate alloc;

// use std in tests
#[cfg(test)]
#[macro_use]
extern crate std;

use alloc::collections::BTreeMap;
use rustfft::algorithm::Radix4;
use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftDirection};
use core::f32::consts::PI;
use alloc::vec::Vec;

/// A map from frequency (in Hertz) to the magnitude.
/// The magnitude is dependent on whether you scaled
/// the values, e.g to logarithmic scale.
pub type FrequencySpectrumMap = BTreeMap<usize, f32>;

/// Takes an array of samples (length must be a power of 2),
/// e.g. 2048, applies an FFT (using library `rustfft`) on it
/// and returns all frequencies with their volume/magnitude.
///
/// * `samples` raw audio, e.g. 16bit audio data but as f32.
///             You should apply an window function (like hann) on the data first.
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
/// * `scaling_fn` Optional scaling function. For example transform all values to dB/logarithmic scale:
///               (`|s| 20_f32 * s.log10()`).
/// * `max_frequency` Optional. If you are interested in a maximum frequency in the final
///                   frequency spectrum, say 150Hz, this accelerates the calculation.
///
/// ## Returns value
/// Map from frequency to magnitude, see [`FrequencySpectrumMap`]
pub fn samples_fft_to_spectrum(
    samples: &[f32],
    sampling_rate: u32,
    scaling_fn: Option<&dyn Fn(f32) -> f32>,
    max_frequency: Option<f32>,
) -> BTreeMap<usize, f32> {
    // With FFT we transform an array of time-domain waveform samples
    // into an array of frequency-domain spectrum samples
    // https://www.youtube.com/watch?v=z7X6jgFnB6Y

    // FFT result has same length as input

    // convert to Complex for FFT
    let mut buffer = samples_to_complex(samples);

    // a power of 2, like 1024 or 2048
    let fft_len = samples.len();

    // apply the fft
    let fft = Radix4::new(fft_len, FftDirection::Forward);
    fft.process(&mut buffer);

    // we only need the first half of the results with FFT
    // because of Nyquist theorem. 44100hz sampling frequency
    // => 22050hz maximum detectable frequency

    let magnitudes = fft_result_to_magnitudes(buffer, fft_len, scaling_fn);

    // calc frequency spectrum: map from Frequency to magnitude
    magnitudes_to_frequency_spectrum(magnitudes, fft_len, sampling_rate, max_frequency)
}

/// Applies a Hann window (https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
pub fn hann_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    for i in 0..samples.len() {
        let two_pi_i = 2_f32 * PI * i as f32;
        let idontknowthename = (two_pi_i / samples.len() as f32).cos();
        let multiplier = 0.5 * (1.0 - idontknowthename);
        windowed_samples.push(multiplier * samples[i])
    }
    windowed_samples
}

/// Applies a Hamming window (https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
pub fn hamming_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    for i in 0..samples.len() {
        let multiplier = 0.54 - (0.46 * (2_f32 * PI * i as f32 / (samples.len() - 1) as f32).cos());
        windowed_samples.push(multiplier * samples[i])
    }
    windowed_samples
}

/// Converts all samples to a complex number (imaginary part is set to two)
/// as preparation for the FFT.
///
/// ## Return value
/// New vector of samples but as Complex data type.
fn samples_to_complex(samples: &[f32]) -> Vec<Complex32> {
    samples
        .iter()
        .map(|x| Complex32::new(x.clone(), 0.0))
        .collect::<Vec<Complex32>>()
}

/// Transforms the complex numbers of the first half of the FFT results (only the first
/// half is relevant, Nyquist theorem) to their magnitudes.
///
/// ## Parameters
/// * `fft_result` Result buffer from FFT.
/// * `fft_len` FFT length. A power of 2 or `2* magnitudes.len()`
/// * `scaling_fn` optional scaling function. For example transform all values to dB/logarithmic scale:
///               (`|s| 20_f32 * s.log10()`).
/// ## Return value
/// New vector of all magnitudes. The indices correspond to the indices in the FFT result (first half).
/// The resulting vector has half the length of the FFT result.
fn fft_result_to_magnitudes(
    fft_result: Vec<Complex32>,
    fft_len: usize,
    scaling_fn: Option<&dyn Fn(f32) -> f32>,
) -> Vec<f32> {
    let identity_fn = |x| x;

    fft_result
        .into_iter()
        // take first half; half of input length
        .take(fft_len / 2)
        // START: calc magnitude: sqrt(re*re + im*im) (re: real part, im: imaginary part)
        .map(|c| c.norm())
        // END: calc magnitude
        // optionally scale
        .map(|s| scaling_fn.unwrap_or(&identity_fn)(s))
        .collect::<Vec<f32>>()
}

/// Calculates the frequency spectrum from the magnitudes of an FFT. Usually you will
/// call this with the result of [`fft_result_to_magnitudes`].
///
/// ## Parameters
/// * `magnitudes` All magnitudes. If you did the FFT with 2048 samples, this vector will be 1024
///                magnitudes long.
/// * `fft_len` FFT length. A power of 2 or `2* magnitudes.len()`
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
/// * `max_frequency` Optional. If you are interested in a maximum frequency, say 150Hz, this
///                   accelerates the calculation.
/// ## Return value
/// Map from frequency to magnitude. Contains either `magnitudes.len()` entries if `max_frequency`
/// is None, or else maybe less.
fn magnitudes_to_frequency_spectrum(
    magnitudes: Vec<f32>,
    fft_len: usize,
    sampling_rate: u32,
    max_frequency: Option<f32>,
) -> FrequencySpectrumMap {
    let mut frequency_to_mag_map = BTreeMap::new();
    for (i, vol) in magnitudes.into_iter().enumerate() {
        // where this line comes from is explained here:
        // https://stackoverflow.com/questions/4364823/
        let frequency = i as f32 / fft_len as f32 * sampling_rate as f32;
        frequency_to_mag_map.insert(frequency as usize, vol);

        // speed up execution; only calc the frequencies we want
        if let Some(max) = max_frequency {
            if frequency > max {
                break;
            }
        }
    }
    frequency_to_mag_map
}

#[cfg(test)]
mod tests;
