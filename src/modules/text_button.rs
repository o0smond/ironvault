/*
Made by: Mathew Dusome
May 5 2025
To import you need:
Adds a button object

In your mod.rs file located in the modules folder add the following to the end of the file:
    pub mod text_button;

Then with the other use commands add:
use crate::modules::text_button::TextButton;

Then add the following to the top of your file:

Then above the loop section to use you would go:
   
    let btn_text = TextButton::new(
        100.0,
        200.0,
        200.0,
        60.0,
        "Click Me",
        BLUE,
        GREEN,
        30
    );

You can customize the text colors with:
    btn_text.with_text_color(WHITE);        // Sets the normal text color
    btn_text.with_hover_text_color(YELLOW); // Sets the text color when hovering

You can also specify a custom font with:
    btn_text.with_font(my_font.clone());
Otherwise the default system font will be used.

You can add rounded corners to the button with:
    btn_text.with_round(10.0);
Where the value is the corner radius in pixels.

You can add a border to the button with:
    btn_text.with_border(RED, 2.0);
Where the first value is the border color and the second is the thickness.

To access the button's position:
    let x = btn_text.get_x();
    let y = btn_text.get_y();
    let position = btn_text.get_position(); // Returns a Vec2 with both x and y

To change the button's position:
    btn_text.update_position(150.0, 250.0, None, None);
Where the first two values are x and y positions, and the optional values are for width and height.

To change the button's text:
    btn_text.set_text("New Text");

Then in the loop you would use:
if btn_text.click() {

}

Note: For buttons with transparent backgrounds (set normal_color with alpha=0), 
only the text area is clickable, not the entire button area.
*/
use macroquad::prelude::*;
#[cfg(feature = "scale")]
use crate::modules::scale::mouse_position_world as mouse_position;

// Custom struct for ButtonText
pub struct TextButton {
    x: f32,              // Now private
    y: f32,              // Now private
    pub width: f32,
    pub height: f32,
    text: String, // Now private
    pub enabled: bool,
    pub normal_color: Color,
    pub hover_color: Color,
    off_color: Color,
    pub text_color: Color,
    pub hover_text_color: Color, // Added hover text color
    pub font_size: u16,
    pub font: Option<Font>, // Store the font directly since Font is Clone
    pub corner_radius: f32, // For rounded corners
    pub border: bool,       // Whether to draw a border
    pub border_color: Color, // Color of the border
    pub border_thickness: f32, // Thickness of the border
    
    // Cached values for performance
    cached_text_width: f32,
    cached_text_position: Vec2,
    cached_rect: Rect,
    pub visible: bool,
}

impl TextButton {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: impl Into<String>, normal_color: Color, hover_color: Color, font_size: u16) -> Self {
        let enabled = true;
        let off_color = lerp_color(normal_color, GRAY, 0.5);
        let text_string = text.into();
        let text_color = WHITE; // Default text color
        
        // Pre-calculate and cache values
        let cached_text_width = measure_text(&text_string, None, font_size, 1.0).width;
        let cached_text_position = Vec2::new(
            x + (width / 2.0) - (cached_text_width / 2.0),
            y + (height / 2.0),
        );
        let cached_rect = Rect::new(x, y, width, height);
        
        Self {
            x,
            y,
            width,
            height,
            text: text_string.to_string(),
            enabled,
            normal_color,
            hover_color,
            off_color,
            text_color,
            hover_text_color: text_color, // Default hover text color to regular text color
            font_size,
            font: None, // Default to None (use system font)
            corner_radius: 0.0, // Default to no rounded corners
            border: false, // Default to no border
            border_color: BLACK, // Default border color
            border_thickness: 1.0, // Default border thickness
            cached_text_width,
            cached_text_position,
            cached_rect,
            visible: true,
        }
    }

    // Method to set custom font - taking Font by value since it implements Clone
    #[allow(unused)]
    pub fn with_font(&mut self, font: Font) -> &mut Self {
        self.font = Some(font.clone());
        
        // Update cached text width with the new font
        self.cached_text_width = measure_text(&self.text, Some(&font), self.font_size, 1.0).width;
        
        // Update text position based on new measurement
        self.cached_text_position = Vec2::new(
            self.x + (self.width / 2.0) - (self.cached_text_width / 2.0),
            self.y + (self.height / 2.0),
        );
        
        self
    }

    // Method to set rounded corners
    #[allow(unused)]
    pub fn with_round(&mut self, radius: f32) -> &mut Self {
        self.corner_radius = radius;
        self
    }

    // Method to add border with custom color and thickness
    #[allow(unused)]
    pub fn with_border(&mut self, color: Color, thickness: f32) -> &mut Self {
        self.border = true;
        self.border_color = color;
        self.border_thickness = thickness;
        self
    }
    
    // Method to set hover text color
    #[allow(unused)]
    pub fn with_hover_text_color(&mut self, color: Color) -> &mut Self {
        self.hover_text_color = color;
        self
    }

    // Method to set text color
    #[allow(unused)]
    pub fn with_text_color(&mut self, color: Color) -> &mut Self {
        self.text_color = color;
        if self.hover_text_color == WHITE { // Only update if it wasn't explicitly set
            self.hover_text_color = color;
        }
        self
    }
    
    // Getter for x position
    #[allow(unused)]
    pub fn get_x(&self) -> f32 {
        self.x
    }
    
    // Getter for y position
    #[allow(unused)]
    pub fn get_y(&self) -> f32 {
        self.y
    }
    
    // Getter for position as Vec2
    #[allow(unused)]
    pub fn get_position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
    
    // Getter for the button text
    #[allow(unused)]
    pub fn get_text(&self) -> &str {
        &self.text
    }
    
    // Setter for the button text - updates cached measurements
    #[allow(unused)]
    pub fn set_text<T: Into<String>>(&mut self, text: T) -> &mut Self {
        self.text = text.into();
        
        // Update cached text measurements
        self.cached_text_width = match &self.font {
            Some(font) => measure_text(&self.text, Some(font), self.font_size, 1.0).width,
            None => measure_text(&self.text, None, self.font_size, 1.0).width,
        };
        
        // Update text position
        self.cached_text_position = Vec2::new(
            self.x + (self.width / 2.0) - (self.cached_text_width / 2.0),
            self.y + (self.height / 2.0),
        );
        
        self
    }
    
    // Update method to recalculate values when position or size changes
    #[allow(unused)]
    pub fn update_position(&mut self, x: f32, y: f32, width: Option<f32>, height: Option<f32>) -> &mut Self {
        self.x = x;
        self.y = y;
        
        if let Some(w) = width {
            self.width = w;
        }
        
        if let Some(h) = height {
            self.height = h;
        }
        
        // Update cached rectangle
        self.cached_rect = Rect::new(self.x, self.y, self.width, self.height);
        
        // Update text position
        self.cached_text_position = Vec2::new(
            self.x + (self.width / 2.0) - (self.cached_text_width / 2.0),
            self.y + (self.height / 2.0),
        );
        
        self
    }

    pub fn click(&self) -> bool {
        if !self.visible {
            return false; // If not visible, don't process clicks
        }
        // Get mouse position
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos = Vec2::new(mouse_x, mouse_y);

        // Check if the background is transparent (alpha is 0)
        let is_background_transparent = self.normal_color.a == 0.0;
        
        // Determine is_hovered based on background transparency
        let is_hovered = if is_background_transparent {
            // If transparent, only detect clicks on the text area
            let text_height = self.font_size as f32; // Approximate text height
            let text_rect = Rect::new(
                self.cached_text_position.x,
                self.cached_text_position.y - text_height,
                self.cached_text_width,
                text_height
            );
            text_rect.contains(mouse_pos)
        } else {
            // Otherwise use the full button area
            self.cached_rect.contains(mouse_pos)
        };

        // Draw the text button (change color on hover)
        let button_color = if self.enabled {
            if is_hovered {
                self.hover_color
            } else {
                self.normal_color
            }
        } else {
            self.off_color
        };

        // Draw the button with or without rounded corners
        if self.corner_radius > 0.0 {
            draw_round_rect(self.x, self.y, self.width, self.height, self.corner_radius, button_color);
            
            // Draw rounded border if enabled
            if self.border {
                draw_round_rect_lines(self.x, self.y, self.width, self.height, 
                                     self.corner_radius, self.border_thickness, self.border_color);
            }
        } else {
            draw_rectangle(self.x, self.y, self.width, self.height, button_color);
            
            // Draw regular border if enabled
            if self.border {
                draw_rectangle_lines(self.x, self.y, self.width, self.height, 
                                    self.border_thickness, self.border_color);
            }
        }

        // Draw the text with the appropriate font using cached position
        let current_text_color = if self.enabled {
            if is_hovered {
                self.hover_text_color
            } else {
                self.text_color
            }
        } else {
            // Use a dimmed text color for disabled state
            Color::new(self.text_color.r, self.text_color.g, self.text_color.b, 0.5)
        };
        
        match &self.font {
            Some(font) => {
                draw_text_ex(
                    &self.text,
                    self.cached_text_position.x,
                    self.cached_text_position.y,
                    TextParams {
                        font: Some(font),
                        font_size: self.font_size,
                        color: current_text_color,
                        ..Default::default()
                    },
                );
            },
            None => {
                // Use the default draw_text function
                draw_text(
                    &self.text,
                    self.cached_text_position.x,
                    self.cached_text_position.y,
                    self.font_size.into(),
                    current_text_color,
                );
            }
        }

        // After drawing, check if the button was clicked
        is_hovered && self.enabled && is_mouse_button_pressed(MouseButton::Left)
    }
}

// Function to draw a rectangle with rounded corners - optimized version
fn draw_round_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    // Precompute corner positions
    let top_left = Vec2::new(x + radius, y + radius);
    let top_right = Vec2::new(x + w - radius, y + radius);
    let bottom_left = Vec2::new(x + radius, y + h - radius);
    let bottom_right = Vec2::new(x + w - radius, y + h - radius);
    
    // Draw center rectangle
    draw_rectangle(x + radius, y, w - 2.0 * radius, h, color);
    
    // Draw the side rectangles
    draw_rectangle(x, y + radius, radius, h - 2.0 * radius, color);
    draw_rectangle(x + w - radius, y + radius, radius, h - 2.0 * radius, color);
    
    // Draw the four corner circles (could be batched in a real engine)
    draw_circle(top_left.x, top_left.y, radius, color);     // Top-left
    draw_circle(top_right.x, top_right.y, radius, color);   // Top-right
    draw_circle(bottom_left.x, bottom_left.y, radius, color);  // Bottom-left
    draw_circle(bottom_right.x, bottom_right.y, radius, color); // Bottom-right
}

// Function to draw rounded rectangle borders - optimized version
fn draw_round_rect_lines(x: f32, y: f32, w: f32, h: f32, radius: f32, thickness: f32, color: Color) {
    // Precompute corner positions
    let top_left = Vec2::new(x + radius, y + radius);
    let top_right = Vec2::new(x + w - radius, y + radius);
    let bottom_left = Vec2::new(x + radius, y + h - radius);
    let bottom_right = Vec2::new(x + w - radius, y + h - radius);
    
    // Draw the horizontal and vertical lines
    draw_line(top_left.x, y, top_right.x, y, thickness, color);          // Top
    draw_line(bottom_left.x, y + h, bottom_right.x, y + h, thickness, color); // Bottom
    draw_line(x, top_left.y, x, bottom_left.y, thickness, color);           // Left
    draw_line(x + w, top_right.y, x + w, bottom_right.y, thickness, color);  // Right
    
    // Draw the four corner arcs with fewer segments for better performance
    let segments = 8; // Reduced from 16 - still looks good but fewer draw calls
    let step = std::f32::consts::PI / 2.0 / segments as f32;
    
    // Pre-calculate sin/cos values for angle offsets to avoid redundant calculations
    let angles: Vec<(f32, f32)> = (0..=segments)
        .map(|i| {
            let angle = i as f32 * step;
            (angle.cos(), angle.sin())
        })
        .collect();
    
    // Draw arcs for each corner
    // Top-left corner: PI to PI*3/2
    for i in 0..segments {
        let (cos1, sin1) = angles[i];
        let (cos2, sin2) = angles[i+1];
        draw_line(
            top_left.x - radius * cos1,
            top_left.y - radius * sin1,
            top_left.x - radius * cos2,
            top_left.y - radius * sin2,
            thickness,
            color
        );
    }
    
    // Top-right corner: PI*3/2 to PI*2
    for i in 0..segments {
        let (cos1, sin1) = angles[i];
        let (cos2, sin2) = angles[i+1];
        draw_line(
            top_right.x + radius * sin1,
            top_right.y - radius * cos1,
            top_right.x + radius * sin2,
            top_right.y - radius * cos2,
            thickness,
            color
        );
    }
    
    // Bottom-left corner: PI/2 to PI
    for i in 0..segments {
        let (cos1, sin1) = angles[i];
        let (cos2, sin2) = angles[i+1];
        draw_line(
            bottom_left.x - radius * sin1,
            bottom_left.y + radius * cos1,
            bottom_left.x - radius * sin2,
            bottom_left.y + radius * cos2,
            thickness, 
            color
        );
    }
    
    // Bottom-right corner: 0 to PI/2
    for i in 0..segments {
        let (cos1, sin1) = angles[i];
        let (cos2, sin2) = angles[i+1];
        draw_line(
            bottom_right.x + radius * cos1,
            bottom_right.y + radius * sin1,
            bottom_right.x + radius * cos2,
            bottom_right.y + radius * sin2,
            thickness,
            color
        );
    }
}

fn lerp_color(c1: Color, c2: Color, factor: f32) -> Color {
    Color::new(c1.r * (1.0 - factor) + c2.r * factor, c1.g * (1.0 - factor) + c2.g * factor, c1.b * (1.0 - factor) + c2.b * factor, 1.0)
}
