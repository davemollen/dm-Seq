function(event) {
  function handle_port_values(symbol, value) {
    switch (symbol) {
      case "knob_target":
        const notes = event.icon.find("#notes");
        const velocities = event.icon.find("#velocities");
        if(value == 1) {
          notes.addClass("hide")
          velocities.removeClass("hide")
        } else {
          notes.removeClass("hide")
          velocities.addClass("hide")
        }
        break;
      case 'current_step':
        const current_step = Math.round(value)
        event.icon.find("[mod-role=input-control-port][id]").each(function () { $(this).removeClass("highlight"); });
		    event.icon.find("[mod-role=input-control-port][id="+current_step+"]").each(function () { $(this).addClass("highlight"); });
        break;
      case "trigger":
        const trigger = event.icon.find("[mod-port-symbol=trigger]");
        if(value == 1) {
          trigger.addClass("on");
        } else {
          trigger.removeClass("on");
        }
        break;
      case "panic":
        const panic = event.icon.find("[mod-port-symbol=panic]");
        if(value == 1) {
          panic.addClass("on");
        } else {
          panic.removeClass("on");
        }
        break;
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