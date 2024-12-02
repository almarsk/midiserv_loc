use midir::{MidiOutput, MidiOutputConnection};
use std::{collections::HashMap, error::Error};

// https://github.com/Boddlnagg/midir/blob/master/examples/test_play.rs

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

    pub fn get_ports(self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let out_ports = self.out.ports();

        let mut ports = HashMap::new();

        out_ports.clone().into_iter().for_each(|port| {
            ports.insert(port.id(), self.out.port_name(&out_ports[0]).unwrap());
        });

        Ok(ports)
    }

    pub fn _update_port(mut self, out_port: String) {
        let m_port = self.out.find_port_by_id(out_port);
        if let Some(mp) = m_port {
            let conn_out = self.out.connect(&mp, "midiserv");
            if let Ok(co) = conn_out {
                self.conn = Some(co);
            }
        }
    }

    pub fn _send_dummy_cc(self, channel: u8) {
        if let Some(mut c) = self.conn {
            let _ = c.send(&[CC_MESSAGE, channel, 0]);
        }
    }
}
/*
    println!("Connection open. Listen!");
    {
        // Define a new scope in which the closure `play_note` borrows conn_out, so it can be called easily
        let mut play_note = |note: u8, duration: u64| {
            const NOTE_ON_MSG: u8 = 0x90;
            const NOTE_OFF_MSG: u8 = 0x80;
            const VELOCITY: u8 = 0x64;
            // We're ignoring errors in here
            let _ = conn_out.send(&[NOTE_ON_MSG, note, VELOCITY]);
            sleep(Duration::from_millis(duration * 150));
            let _ = conn_out.send(&[NOTE_OFF_MSG, note, VELOCITY]);
        };

        sleep(Duration::from_millis(4 * 150));

        play_note(66, 4);
        play_note(65, 3);
        play_note(63, 1);
        play_note(61, 6);
        play_note(59, 2);
        play_note(58, 4);
        play_note(56, 4);
        play_note(54, 4);
    }
    sleep(Duration::from_millis(150));
    println!("\nClosing connection");
    // This is optional, the connection would automatically be closed as soon as it goes out of scope
    conn_out.close();
    println!("Connection closed");
    Ok(())
}

*/
