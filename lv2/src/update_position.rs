use crate::DmSeq;
use lv2::lv2_atom::object::ObjectReader;

impl DmSeq {
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
}
