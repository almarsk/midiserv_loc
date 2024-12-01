#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use slint::{Brush, Color};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;
    app.on_go_to_main({
        let ui_handle = app.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_bgc(Brush::from(Color::from_argb_u8(255, 0, 255, 0)));
            }
        }
    });
    let _ = app.run();

    Ok(())
}
