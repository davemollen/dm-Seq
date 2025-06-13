pub struct SyncedPhasor {
  last_ramp: f32,
  phase: f32,
  speed: f32,
}

impl SyncedPhasor {
  pub fn new() -> Self {
    Self {
      last_ramp: 0.0,
      phase: 0.0,
      speed: 1.0,
    }
  }

  pub fn set_initial_speed(&mut self, speed: f32) {
    self.speed = speed;
  }

  pub fn process(&mut self, ramp: f32, speed: f32) -> f32 {
    let mut delta = ramp - self.last_ramp;
    if delta < 0. {
      delta += 1.;
      if speed != self.speed {
        self.phase = 0.;
        self.speed = speed;
        self.last_ramp = ramp;
        return self.phase;
      }
    }

    self.phase = (self.phase + delta * self.speed).fract();
    self.last_ramp = ramp;

    return self.phase;
  }
}

#[cfg(test)]
mod tests {
  use crate::synced_phasor::SyncedPhasor;

  fn assert_approx_eq(left: f32, right: f32) {
    let left = (left * 100000.).round() / 100000.;
    let right = (right * 100000.).round() / 100000.;
    assert_eq!(left, right);
  }

  #[test]
  fn should_ramp_up_twice_as_slow() {
    let mut phasor = SyncedPhasor::new();
    phasor.set_initial_speed(0.5);
    assert_approx_eq(phasor.process(0.0, 0.5), 0.);
    assert_approx_eq(phasor.process(0.2, 0.5), 0.1);
    assert_approx_eq(phasor.process(0.4, 0.5), 0.2);
    assert_approx_eq(phasor.process(0.6, 0.5), 0.3);
    assert_approx_eq(phasor.process(0.8, 0.5), 0.4);
    assert_approx_eq(phasor.process(0.0, 0.5), 0.5);
    assert_approx_eq(phasor.process(0.2, 0.5), 0.6);
    assert_approx_eq(phasor.process(0.4, 0.5), 0.7);
    assert_approx_eq(phasor.process(0.6, 0.5), 0.8);
    assert_approx_eq(phasor.process(0.8, 0.5), 0.9);
    assert_approx_eq(phasor.process(0.0, 0.5), 0.);
    assert_approx_eq(phasor.process(0.2, 0.5), 0.1);
    assert_approx_eq(phasor.process(0.4, 0.5), 0.2);
    assert_approx_eq(phasor.process(0.6, 0.5), 0.3);
    assert_approx_eq(phasor.process(0.8, 0.5), 0.4);
    assert_approx_eq(phasor.process(0.0, 0.5), 0.5);
    assert_approx_eq(phasor.process(0.2, 0.5), 0.6);
    assert_approx_eq(phasor.process(0.4, 0.5), 0.7);
    assert_approx_eq(phasor.process(0.6, 0.5), 0.8);
    assert_approx_eq(phasor.process(0.8, 0.5), 0.9);
  }

  #[test]
  fn should_ramp_up_even_slower() {
    let mut phasor = SyncedPhasor::new();
    phasor.set_initial_speed(0.25);
    assert_approx_eq(phasor.process(0.0, 0.25), 0.);
    assert_approx_eq(phasor.process(0.2, 0.25), 0.05);
    assert_approx_eq(phasor.process(0.4, 0.25), 0.1);
    assert_approx_eq(phasor.process(0.6, 0.25), 0.15);
    assert_approx_eq(phasor.process(0.8, 0.25), 0.2);
    assert_approx_eq(phasor.process(0.0, 0.25), 0.25);
    assert_approx_eq(phasor.process(0.2, 0.25), 0.3);
    assert_approx_eq(phasor.process(0.4, 0.25), 0.35);
  }

  #[test]
  fn should_ramp_up_twice_as_fast() {
    let mut phasor = SyncedPhasor::new();
    phasor.set_initial_speed(2.);
    assert_approx_eq(phasor.process(0., 2.), 0.);
    assert_approx_eq(phasor.process(0.2, 2.), 0.4);
    assert_approx_eq(phasor.process(0.4, 2.), 0.8);
    assert_approx_eq(phasor.process(0.6, 2.), 0.2);
    assert_approx_eq(phasor.process(0.8, 2.), 0.6);
    assert_approx_eq(phasor.process(0.0, 2.), 0.0);
    assert_approx_eq(phasor.process(0.2, 2.), 0.4);
  }

  #[test]
  fn should_ramp_up_at_original_speed() {
    let mut phasor = SyncedPhasor::new();
    assert_approx_eq(phasor.process(0., 1.), 0.);
    assert_approx_eq(phasor.process(0.2, 1.), 0.2);
    assert_approx_eq(phasor.process(0.4, 1.), 0.4);
    assert_approx_eq(phasor.process(0.6, 1.), 0.6);
    assert_approx_eq(phasor.process(0.8, 1.), 0.8);
    assert_approx_eq(phasor.process(0., 1.), 0.);
    assert_approx_eq(phasor.process(0.2, 1.), 0.2);
    assert_approx_eq(phasor.process(0.4, 1.), 0.4);
    assert_approx_eq(phasor.process(0.6, 1.), 0.6);
    assert_approx_eq(phasor.process(0.8, 1.), 0.8);
  }

  #[test]
  fn should_switch_speed_only_at_wrap_around() {
    let mut phasor = SyncedPhasor::new();
    assert_approx_eq(phasor.process(0., 1.), 0.);
    assert_approx_eq(phasor.process(0.2, 1.), 0.2);
    assert_approx_eq(phasor.process(0.4, 1.), 0.4);
    assert_approx_eq(phasor.process(0.6, 1.), 0.6);
    assert_approx_eq(phasor.process(0.8, 2.), 0.8);
    assert_eq!(phasor.process(0., 2.), 0.);
    assert_eq!(phasor.process(0.2, 2.), 0.4);
  }
}
