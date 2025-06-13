pub struct Phasor {
  sample_period: f32,
  x: f32,
}

impl Phasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      sample_period: sample_rate.recip(),
      x: 0.,
    }
  }

  pub fn process(&mut self, freq: f32, sample_count: u32) -> f32 {
    self.x = (self.x + (freq * self.sample_period * sample_count as f32)).fract();
    self.x
  }
}

#[cfg(test)]
mod tests {
  use crate::phasor::Phasor;

  fn assert_approx_eq(left: f32, right: f32) {
    let left = (left * 100000.).round() / 100000.;
    let right = (right * 100000.).round() / 100000.;
    assert_eq!(left, right);
  }

  #[test]
  fn should_ramp_up_at_desired_speed() {
    let mut phasor = Phasor::new(10.);
    assert_approx_eq(phasor.process(1.0, 1), 0.1);
    assert_approx_eq(phasor.process(1.0, 1), 0.2);
    assert_approx_eq(phasor.process(1.0, 1), 0.3);
    assert_approx_eq(phasor.process(1.0, 1), 0.4);
    assert_approx_eq(phasor.process(1.0, 1), 0.5);
    assert_approx_eq(phasor.process(1.0, 1), 0.6);
    assert_approx_eq(phasor.process(1.0, 1), 0.7);
    assert_approx_eq(phasor.process(1.0, 1), 0.8);
    assert_approx_eq(phasor.process(1.0, 1), 0.9);
    assert_approx_eq(phasor.process(1.0, 1), 0.0);
    assert_approx_eq(phasor.process(2.0, 1), 0.2);
    assert_approx_eq(phasor.process(2.0, 1), 0.4);
    assert_approx_eq(phasor.process(2.0, 1), 0.6);
    assert_approx_eq(phasor.process(2.0, 1), 0.8);
    assert_approx_eq(phasor.process(2.0, 1), 0.0);
    assert_approx_eq(phasor.process(0.5, 1), 0.05);
    assert_approx_eq(phasor.process(0.5, 1), 0.1);
  }
}
