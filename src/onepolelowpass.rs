use std::f32::consts::PI;

pub struct OnePoleLowPass {
    a0: f32,
    b1: f32,
    z1: f32,
    sample_rate: f32
}

impl OnePoleLowPass {
    pub fn new(freq: f32, sample_rate: f32) -> OnePoleLowPass {
        let mut lp = OnePoleLowPass {
            a0: 1.0,
            b1: 0.0,
            z1: 0.0,
            sample_rate
        };
        lp.set_frequency(freq);
        return lp;
    }
    pub fn set_frequency(&mut self, freq: f32) {
        let normalized_freq = freq / self.sample_rate;
        self.b1 = (-2.0 * PI * normalized_freq).exp();
        self.a0 = 1.0 - self.b1;
    }
    pub fn process(&mut self, input: f32, output: &mut f32) {
        self.z1 = input * self.a0 + self.z1 * self.b1;
        *output = self.z1;
    }
}
