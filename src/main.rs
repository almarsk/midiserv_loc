#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod midi;
// https://github.com/Boddlnagg/midir/blob/master/examples/test_play.rs

use slint::{Model, ModelRc, SharedString, VecModel};
use std::{error::Error, rc::Rc};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;

    let exposed_devices = Rc::new(VecModel::from(vec![]));
    app.set_exposed_devices(ModelRc::from(Rc::clone(&exposed_devices)));
    let exposed_devices_clone = Rc::clone(&exposed_devices);

    let ports_res = midi::get_ports();

    if let Ok(ports) = ports_res {
        let p = Rc::new(
            ports
                .iter()
                .map(|(k, v)| SharedString::from(format!("{}|{}", v, k)))
                .collect::<VecModel<_>>(),
        );

        app.global::<AppState>()
            .set_midi_ports(ModelRc::from(Rc::clone(&p)));
    }

    app.global::<AppState>()
        .on_expose_device(move |input1, input2, input3| {
            let new_device = SharedString::from(format!("{}|{}|{}", input1, input2, input3));
            exposed_devices_clone.push(new_device);
        });

    app.global::<AppState>().on_hide_device(move |string| {
        let exposed_devices_clone2 = Rc::clone(&exposed_devices);
        if let Some(i) = exposed_devices_clone2.iter().position(|s| s == string) {
            exposed_devices.remove(i);
        }
    });

    let _ = app.run();
    Ok(())
}
