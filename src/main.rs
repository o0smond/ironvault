/*
By: <Your Name Here>
Date: 2025-12-12
Program Details: <Program Description Here>
*/

mod modules;

use macroquad::prelude::*;
use modules::grid::draw_grid;
use crate::modules::label::Label;
use crate::modules::text_button::TextButton;
    use crate::modules::text_input::TextInput;


/// Set up window settings before the app runs
fn window_conf() -> Conf {
    Conf {
        window_title: "password_manager".to_string(),
        window_width: 600,
        window_height: 200,
        fullscreen: false,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4, // MSAA: makes shapes look smoother
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    
    loop {
        clear_background(GRAY);

        draw_grid(50.0, BLACK);

        let lbl_out = Label::new("Welcome to password manager! Please enter your password, or what you would like it to be \n if this is your first time using the software.", 50.0, 100.0, 30);
        lbl_out.draw();

        next_frame().await;
    }
}
