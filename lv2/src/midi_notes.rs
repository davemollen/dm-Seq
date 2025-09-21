pub struct MidiNotes {
  note: Option<u8>,
  note_queue: Vec<u8>,
  note_off_queue: Vec<u8>,
  is_sustained: bool,
}

impl MidiNotes {
  pub fn new() -> Self {
    Self {
      note: None,
      note_queue: Vec::with_capacity(128),
      note_off_queue: Vec::with_capacity(128),
      is_sustained: false,
    }
  }

  pub fn get_note(&mut self) -> Option<u8> {
    self.note
  }

  pub fn note_on(&mut self, note: u8) {
    self.note_queue.push(note);
    self.note = Some(note);
  }

  pub fn note_off(&mut self, note: u8) {
    if self.is_sustained {
      self.note_off_queue.push(note);
      return;
    }
    self.note_queue.retain(|n| *n != note);

    if self.note_queue.len() == 0 {
      self.note = None;
    } else {
      self.note = Some(self.note_queue[self.note_queue.len() - 1]);
    }
  }

  pub fn sustain(&mut self, sustain: bool) {
    let prev_is_sustained = self.is_sustained;
    self.is_sustained = sustain;

    if prev_is_sustained && !self.is_sustained {
      let notes_to_off = std::mem::take(&mut self.note_off_queue);
      for note in notes_to_off {
        self.note_off(note);
      }
    }
  }

  pub fn remove_notes(&mut self) {
    self.note = None;
    self.note_queue.clear();
    self.note_off_queue.clear();
  }
}
