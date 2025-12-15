/*
Made by: Mathew Dusome
May 16, 2025
Adds a message box (dialog) component for displaying messages and options to users

In your mod.rs file located in the modules folder add the following to the end of the file
    pub mod messagebox;
    
For info boxs add the following with the use commands:
use crate::modules::messagebox::MessageBox;

or for comfirmation boxes use:
use crate::modules::messagebox::{MessageBox, MessageBoxResult};

QUICK EXAMPLES:

1. SIMPLEST USAGE (just call this in your main loop)
```rust
// Create a message box (do this once, outside main loop)
let mut info_box = MessageBox::info("Information", "Operation completed successfully!");
info_box.show();  // Show it immediately or when needed

// In your main loop, after drawing other elements:
info_box.draw();  // That's it! No if statement needed for simple info dialogs

// Show the dialog again when needed (e.g., on a key press)
if is_key_pressed(KeyCode::I) {
    info_box.show();
}
```

2. Other message box types
```rust
// Confirmation dialog with Yes/No buttons
let mut confirm_box = MessageBox::confirm(
    "Confirm Action",
    "Do you want to save your progress?"
);

// Dialog with Yes/No/Cancel buttons
let mut save_dialog = MessageBox::confirm_with_cancel(
    "Save Game",
    "Would you like to save your progress?"
);

// Custom dialog with specific buttons
let mut custom_dialog = MessageBox::custom(
    "Choose Difficulty",
    "Select game difficulty:",
    vec!["Easy", "Normal", "Hard", "Extreme"],
    Some(1)  // Default to "Normal"
);
```

3. Handling the result in the main loop (when you need to know which button was clicked)
```rust
// Inside the game loop, after drawing other elements:
if let Some(result) = confirm_box.draw() {
    // Only runs when a button was clicked or dialog was closed
    match result {
        MessageBoxResult::ButtonPressed(0) => {
            // "Yes" button pressed
            save_game();
        },
        MessageBoxResult::ButtonPressed(1) => {
            // "No" button pressed
            // Continue without saving...
        },
        MessageBoxResult::ButtonPressed(2) => {
            // "Cancel" button pressed (for confirm_with_cancel dialogs)
            // Handle cancel operation...
        },
        #[allow(unused)]
        MessageBoxResult::ButtonPressed(_) => {
            // IMPORTANT: This catch-all pattern is required by the Rust compiler
            // even for simple confirm dialogs to ensure all possible values are covered
        },
        MessageBoxResult::Closed => {
            // Dialog closed with X or Escape key
            // Handle as cancel...
        }
    }
}
```

4. Customizing appearance
```rust
let mut dialog = MessageBox::info("Custom Style", "This dialog has custom colors");
dialog.with_colors(
    DARKBLUE,                   // Title background
    SKYBLUE,                    // Dialog background
    WHITE,                      // Title text
    BLACK,                      // Message text
    Color::new(0.0, 0.0, 0.0, 0.5)  // Modal overlay color
);

// Custom font and font sizes
// Load font in an async context
let font = load_ttf_font_from_bytes(include_bytes!("path/to/your/font.ttf"))
    .expect("Failed to load font");
dialog.with_font(font);
dialog.with_font_sizes(24.0, 18.0, 16.0);  // Title, message, and button sizes

// Custom button colors
dialog.with_button_colors(
    LIGHTGRAY,          // Normal button color
    SKYBLUE,            // Button hover/selected color
    BLACK               // Button text color
);
```
    // Clear background and draw other elements first
    clear_background(RED);
    draw_your_other_elements();
    
    // For simple info dialogs, just call draw without an if statement:
    info_box.draw();
    
    // For dialogs where you need to know which button was clicked:
    if let Some(result) = confirm_box.draw() {
        match result {
            MessageBoxResult::ButtonPressed(0) => {
                // "Yes" button was pressed
                // Save game...
                // No need to call hide() - it's done automatically
            },
            MessageBoxResult::ButtonPressed(1) => {
                // "No" button was pressed
                // Continue without saving...
            },
            #[allow(unused)]
            MessageBoxResult::ButtonPressed(_) => {
                // IMPORTANT: This catch-all is required even for confirm dialogs
                // The Rust compiler needs this to ensure all cases are handled
            },
            MessageBoxResult::Closed => {
                // Dialog was closed with X or Escape key
                // Handle as cancel...
            }
        }
    }
    
    // Show message box again when a key is pressed
    if is_key_pressed(KeyCode::S) {
        confirm_box.show();
    }
    
IMPORTANT NOTES:
- Make sure to call draw() AFTER drawing other elements in your main loop
- Don't call .show() inside the main loop unless in response to an event (like a key press)
- The dialog automatically hides when a button is clicked or it's closed
- Buttons change color and show a highlighted border when hovered for better visual feedback
*/

use macroquad::prelude::*;
#[cfg(feature = "scale")]
use crate::modules::scale::mouse_position_world as mouse_position;

#[derive(Debug, PartialEq, Clone)]
pub enum MessageBoxResult {
    ButtonPressed(usize),  // Index of the button pressed
    Closed,                // Dialog was closed via X button or escape key
}

pub struct MessageBox {
    visible: bool,
    title: String,
    message: String,
    buttons: Vec<String>,
    default_button: Option<usize>,
    selected_button: Option<usize>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    title_height: f32,
    button_height: f32,
    padding: f32,
    title_bg_color: Color,
    bg_color: Color,
    title_text_color: Color,
    message_text_color: Color,
    button_bg_color: Color,
    button_hover_color: Color,
    button_text_color: Color,
    close_button_size: f32,
    show_close_button: bool,
    modal: bool,
    modal_color: Color,
    result: Option<MessageBoxResult>,
    dragging: bool,
    drag_offset_x: f32,
    drag_offset_y: f32,
    // Additional fields for font options
    font: Option<Font>,
    title_font_size: f32,
    message_font_size: f32,
    button_font_size: f32,
}

impl MessageBox {
    // Create a new message box
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        buttons: Vec<impl Into<String>>,
        default_button: Option<usize>,
        width: f32,
        height: f32,
    ) -> Self {
        let title = title.into();
        let message = message.into();
        let buttons: Vec<String> = buttons.into_iter().map(|b| b.into()).collect();
        let default_button = if let Some(idx) = default_button {
            if idx < buttons.len() { Some(idx) } else { None }
        } else {
            None
        };
        
        Self {
            visible: false,
            title,
            message,
            buttons,
            default_button,
            selected_button: default_button,
            x: 0.0,
            y: 0.0,
            width,
            height,
            title_height: 30.0,
            button_height: 40.0,
            padding: 15.0,
            title_bg_color: DARKBLUE,
            bg_color: Color::new(0.9, 0.9, 0.9, 1.0), // Light gray
            title_text_color: WHITE,
            message_text_color: BLACK,
            button_bg_color: LIGHTGRAY,
            button_hover_color: SKYBLUE, // Default hover color for better visibility
            button_text_color: BLACK,
            close_button_size: 20.0,
            show_close_button: true,
            modal: true,
            modal_color: Color::new(0.0, 0.0, 0.0, 0.5),
            result: None,
            dragging: false,
            drag_offset_x: 0.0,
            drag_offset_y: 0.0,
            font: None,
            title_font_size: 18.0,
            message_font_size: 16.0,
            button_font_size: 16.0,
        }
    }
    
    // Center the dialog in the screen
    pub fn centered(&mut self) -> &mut Self {
        self.x = (screen_width() - self.width) / 2.0;
        self.y = (screen_height() - self.height) / 2.0;
        self
    }
    
    // Set dialog position
    #[allow(unused)]
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }
    
    // Set dialog size
    #[allow(unused)]
    pub fn set_size(&mut self, width: f32, height: f32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }
    
    // Customize colors
    #[allow(unused)]
    pub fn with_colors(
        &mut self,
        title_bg: Color,
        dialog_bg: Color,
        title_text: Color,
        message_text: Color,
        modal_overlay: Color,
    ) -> &mut Self {
        self.title_bg_color = title_bg;
        self.bg_color = dialog_bg;
        self.title_text_color = title_text;
        self.message_text_color = message_text;
        self.modal_color = modal_overlay;
        self
    }
    
    // Customize button colors
    #[allow(unused)]
    pub fn with_button_colors(
        &mut self,
        button_bg: Color,
        button_hover: Color,
        button_text: Color,
    ) -> &mut Self {
        self.button_bg_color = button_bg;
        self.button_hover_color = button_hover;
        self.button_text_color = button_text;
        self
    }
    
    // Configure modal overlay
    #[allow(unused)]
    pub fn with_modal(&mut self, modal: bool) -> &mut Self {
        self.modal = modal;
        self
    }
    
    // Configure close button
    #[allow(unused)]
    pub fn with_close_button(&mut self, show: bool) -> &mut Self {
        self.show_close_button = show;
        self
    }
    
    // Set custom font
    #[allow(unused)]
    pub fn with_font(&mut self, font: Font) -> &mut Self {
        self.font = Some(font);
        self
    }
    
    // Set font sizes
    #[allow(unused)]
    pub fn with_font_sizes(&mut self, title_size: f32, message_size: f32, button_size: f32) -> &mut Self {
        self.title_font_size = title_size;
        self.message_font_size = message_size; 
        self.button_font_size = button_size;
        self
    }
    
    // Show the dialog
    pub fn show(&mut self) -> &mut Self {
        self.visible = true;
        self.result = None;
        self.selected_button = self.default_button;
        self.dragging = false; // Reset dragging state for clean start
        
        // Auto center the dialog whenever it's shown
        // This ensures proper placement even if the window was resized
        self.centered();
        
        self
    }
    
    // Hide the dialog
    pub fn hide(&mut self) -> &mut Self {
        self.visible = false;
        self.result = None;
        self
    }
    #[allow(unused)]
    // Check if dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    // Get the last result
    #[allow(unused)]
    pub fn get_result(&self) -> Option<&MessageBoxResult> {
        self.result.as_ref()
    }
    
    // Clear the result
    #[allow(unused)]
    pub fn clear_result(&mut self) -> &mut Self {
        self.result = None;
        self
    }
    
    /// Simple method for main game loop - just call this once per frame
    /// This will handle both drawing and updating the message box
    /// Returns Some(result) only when a button is clicked or the dialog is closed
    /// 
    /// Example usage in main loop:
    /// ```
    /// // In main loop:
    /// if let Some(result) = message_box.draw() {
    ///     match result {
    ///         MessageBoxResult::ButtonPressed(0) => {
    ///             // OK button pressed
    ///             println!("OK clicked!");
    ///         },
    ///         #[allow(unused)]
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn draw(&mut self) -> Option<MessageBoxResult> {
        // Don't do anything if not visible
        if !self.visible {
            return None;
        }
        
        // First, draw and update the message box
        let result = self.update_and_draw();
        
        // If we got a result, hide the dialog automatically
        if result.is_some() {
            self.hide();
        }
        
        result
    }
       
    // Update and draw the message box, returning a result if a button was clicked
    pub fn update_and_draw(&mut self) -> Option<MessageBoxResult> {
        if !self.visible {
            return None;
        }
        
        // Draw modal background if enabled
        if self.modal {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), self.modal_color);
            
            // Consume any mouse clicks outside the dialog
            let mouse_in_dialog = self.is_mouse_over_rect(
                self.x, self.y, self.width, self.height
            );
            
            // If mouse is clicked outside dialog and modal is enabled, consume the event
            if !mouse_in_dialog && is_mouse_button_pressed(MouseButton::Left) {
                // Optional: make dialog flash or shake slightly to indicate it needs attention
                // For now, we just consume the click
            }
        }
        
        // Handle dragging
        self.handle_dragging();
        
        // Draw dialog background
        draw_rectangle(self.x, self.y, self.width, self.height, self.bg_color);
        
        // Draw title bar
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.title_height,
            self.title_bg_color
        );
        
        // Draw title text
        let title_size = measure_text(&self.title, self.font.as_ref(), self.title_font_size as u16, 1.0);
        draw_text_ex(
            &self.title,
            self.x + self.padding,
            self.y + (self.title_height + title_size.height) / 2.0,
            TextParams {
                font: self.font.as_ref(),
                font_size: self.title_font_size as u16,
                color: self.title_text_color,
                ..Default::default()
            }
        );
        
        // Draw close button if enabled
        if self.show_close_button {
            let close_x = self.x + self.width - self.close_button_size - self.padding;
            let close_y = self.y + (self.title_height - self.close_button_size) / 2.0;
            
            // Check if mouse is over close button using helper function
            let is_over_close = self.is_mouse_over_rect(
                close_x, close_y, self.close_button_size, self.close_button_size
            );
            
            // Draw X symbol with appropriate color based on hover state
            let close_color = self.get_hover_color(
                is_over_close, 
                self.title_text_color, 
                Color::new(1.0, 0.3, 0.3, 1.0) // Bright red for hover
            );
            
            // Draw X lines
            let thickness = 2.0;
            draw_line(close_x, close_y, close_x + self.close_button_size, close_y + self.close_button_size, thickness, close_color);
            draw_line(close_x, close_y + self.close_button_size, close_x + self.close_button_size, close_y, thickness, close_color);
            
            // Handle close button click
            if is_over_close && is_mouse_button_pressed(MouseButton::Left) {
                self.result = Some(MessageBoxResult::Closed);
                return self.result.clone();
            }
        }
        
        // Draw message
        let message_y = self.y + self.title_height + self.padding;
        
        // Handle multiline messages
        let max_line_width = self.width - 2.0 * self.padding;
        let lines = self.wrap_text(&self.message, max_line_width, self.message_font_size);
        
        let line_height = self.message_font_size * 1.2;
        for (i, line) in lines.iter().enumerate() {
            draw_text_ex(
                line,
                self.x + self.padding,
                message_y + i as f32 * line_height + self.message_font_size,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: self.message_font_size as u16,
                    color: self.message_text_color,
                    ..Default::default()
                }
            );
        }
        
        // Draw buttons
        let button_spacing = 10.0;
        let num_buttons = self.buttons.len();
        
        if num_buttons > 0 {
            let total_button_width: f32 = if num_buttons == 1 {
                self.width * 0.33 // Single button takes 1/3 of dialog width
            } else {
                num_buttons as f32 * 100.0 + (num_buttons - 1) as f32 * button_spacing
            };
            
            let first_button_x = self.x + (self.width - total_button_width) / 2.0;
            let button_y = self.y + self.height - self.button_height - self.padding;
            
            // Collect button details for drawing
            let mut button_details = Vec::with_capacity(num_buttons);
            
            for i in 0..num_buttons {
                let button_width = if num_buttons == 1 {
                    self.width * 0.33 // Single button takes 1/3 of dialog width
                } else {
                    100.0 // Multiple buttons have fixed width
                };
                
                let button_x = first_button_x + i as f32 * (button_width + button_spacing);
                
                // Check if mouse is over this button
                let is_over_button = self.is_mouse_over_rect(
                    button_x, button_y, button_width, self.button_height
                );
                
                // Update selected button on hover for keyboard navigation
                if is_over_button {
                    self.selected_button = Some(i);
                }
                
                // Store button text and position details
                button_details.push((
                    i, 
                    self.buttons[i].clone(), 
                    button_x, 
                    button_y, 
                    button_width, 
                    is_over_button
                ));
            }
            
            // Now draw all buttons and check for clicks
            for (i, button_text, button_x, button_y, button_width, is_over_button) in button_details {
                // Draw button with appropriate colors
                draw_rectangle(
                    button_x, button_y, button_width, self.button_height,
                    self.get_hover_color(is_over_button, self.button_bg_color, self.button_hover_color)
                );
                
                // Draw border
                draw_rectangle_lines(
                    button_x, button_y, button_width, self.button_height, 1.0,
                    self.get_hover_color(is_over_button, DARKGRAY, BLUE)
                );
                
                // Draw text
                let text_size = measure_text(&button_text, self.font.as_ref(), self.button_font_size as u16, 1.0);
                draw_text_ex(
                    &button_text,
                    button_x + (button_width - text_size.width) / 2.0,
                    button_y + (self.button_height + text_size.height) / 2.0,
                    TextParams {
                        font: self.font.as_ref(),
                        font_size: self.button_font_size as u16,
                        color: self.get_hover_color(is_over_button, self.button_text_color, BLACK),
                        ..Default::default()
                    }
                );
                
                // Handle button click
                if is_over_button && is_mouse_button_pressed(MouseButton::Left) {
                    self.result = Some(MessageBoxResult::ButtonPressed(i));
                    return self.result.clone();
                }
            }
        }
        
        // Handle keyboard navigation
        if is_key_pressed(KeyCode::Tab) {
            if let Some(selected) = self.selected_button {
                // Shift selection to next button (with wrap-around)
                let next = (selected + 1) % num_buttons;
                self.selected_button = Some(next);
            } else if num_buttons > 0 {
                // Select first button if none selected
                self.selected_button = Some(0);
            }
        }
        
        // Handle Enter key to activate selected button
        if is_key_pressed(KeyCode::Enter) {
            if let Some(selected) = self.selected_button {
                self.result = Some(MessageBoxResult::ButtonPressed(selected));
                return self.result.clone();
            }
        }
        
        // Handle Escape key to close dialog
        if is_key_pressed(KeyCode::Escape) {
            self.result = Some(MessageBoxResult::Closed);
            return self.result.clone();
        }
        
        self.result.clone()
    }
    
    // Helper function to determine if mouse is over a rectangular area
    fn is_mouse_over_rect(&self, rect_x: f32, rect_y: f32, rect_width: f32, rect_height: f32) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        mouse_x >= rect_x && mouse_x <= rect_x + rect_width &&
        mouse_y >= rect_y && mouse_y <= rect_y + rect_height
    }
    
    // Helper function to get hover color for any interactive element
    fn get_hover_color(&self, is_hovered: bool, normal_color: Color, hover_color: Color) -> Color {
        if is_hovered { hover_color } else { normal_color }
    }
    
    // Handle dialog dragging
    fn handle_dragging(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        
        // Check if mouse is in title bar area using our helper
        let in_title_bar = self.is_mouse_over_rect(
            self.x, self.y, self.width, self.title_height
        );
        
        // Start dragging when clicking on title bar
        if in_title_bar && is_mouse_button_pressed(MouseButton::Left) {
            self.dragging = true;
            self.drag_offset_x = mouse_x - self.x;
            self.drag_offset_y = mouse_y - self.y;
        }
        
        // Continue dragging while button is held
        if self.dragging {
            if is_mouse_button_down(MouseButton::Left) {
                self.x = mouse_x - self.drag_offset_x;
                self.y = mouse_y - self.drag_offset_y;
                
                // Keep dialog within screen bounds
                self.x = self.x.max(0.0).min(screen_width() - self.width);
                self.y = self.y.max(0.0).min(screen_height() - self.height);
            } else {
                // Stop dragging when button is released
                self.dragging = false;
            }
        }
    }
    
    // Wrap text to fit within width
    fn wrap_text(&self, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            let word_with_space = if current_line.is_empty() {
                word.to_string()
            } else {
                format!(" {}", word)
            };
            
            let test_line = format!("{}{}", current_line, word_with_space);
            let test_width = measure_text(&test_line, self.font.as_ref(), font_size as u16, 1.0).width;
            
            if test_width <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                current_line = word.to_string();
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        // Handle case where no lines were created (empty message)
        if lines.is_empty() {
            lines.push(String::new());
        }
        
        lines
    }
    
    /// Show the dialog and return true when it has a result
    /// This should be used when you want to wait for user input before proceeding
    /// Example: 
    /// ```
    /// if !confirmation_box.is_active() {
    ///     confirmation_box.show();
    /// }
    /// if let Some(result) = confirmation_box.get_result() {
    ///     // Handle result
    /// }
    /// ```
    #[allow(unused)]
    pub fn is_active(&self) -> bool {
        self.visible && self.result.is_none()
    }
    
    /// Process the message box within game loop, returning true when processed
    /// Usage example: if message_box.process() { /* handle result here */ }
    #[allow(unused)]
    pub fn process(&mut self) -> bool {
        if let Some(result) = self.update_and_draw() {
            self.hide();
            self.result = Some(result);
            true
        } else {
            false
        }
    }
    
    // Convenience functions for common dialog types
    
    /// Create an information dialog with just an OK button
    #[allow(unused)]
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut dialog = Self::new(
            title,
            message,
            vec!["OK"],
            Some(0),
            400.0, 200.0
        );
        dialog.centered();
        
        // Set very obvious hover color for info box buttons - bright blue like the X turns bright red
        dialog.button_hover_color = Color::new(0.3, 0.5, 1.0, 1.0); // Strong blue hover (more obvious)
        dialog
    }
    
    /// Create a yes/no confirmation dialog
    /// 
    /// This creates a dialog with two buttons: Yes (index 0) and No (index 1)
    /// When handling results, you MUST include the catch-all pattern for full Rust pattern matching:
    /// ```
    /// match result {
    ///     MessageBoxResult::ButtonPressed(0) => { /* Yes button */ },
    ///     MessageBoxResult::ButtonPressed(1) => { /* No button */ },
    ///     #[allow(unused)]
    ///     MessageBoxResult::ButtonPressed(_) => { /* Required catch-all */ },
    ///     MessageBoxResult::Closed => { /* X button or Escape key */ }
    /// }
    /// ```
    #[allow(unused)]
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut dialog = Self::new(
            title,
            message,
            vec!["Yes", "No"],
            Some(0), // Default to "Yes"
            400.0, 200.0
        );
        dialog.centered();
        dialog
    }
    
    /// Create a dialog with yes/no/cancel options
    /// 
    /// This creates a dialog with three buttons: Yes (index 0), No (index 1), and Cancel (index 2)
    /// You should handle all three button indices in your match statement.
    #[allow(unused)]
    pub fn confirm_with_cancel(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut dialog = Self::new(
            title,
            message,
            vec!["Yes", "No", "Cancel"],
            Some(0), // Default to "Yes"
            450.0, 200.0
        );
        dialog.centered();
        dialog
    }
    
    /// Create a custom dialog with the specified buttons
    #[allow(unused)]
    pub fn custom(title: impl Into<String>, message: impl Into<String>, buttons: Vec<impl Into<String>>, default_button: Option<usize>) -> Self {
        let buttons: Vec<String> = buttons.into_iter().map(|b| b.into()).collect();
        let width = (buttons.len() as f32 * 120.0).max(400.0);
        
        let mut dialog = Self::new(
            title,
            message,
            buttons,
            default_button,
            width, 200.0
        );
        dialog.centered();
        dialog
    }
}