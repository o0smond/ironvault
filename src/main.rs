/*
By: Oliver Osmond
Date: 2025-12-12

This file contains the frontend logic for the password manager. 
Handles user input, returning information back to the user,
and providing a GUI for the user to interact with.
*/

mod modules; //use of Matthew Dusome's modules for GUI elements; Buttons labels and text input

use macroquad::{prelude::*};
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

//enum defining the different screens that are possible
enum Screen {
    Login,
    Menu,
    Popup,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut pop_type = String::new(); // defines the type of popup to be displayed
    let mut pop_input: Option<String> = None; // defines the input for the popup, option to avoid warnings
    let mut temp_user = String::new(); // temporary storage for user input
    let mut temp_pass: Option<String> = None; // temporary storage for user input, option to avoid warnings
    let mut temp_euser = String::new(); // temporary storage for user input in the edit popup

    //Defining all the objects for all the screens. Note if you see objects placed at 0.0 they are to be defined later.
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
    let mut screen = Screen::Login; //Setting default screen to Login

    //Defining all the objects for the popup screens
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
        //Defining screen width and height so that objects can position themselves relative to the window size
        let w = screen_width();
        let h = screen_height();

        //Defining 0.0 objects so they can position relative to the screen size each frame
        lbl_hello.set_position((w-500.0)*0.5, (h-37.5)*0.125);
        txt_input.set_position((w-300.0)*0.5, (h-50.0)*0.322);
        btn_ok.update_position((w-200.0)*0.5, (h-50.0)*0.68,Some(200.0),Some(50.0));
        lbl_out.with_fixed_size(w-300.0, h-25.0);

        //main match for doing things based on screen type
        match screen {
            Screen::Login => {
                clear_background(Color::from_rgba(44, 112, 171, 0));

                lbl_hello.with_colors(WHITE, Some(MAROON)).with_font(font.clone());
                lbl_hello.draw();

                btn_ok.with_font(font.clone()).with_round(10.0);
                if btn_ok.click() {
                    let mpass = txt_input.get_text();
                    if mpass == "" {
                        lbl_hello.set_text("Please enter a password");
                    } else {
                        let rmsg = vault_core::pg1_startup(&mpass, "src/modules/vault_core/storage.json").unwrap(); //rmsg = return message
                        if rmsg == "ok" {
                            screen = Screen::Menu;
                            vault_core::unlock_vault().unwrap();
                        } else {
                            lbl_hello.set_text(rmsg); //unlock_vault() will return an error message if password is incorrect
                        }
                    }
                }

                txt_input.with_font(font.clone()).draw();
            }

            Screen::Menu => {
                //updating buttons to draw for the menu screen
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
                //button click actions
                if btn_add.click() {
                    pop_type = "username".to_string(); //taking input through a popup screen, needs type.
                    screen = Screen::Popup;
                }
                if btn_e_r.click() {
                    pop_type = "e_r".to_string();
                    screen = Screen::Popup;
                }
                if btn_save.click() {
                    let rmsg = vault_core::lock_vault().unwrap(); //lock_vault() encrypts and updates the vault JSON. Does not end session.
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
                        break   //same as save, but this ends the session
                    } else {
                        println!("save failed")
                    }
                }
            }

            Screen::Popup => {
                let _ = temp_pass; // Intentional use of _ variable to silence unused variable warning on temp_pass.
                let _ = pop_input; // Intentional use of _ variable to silence unused variable warning on pop_input.

                //defining things to draw in the popup screen
                clear_background(WHITE);
                lbl_bkgrnd.set_position((w-300.0)/2.0, (h-200.0)/2.0).with_fixed_size(300.0,200.0).with_colors(BLACK, Some(BEIGE)).draw();
                btn_ok_msg.update_position(((w-100.0)/2.0)+90.0,((h-50.0)/2.0)+50.0,Some(100.0),Some(50.0)).with_text_color(BLACK);
                btn_exit_msg.update_position(((w-100.0)/2.0)-100.0,((h-50.0)/2.0)+50.0,Some(100.0),Some(50.0)).with_text_color(BLACK);
                txt_input_msg.set_position((w-200.0)/2.0, ((h-50.0)/2.0)-25.0).with_colors(BLACK, BLACK, WHITE, BLACK).draw();
                
                //if user clicks "add password"
                if pop_type == "username" || pop_type == "password" {

                    lbl_bkgrnd.set_text(format!("Input your {}", pop_type));
                    btn_ok_msg.set_text("OK");
                    
                    if btn_ok_msg.click() {
                        pop_input = Some(txt_input_msg.get_text()); //collecting user input
                        let exist_state = vault_core::check_entry(&pop_input.clone().unwrap()).unwrap(); //checks if entry already exists to prevent changing an existing entry's password
                        
                        //made such that one button (ok) can be used to accept user and pass input
                        if pop_input.clone().unwrap().is_empty() {
                            pop_input = None;
                            txt_input_msg.set_text("Blank input"); //writes to input to force them to read before entering another entry, and to prevent overwriting every frame update
                        } else if exist_state {
                            txt_input_msg.set_text("Entry already exists");
                        } else {
                            if pop_type == "username" {
                                temp_user = pop_input.clone().unwrap();
                                pop_type = "password".to_string();
                            } else if pop_type == "password" {
                                vault_core::add_password(&temp_user, &pop_input.clone().unwrap()).unwrap(); //adds passwords to the vault manager. Save still required to write to JSON.
                                //clearing temporary variables
                                pop_type = String::new();
                                temp_user = String::new();
                                temp_pass = None;
                                screen = Screen::Menu;
                            }
                            txt_input_msg.set_text("");
                        }
                    }
                    if btn_exit_msg.click() { //exits the popup
                        pop_input = None;
                        screen = Screen::Menu;
                    }
                //if user clicks "edit or remove"
                } else {
                    lbl_bkgrnd.set_text("Edit or Remove?".to_string());
                    btn_rem_msg.update_position((w-100.0)/2.0,((h-50.0)/2.0)+50.0,Some(100.0),Some(50.0)).with_text_color(BLACK);
                    btn_ok_msg.set_text("Edit");

                    if pop_type == "new_user" {
                        lbl_bkgrnd.set_text("New Username?".to_string());
                        btn_ok_msg.set_text("OK");
                        btn_rem_msg.update_position(0.0, 0.0, Some(0.0), Some(0.0));
                    } else if pop_type == "new_pass" {
                        lbl_bkgrnd.set_text("New Password?".to_string());
                        btn_ok_msg.set_text("OK");
                        btn_rem_msg.update_position(0.0, 0.0, Some(0.0), Some(0.0));
                    }
                    
                    //if user clicks "edit"
                    if btn_ok_msg.click() {
                        pop_input = Some(txt_input_msg.get_text());
                        let ent_existstate = vault_core::check_entry(&pop_input.clone().unwrap()).unwrap();

                        //made so that one button (ok) can be used to accept euser (entry user/username of entry to be edited) input, and user and pass input
                        if ent_existstate == false && pop_type != "new_user" && pop_type != "new_pass" {
                            pop_input = None;
                            txt_input_msg.set_text("Entry nonexistant");
                        } else if pop_input.clone().unwrap() == "" {
                            pop_input = None;
                            txt_input_msg.set_text("Blank input");
                        }else if pop_type != "new_user" && pop_type != "new_pass" {
                            temp_euser = pop_input.clone().unwrap();
                            pop_type = "new_user".to_string();
                            txt_input_msg.set_text("");
                        } else if pop_type == "new_user" {
                            temp_user = pop_input.clone().unwrap();
                            pop_type = "new_pass".to_string();
                            txt_input_msg.set_text("");
                        } else if pop_type == "new_pass" {
                            vault_core::delete(&temp_euser).unwrap();
                            vault_core::add_password(&temp_user, &pop_input.clone().unwrap()).unwrap();
                            pop_type = String::new();
                            temp_user = String::new();
                            temp_pass = None; 
                            txt_input_msg.set_text("");
                            screen = Screen::Menu;
                        } else {
                            txt_input_msg.set_text("Error");
                        }
                    }

                    //If user clicks "remove"
                    if btn_rem_msg.click() {
                        let input_text = txt_input_msg.get_text();
                        let exist_state = vault_core::check_entry(&input_text).unwrap();
                        
                        if input_text.is_empty() {
                            txt_input_msg.set_text("Blank input");
                        } else if exist_state == false {
                            txt_input_msg.set_text("Entry nonexistant");
                        } else {
                            vault_core::delete(&input_text).unwrap();
                            screen = Screen::Menu;
                            txt_input_msg.set_text("");
                        }
                        pop_input = None;
                    }
                    if btn_exit_msg.click() {
                        txt_input_msg.set_text("");
                        pop_input = None;
                        screen = Screen::Menu;
                    }
                }
            }

        }
        next_frame().await;
    }
}
