use crate::biquad::Biquad;

#[derive(Copy,Clone)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    LowShelf,
    HighShelf,
    Peaking,
    AllPass,
    Notch,
}

pub struct Filter {
    filter_type: FilterType,
    frequency: f32,
    q: f32,
    gain: f32,
    biquad: Biquad
}

impl Filter {
    pub fn new(filter_type: FilterType, frequency: f32, q: f32, gain: f32, sample_rate: f32) -> Filter {
        let nyquist = sample_rate / 2.0;
        let frequency = frequency / nyquist;
        let mut biquad = Biquad::new(sample_rate);

        match filter_type {
            FilterType::LowPass => {
                biquad.set_lowpass_params(frequency, q);
            }
            FilterType::HighPass => {
                biquad.set_highpass_params(frequency, q);
            }
            FilterType::BandPass => {
                biquad.set_bandpass_params(frequency, q);
            }
            FilterType::LowShelf => {
                biquad.set_lowshelf_params(frequency, gain);
            }
            FilterType::HighShelf => {
                biquad.set_highshelf_params(frequency, gain);
            }
            FilterType::Peaking => {
                biquad.set_peaking_params(frequency, q, gain);
            }
            FilterType::AllPass => {
                biquad.set_allpass_params(frequency, q);
            }
            FilterType::Notch => {
                biquad.set_notch_params(frequency, q);
            }
        }

        Filter {
            filter_type: filter_type,
            frequency,
            q,
            gain,
            biquad
        }
    }
    pub fn lowpass(frequency: f32, q: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::LowPass, frequency, q, 1.0, sample_rate)
    }
    pub fn highpass(frequency: f32, q: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::HighPass, frequency, q, 1.0, sample_rate)
    }
    pub fn bandpass(frequency: f32, q: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::BandPass, frequency, q, 1.0, sample_rate)
    }
    pub fn lowshelf(frequency: f32, gain: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::LowShelf, frequency, 0.0, gain, sample_rate)
    }
    pub fn highshelf(frequency: f32, gain: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::HighShelf, frequency, 0.0, gain, sample_rate)
    }
    pub fn peaking(frequency: f32, q: f32, gain: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::Peaking, frequency, q, gain, sample_rate)
    }
    pub fn allpass(frequency: f32, q: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::Peaking, frequency, q, 1.0, sample_rate)
    }
    pub fn notch(frequency: f32, q: f32, sample_rate: f32) -> Filter {
        Filter::new(FilterType::Notch, frequency, q, 1.0, sample_rate)
    }
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.set_params_on_biquad();
    }
    pub fn set_q(&mut self, q: f32) {
        self.q = q;
        self.set_params_on_biquad();
    }
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
        self.set_params_on_biquad();
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.biquad.process(input, output);
    }
    pub fn set_params_on_biquad(&mut self) {
        match self.filter_type {
            FilterType::LowPass => {
                self.biquad.set_lowpass_params(self.frequency, self.q);
            }
            FilterType::HighPass => {
                self.biquad.set_highpass_params(self.frequency, self.q);
            }
            FilterType::BandPass => {
                self.biquad.set_bandpass_params(self.frequency, self.q);
            }
            FilterType::LowShelf => {
                self.biquad.set_lowshelf_params(self.frequency, self.gain);
            }
            FilterType::HighShelf => {
                self.biquad.set_highshelf_params(self.frequency, self.gain);
            }
            FilterType::Peaking => {
                self.biquad.set_peaking_params(self.frequency, self.q, self.gain);
            }
            FilterType::AllPass => {
                self.biquad.set_allpass_params(self.frequency, self.q);
            }
            FilterType::Notch => {
                self.biquad.set_notch_params(self.frequency, self.q);
            }
        }
    }
}
