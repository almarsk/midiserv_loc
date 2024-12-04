use midir::{MidiOutput, MidiOutputConnection};
use std::{collections::HashMap, error::Error};

// inspired by https://github.com/Boddlnagg/midir/blob/master/examples/test_play.rs

const CC_MESSAGE: u8 = 0xB0;

pub struct Midi {
    out: MidiOutput,
    conn: Option<MidiOutputConnection>,
}

impl Midi {
    pub fn new() -> Result<Self, ()> {
        let midi_out = MidiOutput::new("midiserve");

        if let Ok(m) = midi_out {
            Ok(Midi { out: m, conn: None })
        } else {
            Err(())
        }
    }

    pub fn get_ports(&self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        if let Ok(m) = MidiOutput::new("midiserve") {
            let out_ports = m.ports();
            let mut ports = HashMap::new();
            out_ports.clone().into_iter().for_each(|port| {
                ports.insert(port.id(), self.out.port_name(&out_ports[0]).unwrap());
            });
            Ok(ports)
        } else {
            Err(format!("midi issue"))?
        }
    }

    pub fn update_port(&mut self, out_port: String) {
        if let Ok(m) = MidiOutput::new("midiserve") {
            println!("port candidate {}", out_port);
            let m_port = self.out.find_port_by_id(out_port);
            if let Some(mp) = m_port {
                if let Ok(conn_out) = m.connect(&mp, "midiserv") {
                    self.conn = Some(conn_out);
                    println!("yes port")
                } else {
                    println!("no port")
                }
            }
        }
    }

    pub fn send_dummy_cc(&mut self, channel: u8) {
        if let Some(c) = self.conn.as_mut() {
            println!("sendin {}", channel);
            let _ = c.send(&[CC_MESSAGE, channel, 0]);
        } else {
            println!("nah")
        }
    }
}
