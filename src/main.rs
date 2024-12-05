#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod exposed_devices;
mod midi;

use anyhow::Result;
use exposed_devices::Device;
use exposed_devices::DeviceCommand;
use flume::bounded;
use flume::Receiver;
use flume::Sender;
use midi::MidiCommand;
use slint::ComponentHandle;
use slint::{ModelRc, SharedString, VecModel};
use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (shutdown_tx, shutdown_rx): (Sender<bool>, Receiver<bool>) = bounded(10);

    let mut midi = midi::Midi::new();

    let p = Rc::new(
        midi.get_ports()
            .iter()
            .map(|s| SharedString::from(s))
            .collect::<VecModel<_>>(),
    );
    app.global::<AppState>()
        .set_midi_ports(ModelRc::from(Rc::clone(&p)));

    let exp_dev = Rc::new(VecModel::from(vec![]));
    app.set_exposed_devices(ModelRc::from(Rc::clone(&exp_dev)));

    let (device_command_tx, mut device_command_rx) = mpsc::channel::<DeviceCommand>(10);
    let (device_response_tx, device_response_rx): (Sender<Vec<String>>, Receiver<Vec<String>>) =
        bounded(10);

    let shutdown_rx_clone = shutdown_rx.clone();
    rt.spawn(async move {
        let exposed_devices = exposed_devices::ExposedDevices::new();
        let exposed = Arc::new(Mutex::new(exposed_devices));
        let exposed = Arc::clone(&exposed);
        loop {
            tokio::select! {
                exposed_device_command = device_command_rx.recv() => {
                    if let Some(e) = exposed_device_command {
                        match e {
                            DeviceCommand::Push(d) => exposed.lock().await.push(d),
                            DeviceCommand::Remove(index) => exposed.lock().await.remove(index),
                            DeviceCommand::Clear => exposed.lock().await.clear(),
                            DeviceCommand::CopyToClipBoard => {
                                exposed.lock().await.copy_to_clipboard()
                            },
                            DeviceCommand::GetJoined => {
                                let _ = device_response_tx
                                    .send_async(exposed.lock().await.get_joined())
                                    .await;
                                }
                        }
                    }
                }
                shutdown_option = shutdown_rx_clone.recv_async() => {
                    if let Ok(shutdown) = shutdown_option {
                        if shutdown {
                            break;
                        }
                    }
                }
            }
        }
    });

    let (midi_tx, mut midi_rx) = mpsc::channel::<MidiCommand>(10);

    let shutdown_rx_clone = shutdown_rx.clone();
    rt.spawn(async move {
        let midi = Arc::new(Mutex::new(midi));
        let midi = Arc::clone(&midi);
        loop {
            tokio::select! {
                command_option = midi_rx.recv() => {
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
                shutdown_option = shutdown_rx_clone.recv_async() => {
                    if let Ok(shutdown) = shutdown_option {
                        if shutdown {
                            break;
                        }
                    }
                }
            }
        }
    });

    let tx_clone = midi_tx.clone();
    app.global::<AppState>().on_choose_midi_port(move |port| {
        let tx_clone_clone = tx_clone.clone();
        let _ = slint::spawn_local(async move {
            let _ = tx_clone_clone.send(MidiCommand::Port(port as usize)).await;
        });
    });

    let tx_clone = midi_tx.clone();
    app.global::<AppState>()
        .on_send_dummy_cc(move |controller| {
            let tx_clone_clone = tx_clone.clone();
            let _ = slint::spawn_local(async move {
                if let Ok(cc) = controller.clone().parse::<u8>() {
                    let _ = tx_clone_clone.send(MidiCommand::Dummy(cc)).await;
                }
            });
        });

    let tx_clone = device_command_tx.clone();
    let rx_clone = device_response_rx.clone();
    let exp_dev_clone = Rc::clone(&exp_dev);
    app.global::<AppState>()
        .on_expose_device(move |cc, ui_type, description| {
            if let Some(new_device) = Device::from_string_args(
                cc.to_string(),
                ui_type.to_string(),
                description.to_string(),
            ) {
                let tx_clone = tx_clone.clone();
                let rx_clone = rx_clone.clone();
                let exp_dev_clone = exp_dev_clone.clone();
                let _ = slint::spawn_local(async move {
                    let _ = tx_clone.send(DeviceCommand::Push(new_device)).await;

                    let _ = tx_clone.send(DeviceCommand::GetJoined).await;
                    exp_dev_clone.set_vec(
                        rx_clone
                            .recv_async()
                            .await
                            .map(|v| v.iter().map(|s| SharedString::from(s)).collect())
                            .unwrap_or(vec![]),
                    )
                });
            }
        });

    let tx_clone = device_command_tx.clone();
    let rx_clone = device_response_rx.clone();
    let exp_dev_clone = Rc::clone(&exp_dev);
    app.global::<AppState>().on_hide_device(move |i| {
        if let Ok(index) = i.parse::<usize>() {
            let tx_clone = tx_clone.clone();
            let rx_clone = rx_clone.clone();
            let exp_dev_clone = exp_dev_clone.clone();
            let _ = slint::spawn_local(async move {
                let _ = tx_clone.send(DeviceCommand::Remove(index)).await;

                let _ = tx_clone.send(DeviceCommand::GetJoined).await;
                exp_dev_clone.set_vec(
                    rx_clone
                        .recv_async()
                        .await
                        .map(|v| v.iter().map(|s| SharedString::from(s)).collect())
                        .unwrap_or(vec![]),
                )
            });
        }
    });

    let tx_clone = device_command_tx.clone();
    app.global::<AppState>().on_copy_to_clipboard(move || {
        let tx_clone = tx_clone.clone();
        let _ = slint::spawn_local(async move {
            let _ = tx_clone.send(DeviceCommand::CopyToClipBoard).await;
        });
    });

    let tx_clone = device_command_tx.clone();
    let exp_dev_clone = exp_dev.clone();
    app.global::<AppState>().on_clear_all(move || {
        let tx_clone = tx_clone.clone();
        let exp_dev_clone = exp_dev_clone.clone();
        let _ = slint::spawn_local(async move {
            let _ = tx_clone.send(DeviceCommand::Clear);
            exp_dev_clone.set_vec(vec![])
        });
    });

    let _ = app.run();
    let _ = shutdown_tx.send(true);

    rt.block_on(async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    });
    Ok(())
}
