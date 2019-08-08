pub struct DelayLine {
    memory: Vec<f32>,
    duration: usize,
    read_index: usize,
    write_index: usize,
}

impl DelayLine {
    pub fn new(max_duration: usize) -> DelayLine {
        let mut v = Vec::<f32>::with_capacity(max_duration);
        v.resize(max_duration, 0.0);
        let mut d = DelayLine {
            memory: v,
            duration: 0,
            read_index: 0,
            write_index: 0,
        };
        d.set_duration(max_duration);
        return d;
    }
    pub fn set_duration(&mut self, duration: usize) {
        self.duration = duration;
        self.write_index = self.write_index % duration;
        self.read_index = if self.write_index < self.duration {
            self.memory.len() - (duration - self.write_index)
        } else {
            self.write_index - duration
        };
    }
    pub fn write(&mut self, input: &[f32]) {
        let mut w = self.write_index;
        let l = self.memory.len();
        for i in input.iter() {
            self.memory[w] = *i;
            w = (w + 1) % l;
        }
        self.write_index = w;
    }
    pub fn read(&mut self, output: &mut [f32]) {
        let mut r = self.read_index;
        let l = self.memory.len();
        for o in output.iter_mut() {
            *o = self.memory[r];
            r = (r + 1) % l;
        }
        self.read_index = r;
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.write(input);
        self.read(output);
    }
    pub fn process_single(&mut self, input: f32, output: &mut f32) {
        let i = [input; 1];
        let mut o = [0.0; 1];
        self.write(&i);
        self.read(&mut o);
        *output = o[0];
    }
}
