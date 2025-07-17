use {
  crate::{DmSeq, Ports},
  lv2::lv2_atom::object::ObjectReader,
};

pub struct SequencerData {
  pub notes: [u8; 16],
  pub velocities: [u8; 16],
  pub gates: [bool; 16],
}

impl DmSeq {
  pub fn map_sequencer_data(&self, ports: &mut Ports) -> SequencerData {
    // TODO: check for CPU
    // let notes: [u8; 16] = unsafe {
    //   mem::transmute_copy(&[
    //     *ports.note_1,
    //     *ports.note_2,
    //     *ports.note_3,
    //     *ports.note_4,
    //     *ports.note_5,
    //     *ports.note_6,
    //     *ports.note_7,
    //     *ports.note_8,
    //     *ports.note_9,
    //     *ports.note_10,
    //     *ports.note_11,
    //     *ports.note_12,
    //     *ports.note_13,
    //     *ports.note_14,
    //     *ports.note_15,
    //     *ports.note_16,
    //   ])
    // };
    // let velocities: [u8; 16] = unsafe {
    //   mem::transmute_copy(&[
    //     *ports.velocity_1,
    //     *ports.velocity_2,
    //     *ports.velocity_3,
    //     *ports.velocity_4,
    //     *ports.velocity_5,
    //     *ports.velocity_6,
    //     *ports.velocity_7,
    //     *ports.velocity_8,
    //     *ports.velocity_9,
    //     *ports.velocity_10,
    //     *ports.velocity_11,
    //     *ports.velocity_12,
    //     *ports.velocity_13,
    //     *ports.velocity_14,
    //     *ports.velocity_15,
    //     *ports.velocity_16,
    //   ])
    // };
    let notes = [
      *ports.note_1,
      *ports.note_2,
      *ports.note_3,
      *ports.note_4,
      *ports.note_5,
      *ports.note_6,
      *ports.note_7,
      *ports.note_8,
      *ports.note_9,
      *ports.note_10,
      *ports.note_11,
      *ports.note_12,
      *ports.note_13,
      *ports.note_14,
      *ports.note_15,
      *ports.note_16,
    ]
    .map(|note| note as u8);
    let velocities = [
      *ports.velocity_1,
      *ports.velocity_2,
      *ports.velocity_3,
      *ports.velocity_4,
      *ports.velocity_5,
      *ports.velocity_6,
      *ports.velocity_7,
      *ports.velocity_8,
      *ports.velocity_9,
      *ports.velocity_10,
      *ports.velocity_11,
      *ports.velocity_12,
      *ports.velocity_13,
      *ports.velocity_14,
      *ports.velocity_15,
      *ports.velocity_16,
    ]
    .map(|vel| vel as u8);
    let gates = [
      *ports.gate_1 == 1.,
      *ports.gate_2 == 1.,
      *ports.gate_3 == 1.,
      *ports.gate_4 == 1.,
      *ports.gate_5 == 1.,
      *ports.gate_6 == 1.,
      *ports.gate_7 == 1.,
      *ports.gate_8 == 1.,
      *ports.gate_9 == 1.,
      *ports.gate_10 == 1.,
      *ports.gate_11 == 1.,
      *ports.gate_12 == 1.,
      *ports.gate_13 == 1.,
      *ports.gate_14 == 1.,
      *ports.gate_15 == 1.,
      *ports.gate_16 == 1.,
    ];

    return SequencerData {
      notes,
      velocities,
      gates,
    };
  }

  pub fn map_step_duration_to_divisor(&self, step_duration: f32) -> f32 {
    /*
    lv2:scalePoint [ rdfs:label "64th";      	rdf:value 0 ; ] ;
    lv2:scalePoint [ rdfs:label "32th";      	rdf:value 1 ; ] ;
    lv2:scalePoint [ rdfs:label "16th";      	rdf:value 2 ; ] ;
    lv2:scalePoint [ rdfs:label "8th";       	rdf:value 3 ; ] ;
    lv2:scalePoint [ rdfs:label "Quarter";   	rdf:value 4 ; ] ;
    lv2:scalePoint [ rdfs:label "Half Note"; 	rdf:value 5 ; ] ;
    lv2:scalePoint [ rdfs:label "Whole Note"; 	rdf:value 6 ; ] ;
    */
    match step_duration {
      0. => 64.,
      1. => 32.,
      2. => 16.,
      3. => 8.,
      4. => 4.,
      5. => 2.,
      6. => 1.,
      _ => 16.,
    }
  }

  pub fn map_current_step_to_reordered_step(&mut self, order: u8, steps: usize) -> usize {
    let reordered_step = match order {
      1 => {
        // reverse
        steps - self.current_step - 1
      }
      2 => {
        // shuffle
        if self.current_step == 0 || steps != self.prev_steps {
          self.set_shuffled_steps(steps);
        }
        self.shuffled_steps[self.current_step]
      }
      3 => {
        // random
        fastrand::usize(0..steps)
      }
      _ => self.current_step,
    };
    self.prev_steps = steps;
    return reordered_step;
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

  pub fn update_position(&mut self, object_reader: ObjectReader<'static>) {
    for (property_header, property) in object_reader {
      if property_header.key == self.urids.time.beat_unit {
        self.host_div = property.read(self.urids.atom.int, ()).unwrap_or(4);
      }
      if property_header.key == self.urids.time.beats_per_minute {
        self.host_bpm = property.read(self.urids.atom.float, ()).unwrap_or(120.);
      }
      if property_header.key == self.urids.time.speed {
        self.host_speed = property.read(self.urids.atom.float, ()).unwrap_or(0.);
      }
      if property_header.key == self.urids.time.bar_beat {
        self.beat = property
          .read(self.urids.atom.float, ())
          .map_or(0., |beat| beat.fract());
      }
    }
  }

  pub fn map_step_progress_to_trigger(&mut self, step_progress: f32, swing: f32) -> bool {
    let non_swing_trigger = self.step_progress_delta.process(step_progress) < 0.;
    if non_swing_trigger {
      self.is_in_swing_cycle = !self.is_in_swing_cycle;
    };
    let trigger = if self.is_in_swing_cycle && swing != 0. {
      self.swing_delta.process(if step_progress > (swing * 0.5) {
        1.
      } else {
        0.
      }) > 0.
    } else {
      non_swing_trigger
    };

    trigger
  }

  pub fn handle_transport_stopped(&mut self, ports: &mut Ports) {
    self.current_step = 15;
    **ports.current_step = -1.;
    self.prev_note = None;
    self.is_in_swing_cycle = true;
  }

  pub fn get_trigger(&mut self, ports: &mut Ports, sample_count: u32) -> bool {
    match *ports.clock_mode {
      0. => {
        // Trigger mode
        *ports.trigger == 1.
      }
      1. => {
        // Host sync
        let speed = self.map_step_duration_to_divisor(*ports.step_duration) / self.host_div as f32;
        let step_progress = self.step_progress_phasor.process(self.beat, speed);
        let trigger = self.map_step_progress_to_trigger(step_progress, *ports.swing);
        trigger
      }
      2. => {
        // Free running
        let speed_factor = self.map_step_duration_to_divisor(*ports.step_duration) / 4.;
        let freq = *ports.bpm / 60. * speed_factor;
        let step_progress = self.phasor.process(freq, sample_count);
        let trigger = self.map_step_progress_to_trigger(step_progress, *ports.swing);
        trigger
      }
      _ => false,
    }
  }
}
