use crate::delay_line::DelayLine;

pub struct Allpass {
    gain: f32,
    sample_rate: f32,
    delay_input: DelayLine,
    delay_output: DelayLine
}

impl Allpass {
    pub fn new(delay: f32, gain: f32, sample_rate: f32) -> Allpass {
        let frames = (delay * sample_rate) as usize;
        let mut d_in = DelayLine::new(frames * 2);
        let mut d_out = DelayLine::new(frames * 2);
        d_in.set_duration(frames);
        d_out.set_duration(frames);
        Allpass {
            gain,
            sample_rate,
            delay_input: d_in,
            delay_output: d_out,
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay_input.set_duration((delay * self.sample_rate) as usize);
        self.delay_output.set_duration((delay * self.sample_rate) as usize);
    }

    pub fn process(&mut self, input: f32, output: &mut f32) {
       let mut delayed_out = 0.0;
       let mut delayed_in = 0.0;
       self.delay_input.process(input, &mut delayed_in);
       self.delay_output.read(&mut delayed_out);
       *output = (-self.gain * input) + delayed_in + (self.gain * delayed_out);
       self.delay_output.write(*output);
    }
}
