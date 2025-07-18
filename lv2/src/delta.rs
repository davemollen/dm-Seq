pub struct Delta {
  z: f32,
}

impl Delta {
  pub fn new(initial_value: f32) -> Self {
    Self { z: initial_value }
  }

  pub fn process(&mut self, input: f32) -> f32 {
    let output = input - self.z;
    self.z = input;
    output
  }

  pub fn reset(&mut self, value: f32) {
    self.z = value;
  }
}
