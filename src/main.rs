/*
By: Oliver Osmond
Date: 2025-12-12
Program Details: Rust GUI for password manager
*/

mod modules;

use macroquad::{color, prelude::*};
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
    Popup,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut pop_type = String::new();
    let mut pop_input = String::new();
    let mut temp_user = String::new();
    let mut temp_pass = String::new();
    let mut temp_euser = String::new();

    let mut txt_input = TextInput::new(0.0, 0.0, 300.0, 40.0, 25.0);
    let font = load_ttf_font("assets/CaviarDreams_Bold.ttf").await.unwrap();
    let mut lbl_hello = Label::new("Welcome to password manager! Please enter your password, or what\nyou would like it to be if this is your first time using the software.", 0.0, 0.0, 15);
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
    let mut lbl_out = Label::new("hello",250.0,25.0,25);
    lbl_out.with_colors(BLACK, Some(WHITE)).with_font(font.clone());
    let mut screen = screen::Login;

    let mut txt_input_msg = TextInput::new(0.0, 0.0, 200.0, 50.0, 25.0);
    let mut lbl_bkgrnd = Label::new("", 0.0,0.0, 20);
    let mut btn_ok_msg = TextButton::new(
        0.0, 
        0.0,
        100.0,
        50.0,
        "OK",
        WHITE,
        GREEN,
        30
    );
    let mut btn_rem_msg = TextButton::new(
        0.0, 
        0.0,
        100.0,
        50.0,
        "Remove",
        WHITE,
        RED,
        30
    );
    let mut btn_exit_msg = TextButton::new(
        0.0,
        0.0,
        100.0,
        50.0,
        "Cancel",
        WHITE,
        RED,
        20
    );

    loop {
        let w = screen_width();
        let h = screen_height();

        lbl_hello.set_position((w-500.0)*0.5, (h-37.5)*0.125);
        txt_input.set_position((w-300.0)*0.5, (h-50.0)*0.322);
        btn_ok.update_position((w-200.0)*0.5, (h-50.0)*0.68,Some(200.0),Some(50.0));
        lbl_out.with_fixed_size(w-300.0, h-25.0);

        match screen {
            screen::Login => {
                clear_background(Color::from_rgba(44, 112, 171, 0));

                lbl_hello.with_colors(WHITE, Some(MAROON)).with_font(font.clone());
                lbl_hello.draw();

                btn_ok.with_font(font.clone()).with_round(10.0);
                if btn_ok.click() {
                    let mpass = txt_input.get_text();
                    if mpass == "" {
                        lbl_hello.set_text("Please enter a password");
                    } else {
                        let rmsg = vault_core::pg1_startup(&mpass, "src/modules/vault_core/storage.json").unwrap();
                        if rmsg == "ok" {
                            screen = screen::Menu;
                            vault_core::unlock_vault().unwrap();
                        } else {
                            lbl_hello.set_text(rmsg);
                        }
                    }
                }

                txt_input.with_font(font.clone()).draw();
            }

            screen::Menu => {
                clear_background(Color::from_rgba(44, 112, 171, 0));
                lbl_out.set_text(vault_core::print_map().unwrap());
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
                lbl_out.draw();
                if btn_add.click() {
                    pop_type = "username".to_string();
                    screen = screen::Popup;
                }
                if btn_e_r.click() {
                    pop_type = "e_r".to_string();
                    screen = screen::Popup;
                }
                if btn_save.click() {
                    let rmsg = vault_core::lock_vault().unwrap();
                    if rmsg == "ok".to_string() {
                        println!("Saved")
                    } else {
                        println!("save failed")
                    }
                }
                if btn_exit.click() {
                    let rmsg = vault_core::lock_vault().unwrap();
                    if rmsg == "ok".to_string() {
                        println!("Saved");
                        break
                    } else {
                        println!("save failed")
                    }
                }
            }

            screen::Popup => {
                clear_background(WHITE);
                lbl_bkgrnd.set_position((w-300.0)/2.0, (h-200.0)/2.0).with_fixed_size(300.0,200.0).with_colors(BLACK, Some(BEIGE)).draw();
                btn_ok_msg.update_position(((w-100.0)/2.0)+90.0,((h-50.0)/2.0)+50.0,Some(100.0),Some(50.0)).with_text_color(BLACK);
                btn_exit_msg.update_position(((w-100.0)/2.0)-100.0,((h-50.0)/2.0)+50.0,Some(100.0),Some(50.0)).with_text_color(BLACK);
                txt_input_msg.set_position((w-200.0)/2.0, ((h-50.0)/2.0)-25.0).with_colors(BLACK, BLACK, WHITE, BLACK).draw();
                if pop_type == "username".to_string() || pop_type == "password".to_string() {
                    lbl_bkgrnd.set_text(format!("Input your {}", pop_type));
                    btn_ok_msg.set_text("OK");
                    if btn_ok_msg.click() {
                        pop_input = txt_input_msg.get_text();
                        let exist_state = vault_core::check_entry(&pop_input).unwrap();
                        if pop_input == "" {
                            pop_input = String::new();
                            txt_input_msg.set_text("Blank input");
                        } else if exist_state == true {
                            txt_input_msg.set_text("Entry already exists");
                        }else {
                            if pop_type == "username".to_string() {
                                temp_user = pop_input;
                                pop_type = "password".to_string();
                            } else if pop_type == "password".to_string() {
                                temp_pass = pop_input;
                                vault_core::add_password(&temp_user, &temp_pass);
                                pop_type = String::new();
                                temp_user = String::new();
                                temp_pass = String::new();
                                screen = screen::Menu;
                            }
                            txt_input_msg.set_text("");
                        }
                    }
                    if btn_exit_msg.click() {
                        pop_input = String::new();
                        screen = screen::Menu;
                    }
                } else {
                    lbl_bkgrnd.set_text(format!("Edit or Remove?"));
                    btn_rem_msg.update_position(((w-100.0)/2.0),((h-50.0)/2.0)+50.0,Some(100.0),Some(50.0)).with_text_color(BLACK);
                    btn_ok_msg.set_text("Edit");

                    if pop_type == "new_user".to_string() {
                        lbl_bkgrnd.set_text("New Username?");
                        btn_ok_msg.set_text("OK");
                        btn_rem_msg.update_position(0.0, 0.0, Some(0.0), Some(0.0));
                    } else if pop_type == "new_pass".to_string() {
                        lbl_bkgrnd.set_text("New Password?");
                        btn_ok_msg.set_text("OK");
                        btn_rem_msg.update_position(0.0, 0.0, Some(0.0), Some(0.0));
                    }
                    
                    if btn_ok_msg.click() {
                        pop_input = txt_input_msg.get_text();
                        let ent_existstate = vault_core::check_entry(&pop_input).unwrap();

                        if ent_existstate == false && pop_type != "new_user".to_string() && pop_type != "new_pass".to_string()  {
                            pop_input = String::new();
                            txt_input_msg.set_text("Entry does not exist");
                        } else if pop_type != "new_user".to_string() && pop_type != "new_pass".to_string() {
                            txt_input_msg.set_text("");
                            temp_euser = pop_input;
                            pop_type = "new_user".to_string();
                        } else if pop_type == "new_user".to_string() && pop_input != "" {
                            temp_user = pop_input.to_string();
                            pop_type = "new_pass".to_string();
                            txt_input_msg.set_text("");
                        } else if pop_type == "new_pass".to_string() && pop_input != ""{
                            temp_pass = pop_input;
                            vault_core::delete(&temp_euser);
                            vault_core::add_password(&temp_user, &temp_pass);
                            pop_type = String::new();
                            temp_user = String::new();
                            temp_pass = String::new();
                            txt_input_msg.set_text("");
                            screen = screen::Menu;
                        } else {
                            txt_input_msg.set_text("Blank input");
                        }
                    }
                    if btn_rem_msg.click() {
                        pop_input = txt_input_msg.get_text();
                        let exist_state = vault_core::check_entry(&pop_input).unwrap();
                        if pop_input == "".to_string() {
                            txt_input_msg.set_text("Blank input");
                        } else if exist_state == false {
                            txt_input_msg.set_text("Entry nonexistant");
                        }else {
                            vault_core::delete(&pop_input);
                            screen = screen::Menu;
                            txt_input_msg.set_text("");
                        }
                        pop_input = String::new();
                    }
                    if btn_exit_msg.click() {
                        txt_input_msg.set_text("");
                        pop_input = String::new();
                        screen = screen::Menu;
                    }
                }
            }

        }
        next_frame().await;
    }
}
