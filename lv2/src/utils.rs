use {
  crate::{DmSeq, Ports},
  lv2::{
    lv2_atom::{object::ObjectReader, sequence::SequenceWriter},
    prelude::TimeStamp,
  },
  wmidi::{Channel, ControlNumber, ControlValue, MidiMessage},
};

pub struct SequencerData {
  pub notes: [u8; 16],
  pub velocities: [u8; 16],
  pub note_lengths: [f64; 16],
  pub chance: [f32; 16],
  pub channels: [u8; 16],
  pub gates: [bool; 16],
}

pub struct NextStep {
  pub note: u8,
  pub velocity: u8,
  pub channel: u8,
  pub note_length: f64,
  pub step: usize,
  pub is_note_on: bool,
}

impl DmSeq {
  pub fn map_sequencer_data(&self, ports: &mut Ports) -> SequencerData {
    let notes = [
      ports.note_1.get(),
      ports.note_2.get(),
      ports.note_3.get(),
      ports.note_4.get(),
      ports.note_5.get(),
      ports.note_6.get(),
      ports.note_7.get(),
      ports.note_8.get(),
      ports.note_9.get(),
      ports.note_10.get(),
      ports.note_11.get(),
      ports.note_12.get(),
      ports.note_13.get(),
      ports.note_14.get(),
      ports.note_15.get(),
      ports.note_16.get(),
    ]
    .map(|note| note as u8);
    let velocities = [
      ports.velocity_1.get(),
      ports.velocity_2.get(),
      ports.velocity_3.get(),
      ports.velocity_4.get(),
      ports.velocity_5.get(),
      ports.velocity_6.get(),
      ports.velocity_7.get(),
      ports.velocity_8.get(),
      ports.velocity_9.get(),
      ports.velocity_10.get(),
      ports.velocity_11.get(),
      ports.velocity_12.get(),
      ports.velocity_13.get(),
      ports.velocity_14.get(),
      ports.velocity_15.get(),
      ports.velocity_16.get(),
    ]
    .map(|vel| vel as u8);
    let note_lengths = [
      ports.note_length_1.get(),
      ports.note_length_2.get(),
      ports.note_length_3.get(),
      ports.note_length_4.get(),
      ports.note_length_5.get(),
      ports.note_length_6.get(),
      ports.note_length_7.get(),
      ports.note_length_8.get(),
      ports.note_length_9.get(),
      ports.note_length_10.get(),
      ports.note_length_11.get(),
      ports.note_length_12.get(),
      ports.note_length_13.get(),
      ports.note_length_14.get(),
      ports.note_length_15.get(),
      ports.note_length_16.get(),
    ]
    .map(|note_length| note_length as f64);
    let chance = [
      ports.chance_1.get(),
      ports.chance_2.get(),
      ports.chance_3.get(),
      ports.chance_4.get(),
      ports.chance_5.get(),
      ports.chance_6.get(),
      ports.chance_7.get(),
      ports.chance_8.get(),
      ports.chance_9.get(),
      ports.chance_10.get(),
      ports.chance_11.get(),
      ports.chance_12.get(),
      ports.chance_13.get(),
      ports.chance_14.get(),
      ports.chance_15.get(),
      ports.chance_16.get(),
    ]
    .map(|chance| chance * 0.01);
    let channels = [
      ports.channel_1.get(),
      ports.channel_2.get(),
      ports.channel_3.get(),
      ports.channel_4.get(),
      ports.channel_5.get(),
      ports.channel_6.get(),
      ports.channel_7.get(),
      ports.channel_8.get(),
      ports.channel_9.get(),
      ports.channel_10.get(),
      ports.channel_11.get(),
      ports.channel_12.get(),
      ports.channel_13.get(),
      ports.channel_14.get(),
      ports.channel_15.get(),
      ports.channel_16.get(),
    ]
    .map(|channel| channel as u8 - 1);
    let gates = [
      ports.gate_1.get() == 1.,
      ports.gate_2.get() == 1.,
      ports.gate_3.get() == 1.,
      ports.gate_4.get() == 1.,
      ports.gate_5.get() == 1.,
      ports.gate_6.get() == 1.,
      ports.gate_7.get() == 1.,
      ports.gate_8.get() == 1.,
      ports.gate_9.get() == 1.,
      ports.gate_10.get() == 1.,
      ports.gate_11.get() == 1.,
      ports.gate_12.get() == 1.,
      ports.gate_13.get() == 1.,
      ports.gate_14.get() == 1.,
      ports.gate_15.get() == 1.,
      ports.gate_16.get() == 1.,
    ];

    return SequencerData {
      notes,
      velocities,
      note_lengths,
      chance,
      channels,
      gates,
    };
  }

  pub fn resolve_next_step(
    &mut self,
    ports: &mut Ports,
    sequencer_data: &SequencerData,
  ) -> NextStep {
    let SequencerData {
      notes,
      velocities,
      note_lengths,
      chance,
      channels,
      gates,
    } = *sequencer_data;

    self.advance_step(ports);
    let reordered_step = self.map_current_step_to_reordered_step(ports);
    let repositioned_step =
      (reordered_step + ports.step_offset.get() as usize) % ports.steps.get() as usize;
    let note = notes[repositioned_step];
    let velocity = velocities[repositioned_step];
    let note_length = note_lengths[repositioned_step];
    let channel = channels[repositioned_step];
    let gate = gates[repositioned_step];
    let chance = chance[repositioned_step];
    let probability_gate = if chance == 0. {
      false
    } else if chance == 1. {
      true
    } else {
      fastrand::f32() < chance
    };
    let is_note_on = velocity > 0 && gate && probability_gate;

    NextStep {
      note,
      velocity,
      channel,
      note_length,
      step: repositioned_step,
      is_note_on,
    }
  }

  pub fn map_step_duration_to_divisor(&self, step_duration: f32) -> f64 {
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

  pub fn get_step_duration_in_samples(&self, bpm: f64, division: f64) -> f64 {
    let samples_per_beat = self.sample_rate as f64 * 60.0 / bpm;
    samples_per_beat * division.recip()
  }

  pub fn get_swing_offset_in_samples(
    &self,
    ports: &mut Ports,
    step_duration_in_samples: f64,
  ) -> f64 {
    let step_is_an_even_number = self.current_step & 1 == 0;
    if step_is_an_even_number {
      0.
    } else {
      ports.swing.get() as f64 * 0.5 * step_duration_in_samples
    }
  }

  pub fn set_shuffled_steps(&mut self, steps: usize, should_reshuffle_on_repeat: bool) {
    if should_reshuffle_on_repeat {
      loop {
        fastrand::shuffle(&mut self.shuffled_steps);
        let first_shuffled_step = self
          .shuffled_steps
          .iter()
          .find(|step| **step < steps)
          .unwrap();

        // Reshuffle if this shuffle starts with the previous last element
        if *first_shuffled_step == self.last_shuffled_step {
          continue;
        }
        self.last_shuffled_step = self
          .shuffled_steps
          .iter()
          .filter(|step| **step < steps)
          .last()
          .map_or(self.last_shuffled_step, |x| *x);
        break;
      }
    } else {
      fastrand::shuffle(&mut self.shuffled_steps);
    }
  }

  pub fn update_position(&mut self, object_reader: ObjectReader<'static>) {
    for (property_header, property) in object_reader {
      if property_header.key == self.urids.time.beats_per_minute {
        self.host_bpm = property.read(self.urids.atom.float, ()).unwrap_or(120.) as f64;
      }
      if property_header.key == self.urids.time.speed {
        self.host_speed = property.read(self.urids.atom.float, ()).unwrap_or(0.);
      }
      if property_header.key == self.urids.time.beat_unit {
        self.beat_unit = property.read(self.urids.atom.int, ()).unwrap_or(4);
      }
      if property_header.key == self.urids.time.bar_beat {
        let beat = property
          .read(self.urids.atom.double, ())
          .map_or(0., |beat| beat.fract());
        self.synced_phasor.process(beat, 0.25);
      }
      if property_header.key == self.urids.time.frame {
        self.block_start_frame = property.read(self.urids.atom.long, ()).unwrap_or(0);
      }
    }
  }

  pub fn handle_transport_stopped(&mut self, ports: &mut Ports) {
    self.current_step = 15;
    ports.current_step.set(-1.);
    self.is_in_swing_cycle = true;
    self.next_step_frame = 0;
    self.event_queue.stop_all_notes();
    self.should_alternate_sequence = true;
    self.synced_phasor.reset();
  }

  pub fn midi_panic(&self, midi_out_sequence: &mut SequenceWriter<'static, '_>) {
    for channel in 0..16 {
      // set sustain to zero
      midi_out_sequence.init(
        TimeStamp::Frames(0),
        self.urids.midi.wmidi,
        MidiMessage::ControlChange(
          Channel::from_index(channel).unwrap(),
          ControlNumber::try_from(64).unwrap(),
          ControlValue::try_from(0).unwrap(),
        ),
      );
      // send notes off message
      midi_out_sequence.init(
        TimeStamp::Frames(0),
        self.urids.midi.wmidi,
        MidiMessage::ControlChange(
          Channel::from_index(channel).unwrap(),
          ControlNumber::try_from(123).unwrap(),
          ControlValue::try_from(0).unwrap(),
        ),
      );
      // send sound off message
      midi_out_sequence.init(
        TimeStamp::Frames(0),
        self.urids.midi.wmidi,
        MidiMessage::ControlChange(
          Channel::from_index(channel).unwrap(),
          ControlNumber::try_from(120).unwrap(),
          ControlValue::try_from(0).unwrap(),
        ),
      );
    }
  }

  fn map_current_step_to_reordered_step(&mut self, ports: &mut Ports) -> usize {
    let order = ports.order.get() as u8;
    let steps = ports.steps.get() as usize;

    let reordered_step = match order {
      1 => {
        // Reverse
        steps - self.current_step - 1
      }
      2 => {
        // Alternate
        if self.current_step == 0 {
          self.should_alternate_sequence = !self.should_alternate_sequence;
        }

        if self.should_alternate_sequence {
          steps - self.current_step - 1
        } else {
          self.current_step
        }
      }
      3 => {
        // Pendulum
        if self.should_alternate_sequence {
          if self.current_step == 0 {
            self.should_alternate_sequence = !self.should_alternate_sequence;
            self.advance_step(ports);
            return self.current_step;
          }
          steps - self.current_step - 1
        } else {
          if self.current_step >= steps - 1 {
            self.should_alternate_sequence = !self.should_alternate_sequence;
            self.advance_step(ports);
            return steps - self.current_step - 1;
          }
          self.current_step
        }
      }
      4 => {
        // Random
        fastrand::usize(0..steps)
      }
      5 => {
        // Shuffle A
        if steps == 1 {
          return self.current_step;
        }
        if self.current_step == 0 || steps != self.prev_steps {
          self.set_shuffled_steps(steps, false);
        }
        self
          .shuffled_steps
          .iter()
          .filter(|step| **step < steps)
          .nth(self.current_step)
          .map_or(self.current_step, |x| *x)
      }
      6 => {
        // Shuffle B
        if steps == 1 {
          return self.current_step;
        }
        if self.current_step == 0 || steps != self.prev_steps {
          self.set_shuffled_steps(steps, true);
        }
        self
          .shuffled_steps
          .iter()
          .filter(|step| **step < steps)
          .nth(self.current_step)
          .map_or(self.current_step, |x| *x)
      }
      7 => {
        // Brownian
        let random = fastrand::f32();
        if random < 0.25 {
          // go to previous step
          self.current_step = (self.current_step + steps - 2) % steps;
          return self.current_step;
        }
        if random < 0.5 {
          // stay on current step
          self.current_step = (self.current_step + steps - 1) % steps;
          return self.current_step;
        };
        // advance to the next step
        self.current_step
      }
      8 => {
        // Either Way
        if self.current_step == 0 {
          self.should_alternate_sequence = fastrand::bool();
        }

        if self.should_alternate_sequence {
          steps - self.current_step - 1
        } else {
          self.current_step
        }
      }
      _ => {
        // Forward
        self.current_step
      }
    };
    self.prev_steps = steps;
    return reordered_step;
  }

  fn advance_step(&mut self, ports: &mut Ports) {
    let next_step = self.current_step + 1;
    self.current_step = if next_step >= ports.steps.get() as usize {
      0
    } else {
      next_step
    };
  }
}
