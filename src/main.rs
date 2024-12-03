#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod midi;

use anyhow::Result;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use tokio::sync::mpsc;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;

    // todo tokio refactor
    let _rt = tokio::runtime::Runtime::new().unwrap();
    let (_tx, mut _rx) = mpsc::channel::<String>(32);

    let midi = if let Ok(m) = midi::Midi::new() {
        m
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Midi error occured",
        )));
    };

    if let Ok(ports) = midi.get_ports() {
        let current_port = Rc::new(RefCell::new(String::new()));
        let current_port_clone = Rc::clone(&current_port);
        app.global::<AppState>().on_choose_midi_port(move |port| {
            current_port_clone.replace(port.to_string());
        });
        let p = Rc::new(
            ports
                .iter()
                .map(|(k, v)| SharedString::from(format!("{}|{}", v, k)))
                .collect::<VecModel<_>>(),
        );
        p.insert(0, SharedString::from(""));

        app.global::<AppState>()
            .set_midi_ports(ModelRc::from(Rc::clone(&p)));

        let current_port_clone = Rc::clone(&current_port);
        app.global::<AppState>()
            .on_send_dummy_cc(move |controller| {
                println!(
                    "cc {} port {}",
                    controller,
                    current_port_clone.borrow().split("|").last().unwrap_or("")
                );
            })
    }

    let exposed_devices = Rc::new(VecModel::from(vec![]));
    app.set_exposed_devices(ModelRc::from(Rc::clone(&exposed_devices)));

    let exposed_devices_clone = Rc::clone(&exposed_devices);
    app.global::<AppState>()
        .on_expose_device(move |input1, input2, input3| {
            let new_device = SharedString::from(format!("{}|{}|{}", input1, input2, input3));
            exposed_devices_clone.push(new_device);
        });

    let exposed_devices_clone = Rc::clone(&exposed_devices);
    app.global::<AppState>().on_hide_device(move |string| {
        if let Some(i) = exposed_devices_clone.iter().position(|s| s == string) {
            exposed_devices_clone.remove(i);
        }
    });

    let exposed_devices_clone = Rc::clone(&exposed_devices);
    app.global::<AppState>().on_clear_all(move || {
        exposed_devices_clone.clear();
    });

    let _ = app.run();
    Ok(())
}
