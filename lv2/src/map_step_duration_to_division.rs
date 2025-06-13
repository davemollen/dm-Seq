use crate::DmSeq;

impl DmSeq {
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
}
