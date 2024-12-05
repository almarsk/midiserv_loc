use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

// inspired by https://github.com/Boddlnagg/midir/blob/master/examples/test_play.rs

const CC_MESSAGE: u8 = 0xB0;

pub struct Midi {
    conn: Option<MidiOutputConnection>,
    ports: Vec<Port>,
}

pub enum MidiCommand {
    Dummy(u8),
    Port(usize),
}

struct Port {
    id: String,
    name: String,
    port: MidiOutputPort,
}

impl Port {
    fn new(id: String, name: String, port: MidiOutputPort) -> Self {
        Port { id, name, port }
    }
}

impl Midi {
    pub fn new() -> Self {
        let midi_output = MidiOutput::new("midiserve").unwrap();
        let ports = midi_output
            .ports()
            .into_iter()
            .map(|p| {
                Port::new(
                    p.id().clone(),
                    midi_output.port_name(&p).unwrap_or("".to_string()),
                    p,
                )
            })
            .collect();

        Midi { conn: None, ports }
    }

    pub fn get_ports(&mut self) -> Vec<String> {
        self.ports
            .iter()
            .map(|p| format!("{}|{}", p.id.clone(), p.name.clone()))
            .collect()
    }

    pub fn update_port(&mut self, out_port: usize) {
        if let Some(p) = self.ports.get(out_port) {
            self.conn = Some(
                MidiOutput::new("midiserve")
                    .unwrap()
                    .connect(&p.port, "midiserv")
                    .unwrap(),
            );
        }
    }

    pub fn send_cc(&mut self, controller: u8, value: u8) {
        if let Some(c) = self.conn.as_mut() {
            let _ = c.send(&[CC_MESSAGE, controller, value]);
        }
    }
}
