/*
By: Oliver Osmond
Date: 2025-12-12
Program Details: Rust GUI for password manager
*/

mod modules;

use macroquad::{color, prelude::*};
use modules::grid::draw_grid;
use crate::modules::label::Label;
use crate::modules::text_button::TextButton;
use crate::modules::text_input::TextInput;
use crate::modules::vault_core;


/// Set up window settings before the app runs
fn window_conf() -> Conf {
    Conf {
        window_title: "password_manager".to_string(),
        window_width: 850,
        window_height: 600,
        fullscreen: false,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4, // MSAA: makes shapes look smoother
        ..Default::default()
    }
}

enum screen {
    Login,
    Menu,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut txt_input = TextInput::new(0.0, 0.0, 300.0, 40.0, 25.0);
    let font = load_ttf_font("assets/CaviarDreams_Bold.ttf").await.unwrap();
    let mut lbl_out = Label::new("Welcome to password manager! Please enter your password, or what\nyou would like it to be if this is your first time using the software.", 0.0, 0.0, 15);
    let mut btn_ok = TextButton::new(
        0.0,
        0.0,
        200.0,
        50.0,
        "OK",
        MAROON,
        MAGENTA,
        30
    );

    let mut btn_add = TextButton::new(
        25.0,
        25.0,
        200.0,
        50.0,
        "Add Entry",
        MAROON,
        MAGENTA,
        20
    );

    let mut btn_e_r = TextButton::new(
        25.0,
        95.0,
        200.0,
        50.0,
        "Edit/Remove Entry",
        MAROON,
        MAGENTA,
        20
    );

    let mut btn_save = TextButton::new(
        25.0,
        165.0,
        200.0,
        50.0,
        "Save",
        MAROON,
        MAGENTA,
        20
    );

    let mut btn_exit = TextButton::new(
        25.0,
        235.0,
        200.0,
        50.0,
        "Save+Exit",
        MAROON,
        MAGENTA,
        20
    );
    let mut screen = screen::Login;
    loop {
        let w = screen_width();
        let h = screen_height();

        lbl_out.set_position((w-500.0)*0.5, (h-37.5)*0.125);
        txt_input.set_position((w-300.0)*0.5, (h-50.0)*0.322);
        btn_ok.update_position((w-200.0)*0.5, (h-50.0)*0.68,Some(200.0),Some(50.0));

        match screen {
            screen::Login => {
                clear_background(Color::from_rgba(44, 112, 171, 0));
                draw_grid(50.0, BLACK);

                lbl_out.with_colors(WHITE, Some(MAROON)).with_font(font.clone());
                lbl_out.draw();

                btn_ok.with_font(font.clone()).with_round(10.0);
                if btn_ok.click() {
                    let mpass = txt_input.get_text();
                    if mpass == "" {
                        lbl_out.set_text("Please enter a password");
                    } else {
                        let rmsg = vault_core::pg1_startup(&mpass, "src/modules/vault_core/storage.json").unwrap();
                        if rmsg == "ok" {
                            screen = screen::Menu;
                            vault_core::unlock_vault().unwrap();
                        } else {
                            lbl_out.set_text(rmsg);
                        }
                    }
                    screen = screen::Menu;
                }

                txt_input.with_font(font.clone()).draw();
            }

            screen::Menu => {
                clear_background(Color::from_rgba(44, 112, 171, 0));
                draw_grid(50.0, BLACK);
                btn_add
                    .with_font(font.clone())
                    .with_round(10.0);
                btn_e_r
                    .with_font(font.clone())
                    .with_round(10.0);
                btn_save
                    .with_font(font.clone())
                    .with_round(10.0);
                btn_exit
                    .with_font(font.clone())
                    .with_round(10.0);
                if btn_add.click() {
                    {}
                }
                if btn_e_r.click() {
                    {}
                }
                if btn_save.click() {
                    {}
                }
                if btn_exit.click() {
                    {}
                }
            }
        }

        next_frame().await;
    }
}
