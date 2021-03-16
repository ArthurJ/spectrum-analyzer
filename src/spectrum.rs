//! Module for the struct [`FrequencySpectrum`].

use alloc::vec::Vec;
use crate::frequency::{Frequency, FrequencyValue};
use alloc::collections::BTreeMap;
use core::cell::{RefCell, Cell, Ref};
use alloc::boxed::Box;

/// Describes the type for a function factory that generates a function that can scale/normalize
/// the data inside [`FrequencySpectrum`].
/// This can be used to subtract `min` value from all values for example , if `min`
/// is `> 0`. The signature is the following:
/// `(min: f32, max: f32, average: f32, median: f32) -> fn(f32) -> f32`
/// i.e. you provide a function which generates a function that gets
/// applied to each element.
pub type SpectrumTotalScaleFunctionFactory = Box<dyn Fn(f32, f32, f32, f32) -> Box<dyn Fn(f32) -> f32>>;

#[derive(Debug)]
pub struct FrequencySpectrum {
    /// Raw data. Vector is sorted from lowest
    /// frequency to highest.
    data: RefCell<Vec<(Frequency, FrequencyValue)>>,
    /// Average value of frequency value/magnitude/amplitude.
    average: Cell<FrequencyValue>,
    /// Median value of frequency value/magnitude/amplitude.
    median: Cell<FrequencyValue>,
    /// Minimum value of frequency value/magnitude/amplitude.
    min: Cell<FrequencyValue>,
    /// Maximum value of frequency value/magnitude/amplitude.
    max: Cell<FrequencyValue>,
}

impl FrequencySpectrum {

    /// Creates a new object. Calculates several metrics on top of
    /// the passed vector.
    #[inline(always)]
    pub fn new(data: Vec<(Frequency, FrequencyValue)>) -> Self {
        let obj = Self {
            data: RefCell::new(data),
            average: Cell::new(FrequencyValue::from(-1.0)),
            median: Cell::new(FrequencyValue::from(-1.0)),
            min: Cell::new(FrequencyValue::from(-1.0)),
            max: Cell::new(FrequencyValue::from(-1.0)),
        };
        // IMPORTANT!!
        obj.calc_statistics();
        obj
    }

    /// Applies the function generated by `total_scaling_fn` to each element and updates
    /// `min`, `max`, etc. afterwards accordingly.
    ///
    /// ## Parameters
    /// * `total_scaling_fn` See [`crate::spectrum::SpectrumTotalScaleFunctionFactory`].
    #[inline(always)]
    pub fn apply_total_scaling_fn(&self, total_scaling_fn: SpectrumTotalScaleFunctionFactory) {
        let scale_fn = (total_scaling_fn)(
            // into() => FrequencyValue => f32
            self.min.get().val(),
            self.max.get().val(),
            self.average.get().val(),
            self.median.get().val(),
        );

        {
            // drop RefMut<> from borrow_mut() before calc_statistics
            let mut data = self.data.borrow_mut();
            for (_fr, fr_val) in data.iter_mut() {
                *fr_val = (scale_fn)(fr_val.val()).into()
            }
            // drop RefMut<> from borrow_mut() before calc_statistics
        }
        self.calc_statistics();
    }

    /// Getter for lazily evaluated `average`.
    #[inline(always)]
    pub fn average(&self) -> FrequencyValue {
        self.average.get()
    }

    /// Getter for lazily evaluated `median`.
    #[inline(always)]
    pub fn median(&self) -> FrequencyValue {
        self.median.get()
    }

    /// Getter for lazily evaluated `max`.
    #[inline(always)]
    pub fn max(&self) -> FrequencyValue {
        self.max.get()
    }

    /// Getter for lazily evaluated `min`.
    #[inline(always)]
    pub fn min(&self) -> FrequencyValue {
        self.min.get()
    }

    /// [`max()`] - [`min()`]
    #[inline(always)]
    pub fn range(&self) -> FrequencyValue {
        self.max() - self.min()
    }

    /// Getter for `data`.
    #[inline(always)]
    pub fn data(&self) -> Ref<Vec<(Frequency, FrequencyValue)>> {
        self.data.borrow()
    }

    /// Returns a `BTreeMap`. The key is of type u32.
    /// (`f32` is not `Ord`, hence we can't use it as key.) You can optionally specify a
    /// scale function, e.g. multiply all frequencies with 1000 for better
    /// accuracy when represented as unsigned integer.
    ///
    /// ## Parameters
    /// * `scale_fn` optional scale function, e.g. multiply all frequencies with 1000 for better
    ///              accuracy when represented as unsigned integer.
    ///
    /// ## Return
    /// New `BTreeMap` from frequency to frequency value.
    #[inline(always)]
    pub fn to_map(&self, scale_fn: Option<&dyn Fn(f32) -> u32>) -> BTreeMap<u32, f32> {
        self.data.borrow().iter()
            .map(|(fr, fr_val)| (fr.val(), fr_val.val()))
            .map(|(fr, fr_val)| (
                    if let Some(fnc) = scale_fn {
                        (fnc)(fr)
                    } else {
                        fr as u32
                    },
                    fr_val
                )
            )
            .collect()
    }

    /*/// Returns an iterator over the underlying vector [`data`].
    #[inline(always)]
    pub fn iter(&self) -> Iter<(Frequency, FrequencyValue)> {
        self.data.borrow().iter()
    }*/

    /// Calculates min, max, median and average of the frequency values/magnitudes/amplitudes.
    #[inline(always)]
    fn calc_statistics(&self) {
        let data = self.data.borrow();
        // first: order all by frequency value in ascending order
        let mut vals = data.iter()
            // map to only value
            .map(|(_fr, val)| val)
            // f64 to prevent overflow
            // .map(|v| v as f64)
            .collect::<Vec<&FrequencyValue>>();
        vals.sort();

        // sum
        let sum: f32 = vals.iter()
            .map(|fr_val| fr_val.val())
            .fold(0.0, |a, b| a + b);

        let avg = sum / vals.len() as f32;
        let average: FrequencyValue = avg.into();

        let median = {
            // we assume that vals.length() is always even, because
            // it must be a power of 2 (for FFT)
            let a = *vals[vals.len() / 2 - 1];
            let b = *vals[vals.len() / 2];
            (a + b)/2.0.into()
        };
        let min = *vals[0];
        let max = *vals[vals.len() - 1];

        self.min.replace(min);
        self.max.replace(max);
        self.average.replace(average);
        self.median.replace(median);
    }
}

/*impl FromIterator<(Frequency, FrequencyValue)> for FrequencySpectrum {

    #[inline(always)]
    fn from_iter<T: IntoIterator<Item=(Frequency, FrequencyValue)>>(iter: T) -> Self {
        // 1024 is just a guess: most likely 2048 is a common FFT length,
        // i.e. 1024 results for the frequency spectrum.
        let mut vec = Vec::with_capacity(1024);
        for (fr, val) in iter {
            vec.push((fr, val))
        }

        FrequencySpectrum::new(vec)
    }
}*/


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrum() {
        let spectrum = vec![
            (0.0_f32, 5.0_f32),
            (50.0, 50.0),
            (100.0, 100.0),
            (150.0, 150.0),
            (200.0, 100.0),
            (250.0, 20.0),
            (300.0, 0.0),
            (450.0, 200.0),
        ];

        let spectrum = spectrum.into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();
        let spectrum = FrequencySpectrum::new(spectrum);

        assert_eq!((0.0.into(), 5.0.into()), spectrum.data()[0], "Vector must be ordered");
        assert_eq!((50.0.into(), 50.0.into()), spectrum.data()[1], "Vector must be ordered");
        assert_eq!((100.0.into(), 100.0.into()), spectrum.data()[2], "Vector must be ordered");
        assert_eq!((150.0.into(), 150.0.into()), spectrum.data()[3], "Vector must be ordered");
        assert_eq!((200.0.into(), 100.0.into()), spectrum.data()[4], "Vector must be ordered");
        assert_eq!((250.0.into(), 20.0.into()), spectrum.data()[5], "Vector must be ordered");
        assert_eq!((300.0.into(), 0.0.into()), spectrum.data()[6], "Vector must be ordered");
        assert_eq!((450.0.into(), 200.0.into()), spectrum.data()[7], "Vector must be ordered");

        assert_eq!(0.0, spectrum.min().val(), "min() must work");
        assert_eq!(200.0, spectrum.max().val(), "max() must work");
        assert_eq!(200.0 - 0.0, spectrum.range().val(), "range() must work");
        assert_eq!(78.125, spectrum.average().val(), "average() must work");
        assert_eq!((50 + 100) as f32 / 2.0, spectrum.median().val(), "median() must work");
    }
}
