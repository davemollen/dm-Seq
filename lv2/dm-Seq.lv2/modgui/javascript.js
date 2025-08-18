function(event) {
  function handle_port_values(symbol, value) {
    switch (symbol) {
      case "clock_mode": {
        const trigger = event.icon.find("[mod-port-symbol=trigger]").parent();
        const bpm = event.icon.find("[mod-port-symbol=bpm]").parent();
        const note_length_knobs = event.icon.find('[mod-port-symbol^="note_length_"]').parent();
        const length_tab = event.icon.find(".mod-tab.length .tab-content");
        if(value == 0) {
          // clock_mode == Host Sync
          trigger.addClass("disabled");
          bpm.addClass("disabled");
          note_length_knobs.removeClass("disabled");
          length_tab.removeClass("disabled");
        } else if(value == 1) {
          // clock_mode == Free Running
          trigger.addClass("disabled");
          bpm.removeClass("disabled");
          note_length_knobs.removeClass("disabled");
          length_tab.removeClass("disabled");
        } else if(value == 2) {
          // clock_mode == Trigger
          trigger.removeClass("disabled");
          bpm.addClass("disabled");
          note_length_knobs.addClass("disabled");
          length_tab.addClass("disabled");
        }
        break;
      }
      case "knob_target": {
        const notes = event.icon.find("#notes");
        const velocities = event.icon.find("#velocities");
        const note_lengths = event.icon.find("#note-lengths");
        const channels = event.icon.find("#channels");
        
        [notes, velocities, note_lengths, channels].forEach(function(element, index) {
          if(index == value) {
            element.removeClass("hide");
          } else {
            element.addClass("hide");
          }
        })
        break;
      }
      case 'current_step': {
        const current_step = Math.round(value);
        event.icon.find("[mod-role=input-control-port][id]").each(function () { $(this).removeClass("highlight"); });
		    event.icon.find("[mod-role=input-control-port][id="+current_step+"]").each(function () { $(this).addClass("highlight"); });
        break;
      }
      case "trigger": {
        const trigger = event.icon.find("[mod-port-symbol=trigger]");
        if(value == 1) {
          trigger.addClass("on");
        } else {
          trigger.removeClass("on");
        }
        break;
      }
      case "panic": {
        const panic = event.icon.find("[mod-port-symbol=panic]");
        if(value == 1) {
          panic.addClass("on");
        } else {
          panic.removeClass("on");
        }
        break;
      }
      default:
        break;
    }
  }

  if (event.type == 'start') {
    const ports = event.ports;
    for (const port in ports) {
      handle_port_values(ports[port].symbol, ports[port].value);
    }
  }
  else if (event.type == 'change') {  
    handle_port_values(event.symbol, event.value);
  }
}