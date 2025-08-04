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
  enable: InputPort<InPlaceControl>,
  trigger: InputPort<InPlaceControl>,
  steps: InputPort<InPlaceControl>,
  swing: InputPort<InPlaceControl>,
  step_duration: InputPort<InPlaceControl>,
  clock_mode: InputPort<InPlaceControl>,
  order: InputPort<InPlaceControl>,
  repeat_mode: InputPort<InPlaceControl>,
  _knob_target: InputPort<InPlaceControl>,
  bpm: InputPort<InPlaceControl>,
  note_1: InputPort<InPlaceControl>,
  note_2: InputPort<InPlaceControl>,
  note_3: InputPort<InPlaceControl>,
  note_4: InputPort<InPlaceControl>,
  note_5: InputPort<InPlaceControl>,
  note_6: InputPort<InPlaceControl>,
  note_7: InputPort<InPlaceControl>,
  note_8: InputPort<InPlaceControl>,
  note_9: InputPort<InPlaceControl>,
  note_10: InputPort<InPlaceControl>,
  note_11: InputPort<InPlaceControl>,
  note_12: InputPort<InPlaceControl>,
  note_13: InputPort<InPlaceControl>,
  note_14: InputPort<InPlaceControl>,
  note_15: InputPort<InPlaceControl>,
  note_16: InputPort<InPlaceControl>,
  velocity_1: InputPort<InPlaceControl>,
  velocity_2: InputPort<InPlaceControl>,
  velocity_3: InputPort<InPlaceControl>,
  velocity_4: InputPort<InPlaceControl>,
  velocity_5: InputPort<InPlaceControl>,
  velocity_6: InputPort<InPlaceControl>,
  velocity_7: InputPort<InPlaceControl>,
  velocity_8: InputPort<InPlaceControl>,
  velocity_9: InputPort<InPlaceControl>,
  velocity_10: InputPort<InPlaceControl>,
  velocity_11: InputPort<InPlaceControl>,
  velocity_12: InputPort<InPlaceControl>,
  velocity_13: InputPort<InPlaceControl>,
  velocity_14: InputPort<InPlaceControl>,
  velocity_15: InputPort<InPlaceControl>,
  velocity_16: InputPort<InPlaceControl>,
  gate_1: InputPort<InPlaceControl>,
  gate_2: InputPort<InPlaceControl>,
  gate_3: InputPort<InPlaceControl>,
  gate_4: InputPort<InPlaceControl>,
  gate_5: InputPort<InPlaceControl>,
  gate_6: InputPort<InPlaceControl>,
  gate_7: InputPort<InPlaceControl>,
  gate_8: InputPort<InPlaceControl>,
  gate_9: InputPort<InPlaceControl>,
  gate_10: InputPort<InPlaceControl>,
  gate_11: InputPort<InPlaceControl>,
  gate_12: InputPort<InPlaceControl>,
  gate_13: InputPort<InPlaceControl>,
  gate_14: InputPort<InPlaceControl>,
  gate_15: InputPort<InPlaceControl>,
  gate_16: InputPort<InPlaceControl>,
  midi_channel: InputPort<InPlaceControl>,
  panic: InputPort<InPlaceControl>,
  current_step: OutputPort<InPlaceControl>,
  control: InputPort<AtomPort>,
  midi_out: OutputPort<AtomPort>,
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
}

#[uri("https://github.com/davemollen/dm-Seq")]
struct DmSeq {
  current_step: usize,
  urids: URIDs,
  prev_note: Option<u8>,
  host_speed: f32,
  beat_unit: i32,
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
  type AudioFeatures = ();

  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      current_step: 15,
      prev_note: None,
      urids: features.map.populate_collection()?,
      host_speed: 0.,
      beat_unit: 4,
      beat: 0.,
      step_progress_phasor: SyncedPhasor::new(),
      step_progress_delta: Delta::new(1.),
      phasor: Phasor::new(sample_rate),
      is_initialized: false,
      shuffled_steps: Vec::with_capacity(16),
      is_in_swing_cycle: true,
      swing_delta: Delta::new(1.),
      prev_steps: 8,
    })
  }

  fn run(&mut self, ports: &mut Ports, _features: &mut Self::AudioFeatures, sample_count: u32) {
    if !self.is_initialized {
      let speed =
        self.map_step_duration_to_divisor(ports.step_duration.get()) / self.beat_unit as f32;
      self.step_progress_phasor.set_initial_speed(speed);
      self.set_shuffled_steps(ports.steps.get() as usize);
      self.is_initialized = true;
      self.prev_steps = ports.steps.get() as usize;
    }

    let SequencerData {
      notes,
      velocities,
      gates,
    } = self.map_sequencer_data(ports);
    let trigger = self.get_trigger(ports, sample_count);

    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence, self.urids.unit.beat)
    {
      Some(sequence_iter) => sequence_iter,
      None => return,
    };

    let mut midi_out_sequence = match ports.midi_out.init(
      self.urids.atom.sequence,
      TimeStampURID::Frames(self.urids.unit.frame),
    ) {
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

    if ports.clock_mode.get() == 1. && self.host_speed == 0. {
      self.handle_transport_stopped(ports);
      return;
    }

    if ports.panic.get() == 1. {
      self.midi_panic(&mut midi_out_sequence);
    }

    if trigger {
      let next_step = self.current_step + 1;
      self.current_step = if next_step >= ports.steps.get() as usize {
        0
      } else {
        next_step
      };

      let reordered_step = self
        .map_current_step_to_reordered_step(ports.order.get() as u8, ports.steps.get() as usize);
      let current_note = notes[reordered_step];
      let current_velocity = velocities[reordered_step];
      let current_gate = gates[reordered_step];
      let has_note_on = current_velocity > 0 && current_gate;
      ports.current_step.set(reordered_step as f32);

      if ports.enable.get() == 0. {
        if let Some(prev_note) = self.prev_note {
          midi_out_sequence
            .init(
              TimeStamp::Frames(0),
              self.urids.midi.wmidi,
              MidiMessage::NoteOff(
                Channel::from_index(ports.midi_channel.get() as u8).unwrap(),
                Note::try_from(prev_note).unwrap(),
                Velocity::try_from(0).unwrap(),
              ),
            )
            .unwrap();
          self.prev_note = None;
        }
        return;
      }

      // skip repeated midi note if in legato mode
      if ports.repeat_mode.get() == 0.
        && has_note_on
        && self
          .prev_note
          .map_or(false, |prev_note| current_note == prev_note)
      {
        return;
      }

      if let Some(prev_note) = self.prev_note {
        midi_out_sequence
          .init(
            TimeStamp::Frames(0),
            self.urids.midi.wmidi,
            MidiMessage::NoteOff(
              Channel::from_index(ports.midi_channel.get() as u8).unwrap(),
              Note::try_from(prev_note).unwrap(),
              Velocity::try_from(0).unwrap(),
            ),
          )
          .unwrap();
        self.prev_note = None;
      }
      if has_note_on {
        midi_out_sequence
          .init(
            TimeStamp::Frames(0),
            self.urids.midi.wmidi,
            MidiMessage::NoteOn(
              Channel::from_index(ports.midi_channel.get() as u8).unwrap(),
              Note::try_from(current_note).unwrap(),
              Velocity::try_from(current_velocity).unwrap(),
            ),
          )
          .unwrap();
        self.prev_note = Some(current_note);
      }
    }
  }

  fn activate(&mut self, _features: &mut Self::InitFeatures) {
    self.is_initialized = false;
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmSeq);
