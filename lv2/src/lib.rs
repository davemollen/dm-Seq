mod delta;
mod phasor;
mod synced_phasor;
mod utils;
use {
  crate::{delta::Delta, phasor::Phasor, utils::SequencerData},
  lv2::prelude::*,
  synced_phasor::SyncedPhasor,
  wmidi::{Channel, MidiMessage, Note, Velocity},
};

#[derive(PortCollection)]
struct Ports {
  trigger: InputPort<Control>,
  steps: InputPort<Control>,
  swing: InputPort<Control>,
  step_duration: InputPort<Control>,
  clock_mode: InputPort<Control>,
  order: InputPort<Control>,
  repeat_mode: InputPort<Control>,
  _knob_target: InputPort<Control>,
  bpm: InputPort<Control>,
  note_1: InputPort<Control>,
  note_2: InputPort<Control>,
  note_3: InputPort<Control>,
  note_4: InputPort<Control>,
  note_5: InputPort<Control>,
  note_6: InputPort<Control>,
  note_7: InputPort<Control>,
  note_8: InputPort<Control>,
  note_9: InputPort<Control>,
  note_10: InputPort<Control>,
  note_11: InputPort<Control>,
  note_12: InputPort<Control>,
  note_13: InputPort<Control>,
  note_14: InputPort<Control>,
  note_15: InputPort<Control>,
  note_16: InputPort<Control>,
  velocity_1: InputPort<Control>,
  velocity_2: InputPort<Control>,
  velocity_3: InputPort<Control>,
  velocity_4: InputPort<Control>,
  velocity_5: InputPort<Control>,
  velocity_6: InputPort<Control>,
  velocity_7: InputPort<Control>,
  velocity_8: InputPort<Control>,
  velocity_9: InputPort<Control>,
  velocity_10: InputPort<Control>,
  velocity_11: InputPort<Control>,
  velocity_12: InputPort<Control>,
  velocity_13: InputPort<Control>,
  velocity_14: InputPort<Control>,
  velocity_15: InputPort<Control>,
  velocity_16: InputPort<Control>,
  gate_1: InputPort<Control>,
  gate_2: InputPort<Control>,
  gate_3: InputPort<Control>,
  gate_4: InputPort<Control>,
  gate_5: InputPort<Control>,
  gate_6: InputPort<Control>,
  gate_7: InputPort<Control>,
  gate_8: InputPort<Control>,
  gate_9: InputPort<Control>,
  gate_10: InputPort<Control>,
  gate_11: InputPort<Control>,
  gate_12: InputPort<Control>,
  gate_13: InputPort<Control>,
  gate_14: InputPort<Control>,
  gate_15: InputPort<Control>,
  gate_16: InputPort<Control>,
  midi_channel: InputPort<Control>,
  current_step: OutputPort<Control>,
  control: InputPort<AtomPort>,
  midi_out: OutputPort<AtomPort>,
}

#[derive(FeatureCollection)]
pub struct AudioFeatures<'a> {
  log: Log<'a>,
}

#[derive(FeatureCollection)]
pub struct InitFeatures<'a> {
  map: LV2Map<'a>,
}

#[derive(URIDCollection)]
pub struct URIDs {
  atom: AtomURIDCollection,
  midi: MidiURIDCollection,
  unit: UnitURIDCollection,
  time: TimeURIDCollection,
  log: LogURIDCollection,
}

#[uri("https://github.com/davemollen/dm-Seq")]
struct DmSeq {
  current_step: usize,
  urids: URIDs,
  prev_current_note: Option<u8>,
  host_div: i32,
  host_bpm: f32,
  host_speed: f32,
  beat: f32,
  step_progress_phasor: SyncedPhasor,
  step_progress_delta: Delta,
  phasor: Phasor,
  is_initialized: bool,
  shuffled_steps: Vec<usize>,
  is_in_swing_cycle: bool,
  swing_delta: Delta,
  prev_steps: usize,
}

impl Plugin for DmSeq {
  type Ports = Ports;
  type InitFeatures = InitFeatures<'static>;
  type AudioFeatures = AudioFeatures<'static>;

  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      current_step: 15,
      prev_current_note: None,
      urids: features.map.populate_collection()?,
      host_div: 4,
      host_bpm: 120.,
      host_speed: 0.,
      beat: 0.,
      step_progress_phasor: SyncedPhasor::new(),
      step_progress_delta: Delta::new(),
      phasor: Phasor::new(sample_rate),
      is_initialized: false,
      shuffled_steps: Vec::with_capacity(16),
      is_in_swing_cycle: true,
      swing_delta: Delta::new(),
      prev_steps: 8,
    })
  }

  fn run(&mut self, ports: &mut Ports, _features: &mut Self::AudioFeatures, sample_count: u32) {
    let SequencerData {
      notes,
      velocities,
      gates,
    } = self.map_sequencer_data(ports);

    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence, self.urids.unit.beat)
    {
      Some(sequence_iter) => sequence_iter,
      None => return,
    };

    for (_time_stamp, atom) in control_sequence {
      if let Some((object_header, object_reader)) = atom
        .read(self.urids.atom.object, ())
        .or_else(|| atom.read(self.urids.atom.blank, ()))
      {
        if object_header.otype == self.urids.time.position_class {
          self.update_position(object_reader);
        }
      }
    }

    if !self.is_initialized {
      let speed = self.map_step_duration_to_divisor(*ports.step_duration) / self.host_div as f32;
      self.step_progress_phasor.set_initial_speed(speed);
      self.set_shuffled_steps(*ports.steps as usize);
      self.is_initialized = true;
      self.prev_steps = *ports.steps as usize;
    }

    if self.host_speed == 0. {
      self.handle_transport_stopped(ports);
      return;
    }

    let trigger = match *ports.clock_mode {
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
    };

    if trigger {
      let next_step = self.current_step + 1;
      self.current_step = if next_step >= *ports.steps as usize {
        0
      } else {
        next_step
      };

      let reordered_step =
        self.map_current_step_to_reordered_step(*ports.order as u8, *ports.steps as usize);
      let current_note = notes[reordered_step];
      let current_velocity = velocities[reordered_step];
      let current_gate = gates[reordered_step];
      let has_note_on = current_velocity > 0 && current_gate;
      **ports.current_step = reordered_step as f32;

      // skip repeated midi note if in legato mode
      if *ports.repeat_mode == 0.
        && has_note_on
        && self
          .prev_current_note
          .map_or(false, |prev_current_note| current_note == prev_current_note)
      {
        return;
      }

      let mut midi_out_sequence = match ports.midi_out.init(
        self.urids.atom.sequence,
        TimeStampURID::Frames(self.urids.unit.frame),
      ) {
        Some(sequence_iter) => sequence_iter,
        None => return,
      };

      if let Some(prev_current_note) = self.prev_current_note {
        midi_out_sequence
          .init(
            TimeStamp::Frames(0),
            self.urids.midi.wmidi,
            MidiMessage::NoteOff(
              Channel::from_index(*ports.midi_channel as u8).unwrap(),
              Note::try_from(prev_current_note).unwrap(),
              Velocity::try_from(0).unwrap(),
            ),
          )
          .unwrap();
      }
      if has_note_on {
        midi_out_sequence
          .init(
            TimeStamp::Frames(0),
            self.urids.midi.wmidi,
            MidiMessage::NoteOn(
              Channel::from_index(*ports.midi_channel as u8).unwrap(),
              Note::try_from(current_note).unwrap(),
              Velocity::try_from(current_velocity).unwrap(),
            ),
          )
          .unwrap();
        self.prev_current_note = Some(current_note);
      }
    }
  }

  fn activate(&mut self, _features: &mut Self::InitFeatures) {
    self.is_initialized = false;
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmSeq);
