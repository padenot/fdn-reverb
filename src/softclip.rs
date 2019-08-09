pub struct Softclip {
    hardness: f32
}

impl Softclip {
    pub fn new(hardness: f32) -> Softclip {
        Softclip {
            hardness
        }
    }
    pub fn set_hardness(&mut self, hardness: f32) {
        self.hardness = hardness;
    }
    pub fn process(&mut self, input: f32, output: &mut f32) {
        *output = (input * self.hardness).tanh() / self.hardness;
    }
}
