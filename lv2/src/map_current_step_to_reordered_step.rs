use crate::DmSeq;

impl DmSeq {
  pub fn map_current_step_to_reordered_step(&mut self, order: u8, steps: usize) -> usize {
    match order {
      1 => {
        // reverse
        steps - self.current_step - 1
      }
      2 => {
        // shuffle
        if self.current_step == 0 {
          self.set_shuffled_steps(steps);
        }
        self.shuffled_steps[self.current_step]
      }
      3 => {
        // random
        fastrand::usize(0..steps)
      }
      _ => self.current_step,
    }
  }

  pub fn set_shuffled_steps(&mut self, steps: usize) {
    self.shuffled_steps.resize(steps, 0);
    self
      .shuffled_steps
      .iter_mut()
      .enumerate()
      .for_each(|(i, val)| {
        *val = i;
      });
    fastrand::shuffle(&mut self.shuffled_steps);
  }
}
