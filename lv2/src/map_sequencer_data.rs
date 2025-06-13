use crate::{DmSeq, Ports};

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
}
