use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

// inspired by https://github.com/Boddlnagg/midir/blob/master/examples/test_play.rs

const CC_MESSAGE: u8 = 0xB0;

type Ports = Vec<(String, String, MidiOutputPort)>;

pub struct Midi {
    conn: Option<MidiOutputConnection>,
    ports: Ports,
}

impl Midi {
    pub fn new() -> Self {
        let ports = if let Ok(m) = MidiOutput::new("midiserve") {
            m.ports()
                .into_iter()
                .map(|p| (p.id().clone(), m.port_name(&p).unwrap_or("".to_string()), p))
                .collect()
        } else {
            vec![]
        };

        Midi { conn: None, ports }
    }

    pub fn get_ports(&mut self) -> Vec<String> {
        self.ports
            .iter()
            .map(|p| format!("{}|{}", p.0.clone(), p.1.clone()))
            .collect()
    }

    pub fn update_port(&mut self, out_port: usize) {
        if let Ok(m) = MidiOutput::new("midiserve") {
            if let Some(mp) = self.ports.get(out_port) {
                if let Ok(conn_out) = m.connect(&mp.2, "midiserv") {
                    self.conn = Some(conn_out);
                }
            }
        }
    }

    pub fn send_cc(&mut self, controller: u8, value: u8) {
        if let Some(c) = self.conn.as_mut() {
            let _ = c.send(&[CC_MESSAGE, controller, value]);
        }
    }
}
