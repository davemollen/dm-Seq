use std::collections::VecDeque;
use wmidi::{Channel, MidiMessage, Note, Velocity};

#[derive(Clone)]
pub struct MidiEvent {
  pub samples_until: i64,
  pub data: MidiMessage<'static>,
}

pub struct EventQueue {
  queue: VecDeque<MidiEvent>,
  next_note_off: Option<MidiMessage<'static>>,
}

impl EventQueue {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
      next_note_off: None,
    }
  }

  pub fn schedule_note(
    &mut self,
    channel: u8,
    note: u8,
    velocity: u8,
    start_in_samples: i64,
    length_in_samples: i64,
    legato_mode: bool,
  ) {
    let channel = Channel::from_index(channel).unwrap();
    let note = Note::try_from(note).unwrap();
    let velocity = Velocity::try_from(velocity).unwrap();

    if legato_mode {
      // Try to extend an existing NoteOff for this note in legato mode
      if self.extend_note_off(channel, note, length_in_samples) {
        return;
      }
    } else {
      // Collect all note off messages for the retriggered note
      let note_offs: Vec<Velocity> = self
        .queue
        .iter()
        .filter_map(|ev| {
          if let MidiMessage::NoteOff(ch, n, vel) = ev.data {
            if ch == channel && n == note {
              Some(vel)
            } else {
              None
            }
          } else {
            None
          }
        })
        .collect();

      // Remove the old note from the queue
      self.queue.retain(
        |ev| !matches!(ev.data, MidiMessage::NoteOff(ch, n, _) if ch == channel && n == note),
      );

      // Send note off immediately
      note_offs.iter().for_each(|vel| {
        self.push(0, MidiMessage::NoteOff(channel, note, *vel));
      });
    }

    // Schedule NoteOn
    self.push(
      start_in_samples,
      MidiMessage::NoteOn(channel, note, velocity),
    );

    // Schedule NoteOff
    self.push(
      start_in_samples + length_in_samples,
      MidiMessage::NoteOff(channel, note, velocity),
    );
  }

  pub fn schedule_triggered_note(
    &mut self,
    channel: u8,
    note: u8,
    velocity: u8,
    is_note_on: bool,
    legato_mode: bool,
  ) {
    let note = Note::try_from(note).unwrap();
    let channel = Channel::from_index(channel).unwrap();
    let velocity = Velocity::try_from(velocity).unwrap();

    if legato_mode {
      let note_is_already_active = self.next_note_off.clone().map_or(
        false,
        |ev| matches!(ev, MidiMessage::NoteOff(ch, n, _) if ch == channel && n == note),
      );

      // Do nothing when note is already active in legato mode
      if note_is_already_active {
        return;
      }
    }

    // Send note off for previously sent note
    if let Some(midi_message) = &self.next_note_off {
      self.push(0, midi_message.clone());
    }

    if is_note_on {
      // Schedule NoteOn
      self.push(0, MidiMessage::NoteOn(channel, note, velocity));

      // Store NoteOff to trigger later
      self.next_note_off = Some(MidiMessage::NoteOff(channel, note, Velocity::MIN));
    } else {
      // No note to turn off
      self.next_note_off = None;
    }
  }

  pub fn push(&mut self, samples_from_now: i64, data: MidiMessage<'static>) {
    let ev = MidiEvent {
      samples_until: samples_from_now,
      data,
    };
    let pos = self
      .queue
      .iter()
      .position(|e| e.samples_until > samples_from_now);
    if let Some(idx) = pos {
      self.queue.insert(idx, ev);
    } else {
      self.queue.push_back(ev);
    }
  }

  /// Process the next block, yielding events that occur within it.
  ///
  /// `block_size` = number of samples in this run()
  pub fn pop_block_events(&mut self, block_size: i64) -> Vec<(i64, MidiMessage<'static>)> {
    let mut result = Vec::new();

    for ev in self.queue.iter_mut() {
      if ev.samples_until < block_size {
        // Event lands in this block
        result.push((ev.samples_until as i64, ev.data.clone()));
        ev.samples_until = i64::MAX; // Mark for removal
      } else {
        // Event still in the future: decrement counter
        ev.samples_until -= block_size;
      }
    }

    // Remove all marked events
    self.queue.retain(|ev| ev.samples_until != i64::MAX);

    result
  }

  /// Send all NoteOff events immediately and remove everything else from the queue
  pub fn stop_all_notes(&mut self) {
    self.queue.retain_mut(|ev| {
      if let MidiMessage::NoteOff(_, _, _) = ev.data {
        ev.samples_until = 0;
        true
      } else {
        false // remove non NoteOff events
      }
    });
  }

  pub fn clear(&mut self) {
    self.queue.clear();
  }

  /// Extend the scheduled NoteOff for the given note by extra_samples.
  fn extend_note_off(&mut self, channel: Channel, note: Note, extra_samples: i64) -> bool {
    if let Some(ev) = self
      .queue
      .iter_mut()
      .find(|ev| matches!(ev.data, MidiMessage::NoteOff(ch, n, _) if ch == channel && n == note))
    {
      ev.samples_until += extra_samples;
      true
    } else {
      false
    }
  }
}
