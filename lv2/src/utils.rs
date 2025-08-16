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
  pub gates: [bool; 16],
}

pub struct NextStep {
  pub note: u8,
  pub velocity: u8,
  pub channel: u8,
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
      gates,
    };
  }

  pub fn resolve_next_step(
    &mut self,
    ports: &mut Ports,
    notes: [u8; 16],
    velocities: [u8; 16],
    gates: [bool; 16],
  ) -> NextStep {
    let next_step = self.current_step + 1;
    self.current_step = if next_step >= ports.steps.get() as usize {
      0
    } else {
      next_step
    };

    let reordered_step =
      self.map_current_step_to_reordered_step(ports.order.get() as u8, ports.steps.get() as usize);
    let note = notes[reordered_step];
    let velocity = velocities[reordered_step];
    let gate = gates[reordered_step];
    let is_note_on = velocity > 0 && gate;
    ports.current_step.set(reordered_step as f32);

    NextStep {
      note,
      velocity,
      channel: ports.midi_channel.get() as u8,
      is_note_on,
    }
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

  pub fn get_step_duration_in_samples(&self, bpm: f32, division: f32) -> f32 {
    let samples_per_beat = self.sample_rate * 60.0 / bpm;
    samples_per_beat * division.recip()
  }

  pub fn get_swing_offset_in_samples(
    &self,
    ports: &mut Ports,
    step_duration_in_samples: f32,
  ) -> i64 {
    let step_is_an_even_number = self.current_step & 1 == 0;
    if step_is_an_even_number {
      0
    } else {
      (ports.swing.get() * 0.5 * step_duration_in_samples).round() as i64
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

  pub fn update_position(&mut self, object_reader: ObjectReader<'static>) {
    for (property_header, property) in object_reader {
      if property_header.key == self.urids.time.beats_per_minute {
        self.host_bpm = property.read(self.urids.atom.float, ()).unwrap_or(120.);
      }
      if property_header.key == self.urids.time.speed {
        self.host_speed = property.read(self.urids.atom.float, ()).unwrap_or(0.);
      }
      if property_header.key == self.urids.time.beat_unit {
        self.beat_unit = property.read(self.urids.atom.int, ()).unwrap_or(4);
      }
      if property_header.key == self.urids.time.bar_beat {
        self.beat = property
          .read(self.urids.atom.float, ())
          .map_or(0., |beat| beat.fract());
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

  fn map_current_step_to_reordered_step(&mut self, order: u8, steps: usize) -> usize {
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
}
