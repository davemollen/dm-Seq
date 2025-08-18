mod event_queue;
mod synced_phasor;
mod utils;
use {
  event_queue::EventQueue,
  lv2::prelude::*,
  std::{array, usize},
  synced_phasor::SyncedPhasor,
  utils::NextStep,
};

#[derive(PortCollection)]
struct Ports {
  enable: InputPort<InPlaceControl>,
  trigger: InputPort<InPlaceControl>,
  steps: InputPort<InPlaceControl>,
  step_offset: InputPort<InPlaceControl>,
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
  note_length_1: InputPort<InPlaceControl>,
  note_length_2: InputPort<InPlaceControl>,
  note_length_3: InputPort<InPlaceControl>,
  note_length_4: InputPort<InPlaceControl>,
  note_length_5: InputPort<InPlaceControl>,
  note_length_6: InputPort<InPlaceControl>,
  note_length_7: InputPort<InPlaceControl>,
  note_length_8: InputPort<InPlaceControl>,
  note_length_9: InputPort<InPlaceControl>,
  note_length_10: InputPort<InPlaceControl>,
  note_length_11: InputPort<InPlaceControl>,
  note_length_12: InputPort<InPlaceControl>,
  note_length_13: InputPort<InPlaceControl>,
  note_length_14: InputPort<InPlaceControl>,
  note_length_15: InputPort<InPlaceControl>,
  note_length_16: InputPort<InPlaceControl>,
  channel_1: InputPort<InPlaceControl>,
  channel_2: InputPort<InPlaceControl>,
  channel_3: InputPort<InPlaceControl>,
  channel_4: InputPort<InPlaceControl>,
  channel_5: InputPort<InPlaceControl>,
  channel_6: InputPort<InPlaceControl>,
  channel_7: InputPort<InPlaceControl>,
  channel_8: InputPort<InPlaceControl>,
  channel_9: InputPort<InPlaceControl>,
  channel_10: InputPort<InPlaceControl>,
  channel_11: InputPort<InPlaceControl>,
  channel_12: InputPort<InPlaceControl>,
  channel_13: InputPort<InPlaceControl>,
  channel_14: InputPort<InPlaceControl>,
  channel_15: InputPort<InPlaceControl>,
  channel_16: InputPort<InPlaceControl>,
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
  host_bpm: f64,
  host_speed: f32,
  beat_unit: i32,
  block_start_frame: i64,
  next_step_frame: i64,
  free_running_block_start_frame: i64,
  free_running_next_step_frame: i64,
  is_initialized: bool,
  shuffled_steps: [usize; 16],
  last_shuffled_step: usize,
  is_in_swing_cycle: bool,
  should_alternate_sequence: bool,
  prev_steps: usize,
  prev_clock_mode: f32,
  event_queue: EventQueue,
  synced_phasor: SyncedPhasor,
  sample_rate: f32,
}

impl Plugin for DmSeq {
  type Ports = Ports;
  type InitFeatures = InitFeatures<'static>;
  type AudioFeatures = ();

  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      current_step: 15,
      urids: features.map.populate_collection()?,
      host_bpm: 120.,
      host_speed: 0.,
      beat_unit: 4,
      block_start_frame: 0,
      next_step_frame: 0,
      free_running_block_start_frame: 0,
      free_running_next_step_frame: 0,
      is_initialized: false,
      shuffled_steps: array::from_fn(|i| i),
      last_shuffled_step: usize::MAX,
      is_in_swing_cycle: true,
      should_alternate_sequence: true,
      prev_steps: 8,
      prev_clock_mode: 0.,
      event_queue: EventQueue::new(),
      synced_phasor: SyncedPhasor::new(),
      sample_rate,
    })
  }

  fn run(&mut self, ports: &mut Ports, _features: &mut Self::AudioFeatures, sample_count: u32) {
    if !self.is_initialized {
      self.set_shuffled_steps(ports.steps.get() as usize, false);
      self.is_initialized = true;
      self.prev_steps = ports.steps.get() as usize;
      self.prev_clock_mode = ports.clock_mode.get();
    }

    let sequencer_data = self.map_sequencer_data(ports);

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

    if ports.clock_mode.get() == 0. && self.host_speed == 0. {
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
        0. => {
          // clock_mode == Host Sync
          while self.next_step_frame < self.block_start_frame + sample_count as i64 {
            let NextStep {
              note,
              velocity,
              channel,
              note_length,
              step,
              is_note_on,
            } = self.resolve_next_step(ports, &sequencer_data);

            let division =
              self.map_step_duration_to_divisor(ports.step_duration.get()) / self.beat_unit as f64;
            let step_duration_in_samples =
              self.get_step_duration_in_samples(self.host_bpm, division);
            let swing_offset_in_samples =
              self.get_swing_offset_in_samples(ports, step_duration_in_samples);

            let phase = (self.synced_phasor.get_value() * division * 4.).fract();
            let (step_offset_in_samples, sync_offset_in_samples) = if self.prev_clock_mode != 0. {
              (0., (1. - phase) * step_duration_in_samples)
            } else {
              (
                if phase > 0.5 { 1. - phase } else { -phase } * step_duration_in_samples,
                0.,
              )
            };
            self.next_step_frame = (self.block_start_frame as f64
              + step_duration_in_samples
              + step_offset_in_samples
              + sync_offset_in_samples)
              .round() as i64;

            if ports.enable.get() == 0. {
              self.event_queue.stop_all_notes();
            } else {
              self.event_queue.schedule_note(
                channel,
                note,
                velocity,
                step,
                is_note_on,
                (sync_offset_in_samples + swing_offset_in_samples).round() as i64,
                (step_duration_in_samples * note_length
                  + step_offset_in_samples
                  + sync_offset_in_samples)
                  .round() as i64,
                ports.repeat_mode.get() == 0.,
              );
            }
          }
        }
        1. => {
          // clock_mode == Free Running
          while self.free_running_next_step_frame
            < self.free_running_block_start_frame + sample_count as i64
          {
            let NextStep {
              note,
              velocity,
              channel,
              note_length,
              step,
              is_note_on,
            } = self.resolve_next_step(ports, &sequencer_data);

            let division = self.map_step_duration_to_divisor(ports.step_duration.get()) / 4.;
            let step_duration_in_samples =
              self.get_step_duration_in_samples(ports.bpm.get() as f64, division);
            let start_in_samples =
              self.get_swing_offset_in_samples(ports, step_duration_in_samples);

            self.free_running_next_step_frame =
              self.free_running_block_start_frame + step_duration_in_samples.round() as i64;

            if ports.enable.get() == 0. {
              self.event_queue.stop_all_notes();
            } else {
              self.event_queue.schedule_note(
                channel,
                note,
                velocity,
                step,
                is_note_on,
                start_in_samples.round() as i64,
                (step_duration_in_samples * note_length).round() as i64,
                ports.repeat_mode.get() == 0.,
              );
            }
          }
          self.free_running_block_start_frame += sample_count as i64;
        }
        _ => {
          // clock_mode == Trigger
          if ports.enable.get() == 0. {
            self.event_queue.stop_triggered_note();
          }

          if ports.trigger.get() == 1. {
            let NextStep {
              note,
              velocity,
              channel,
              step,
              is_note_on,
              ..
            } = self.resolve_next_step(ports, &sequencer_data);

            if ports.enable.get() != 0. {
              self.event_queue.schedule_triggered_note(
                channel,
                note,
                velocity,
                step,
                is_note_on,
                ports.repeat_mode.get() == 0.,
              );
            }
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
    for (offset, midi_message, step) in events {
      if let Some(step) = step {
        ports.current_step.set(step as f32);
      }

      if let Some(midi_message) = midi_message {
        match midi_out_sequence.init(
          TimeStamp::Frames(offset),
          self.urids.midi.wmidi,
          midi_message,
        ) {
          None => continue,
          _ => (),
        };
      }
    }

    self.prev_clock_mode = ports.clock_mode.get();
  }

  fn activate(&mut self, _features: &mut Self::InitFeatures) {
    self.is_initialized = false;
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmSeq);
