mod event_queue;
mod utils;
use {
  crate::{event_queue::EventQueue, utils::SequencerData},
  lv2::prelude::*,
  std::ffi::CStr,
  wmidi::{Channel, Note, Velocity},
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

#[derive(FeatureCollection)]
pub struct AudioFeatures<'a> {
  log: Log<'a>,
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
  host_bpm: f32,
  host_speed: f32,
  beat_unit: i32,
  beat: f32,
  block_start_frame: i64,
  next_step_frame: i64,
  free_running_block_start_frame: i64,
  free_running_next_step_frame: i64,
  is_initialized: bool,
  shuffled_steps: Vec<usize>,
  is_in_swing_cycle: bool,
  prev_steps: usize,
  event_queue: EventQueue,
  sample_rate: f32,
}

impl Plugin for DmSeq {
  type Ports = Ports;
  type InitFeatures = InitFeatures<'static>;
  type AudioFeatures = AudioFeatures<'static>;

  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      current_step: 15,
      urids: features.map.populate_collection()?,
      host_bpm: 120.,
      host_speed: 0.,
      beat_unit: 4,
      beat: 0.,
      block_start_frame: 0,
      next_step_frame: 0,
      free_running_block_start_frame: 0,
      free_running_next_step_frame: 0,
      is_initialized: false,
      shuffled_steps: Vec::with_capacity(16),
      is_in_swing_cycle: true,
      prev_steps: 8,
      event_queue: EventQueue::new(),
      sample_rate,
    })
  }

  fn run(&mut self, ports: &mut Ports, features: &mut Self::AudioFeatures, sample_count: u32) {
    if !self.is_initialized {
      self.set_shuffled_steps(ports.steps.get() as usize);
      self.is_initialized = true;
      self.prev_steps = ports.steps.get() as usize;
    }

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

    if ports.clock_mode.get() == 1. && self.host_speed == 0. {
      self.handle_transport_stopped(ports);
    } else if ports.panic.get() == 1. {
      let mut midi_out_sequence = match ports.midi_out.init(
        self.urids.atom.sequence,
        TimeStampURID::Frames(self.urids.unit.frame),
      ) {
        Some(sequence_iter) => sequence_iter,
        None => return,
      };
      self.midi_panic(&mut midi_out_sequence);
      self.event_queue.clear();
    } else {
      match ports.clock_mode.get() {
        1. => {
          while self.next_step_frame < self.block_start_frame + sample_count as i64 {
            let next_step = self.current_step + 1;
            self.current_step = if next_step >= ports.steps.get() as usize {
              0
            } else {
              next_step
            };

            let reordered_step = self.map_current_step_to_reordered_step(
              ports.order.get() as u8,
              ports.steps.get() as usize,
            );
            let current_note = notes[reordered_step];
            let current_velocity = velocities[reordered_step];
            let current_gate = gates[reordered_step];
            let has_note_on = current_velocity > 0 && current_gate;
            ports.current_step.set(reordered_step as f32);

            let samples_per_beat = self.sample_rate * 60.0 / self.host_bpm;
            let division =
              self.map_step_duration_to_divisor(ports.step_duration.get()) / self.beat_unit as f32;

            let phase = (self.beat * division).fract();
            features
              .log
              .print_cstr(
                self.urids.log.note,
                CStr::from_bytes_with_nul(format!("beat phase: {}\n\0", phase).as_bytes()).unwrap(),
              )
              .ok();
            let offset_phase = if phase > 0.5 { 1. - phase } else { -phase };
            let step_in_samples = samples_per_beat * division.recip();
            let step_offset_in_samples = offset_phase * step_in_samples;
            self.next_step_frame =
              (self.block_start_frame as f32 + step_in_samples + step_offset_in_samples).round()
                as i64;
            let step_is_an_even_number = self.current_step & 1 == 0;
            let swing_offset_in_samples = if step_is_an_even_number {
              0
            } else {
              (ports.swing.get() * 0.5 * step_in_samples as f32).round() as i64
            };

            if ports.enable.get() == 0. {
              self.event_queue.stop_all_notes();
            } else if has_note_on {
              let note = Note::try_from(current_note).unwrap();
              let channel = Channel::from_index(ports.midi_channel.get() as u8).unwrap();
              let velocity = Velocity::try_from(current_velocity).unwrap();

              self.event_queue.schedule_note(
                channel,
                note,
                velocity,
                swing_offset_in_samples,
                step_in_samples.round() as i64,
                ports.repeat_mode.get() == 0.,
              );
            }
          }
        }
        2. => {
          while self.free_running_next_step_frame
            < self.free_running_block_start_frame + sample_count as i64
          {
            let next_step = self.current_step + 1;
            self.current_step = if next_step >= ports.steps.get() as usize {
              0
            } else {
              next_step
            };

            let reordered_step = self.map_current_step_to_reordered_step(
              ports.order.get() as u8,
              ports.steps.get() as usize,
            );
            let current_note = notes[reordered_step];
            let current_velocity = velocities[reordered_step];
            let current_gate = gates[reordered_step];
            let has_note_on = current_velocity > 0 && current_gate;
            ports.current_step.set(reordered_step as f32);

            let samples_per_beat = self.sample_rate * 60.0 / ports.bpm.get();
            let division = self.map_step_duration_to_divisor(ports.step_duration.get()) / 4.;
            let step_in_samples = (samples_per_beat * division.recip()).round() as i64;
            self.free_running_next_step_frame =
              self.free_running_block_start_frame + step_in_samples;
            let step_is_an_even_number = self.current_step & 1 == 0;
            let swing_offset_in_samples = if step_is_an_even_number {
              0
            } else {
              (ports.swing.get() * 0.5 * step_in_samples as f32).round() as i64
            };

            if ports.enable.get() == 0. {
              self.event_queue.stop_all_notes();
            } else if has_note_on {
              let note = Note::try_from(current_note).unwrap();
              let channel = Channel::from_index(ports.midi_channel.get() as u8).unwrap();
              let velocity = Velocity::try_from(current_velocity).unwrap();

              self.event_queue.schedule_note(
                channel,
                note,
                velocity,
                swing_offset_in_samples,
                step_in_samples,
                ports.repeat_mode.get() == 0.,
              );
            }
          }
          self.free_running_block_start_frame += sample_count as i64;
        }
        _ => {
          if ports.trigger.get() == 1. {
            let next_step = self.current_step + 1;
            self.current_step = if next_step >= ports.steps.get() as usize {
              0
            } else {
              next_step
            };

            let reordered_step = self.map_current_step_to_reordered_step(
              ports.order.get() as u8,
              ports.steps.get() as usize,
            );
            let current_note = notes[reordered_step];
            let current_velocity = velocities[reordered_step];
            let current_gate = gates[reordered_step];
            let has_note_on = current_velocity > 0 && current_gate;
            ports.current_step.set(reordered_step as f32);

            if ports.enable.get() == 0. {
              self.event_queue.stop_all_notes();
            }
            let note = Note::try_from(current_note).unwrap();
            let channel = Channel::from_index(ports.midi_channel.get() as u8).unwrap();
            let velocity = Velocity::try_from(current_velocity).unwrap();

            self.event_queue.schedule_triggered_note(
              channel,
              note,
              velocity,
              has_note_on,
              ports.repeat_mode.get() == 0.,
            );
          }
        }
      }
    }

    let mut midi_out_sequence = match ports.midi_out.init(
      self.urids.atom.sequence,
      TimeStampURID::Frames(self.urids.unit.frame),
    ) {
      Some(sequence_iter) => sequence_iter,
      None => return,
    };
    let events = self.event_queue.pop_block_events(sample_count as i64);
    for (offset, midi_message) in events {
      match midi_out_sequence.init(
        TimeStamp::Frames(offset),
        self.urids.midi.wmidi,
        midi_message,
      ) {
        None => return,
        _ => (),
      };
    }
  }

  fn activate(&mut self, _features: &mut Self::InitFeatures) {
    self.is_initialized = false;
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmSeq);
