#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod midi;

use anyhow::Result;
use midir::MidiOutput;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

slint::include_modules!();

enum MidiCommand {
    Dummy(u8),
    Port(usize),
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (command_tx, mut command_rx) = mpsc::channel::<MidiCommand>(10);
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

    let mut midi = midi::Midi::new();

    let p = Rc::new(
        midi.get_ports()
            .iter()
            .map(|s| SharedString::from(s))
            .collect::<VecModel<_>>(),
    );
    app.global::<AppState>()
        .set_midi_ports(ModelRc::from(Rc::clone(&p)));

    rt.spawn(async move {
        let midi = Arc::new(Mutex::new(midi));
        let midi = Arc::clone(&midi);
        loop {
            tokio::select! {
                command_option = command_rx.recv() => {
                 if let Some(command) = command_option {
                     let mut midi = midi.lock().await;
                     match command {
                         MidiCommand::Dummy(cc) => {
                             midi.send_cc(cc, 0)
                         }
                         MidiCommand::Port(port) => midi.update_port(port),
                     }
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
    });

    let tx_clone = command_tx.clone();
    app.global::<AppState>().on_choose_midi_port(move |port| {
        let tx_clone_clone = tx_clone.clone();
        let _ = slint::spawn_local(async move {
            let _ = tx_clone_clone.send(MidiCommand::Port(port as usize)).await;
        });
    });

    let tx_clone = command_tx.clone();
    app.global::<AppState>()
        .on_send_dummy_cc(move |controller| {
            let tx_clone_clone = tx_clone.clone();
            let _ = slint::spawn_local(async move {
                if let Ok(cc) = controller.clone().parse::<u8>() {
                    let _ = tx_clone_clone.send(MidiCommand::Dummy(cc)).await;
                }
            });
        });

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
    let _ = shutdown_tx.send(true);

    rt.block_on(async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    });
    Ok(())
}
