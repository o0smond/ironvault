/*
Made by: Mathew Dusome
May 2 2025
To import you need:
Adds a label object

In your mod.rs file located in the modules folder add the following to the end of the file
        pub mod label;
    

Add with the other use statements
    use crate::modules::label::Label;

Then to use this you would put the following above the loop: 
    let lbl_out = Label::new("Hello\nWorld", 50.0, 100.0, 30);
Where the numbers are x, y, font size
You can also set the colors of the text box by using:
     lbl_out.with_colors(WHITE, Some(DARKGRAY));
Where the colors are text color and background color respectively.

You can also specify a custom font with:
     lbl_out.with_font(font);

You can add rounded corners to the label with:
     lbl_out.with_round(10.0);
Where the value is the corner radius in pixels.

You can add a border to the label with:
     lbl_out.with_border(RED, 2.0);
Where the first value is the border color and the second is the thickness.

You can set a fixed size for the label with:
     lbl_out.with_fixed_size(200.0, 100.0);
Where the values are width and height in pixels.

You can also set the text alignment within a fixed-size label with:
     lbl_out.with_alignment(modules::label::TextAlign::Center);
Options are modules::label::TextAlign::Left, objects::label::TextAlign::Center, and objects::label::TextAlign::Right.

To access the label's position:
     let x = lbl_out.get_x();
     let y = lbl_out.get_y();
     let position = lbl_out.get_position(); // Returns a Vec2 with both x and y

To change the label's position:
     lbl_out.set_position(150.0, 250.0);
This changes the x and y coordinates of the label.

To change the font size:
     lbl_out.set_font_size(24);
This changes the font size of the label and recalculates its dimensions.

To change the label's text:
     lbl_out.set_text("New text content");

To control the visibility of a label:
     lbl_out.set_visible(false); // Hide the label
     lbl_out.set_visible(true);  // Show the label
     let is_visible = lbl_out.is_visible(); // Check if the label is visible
     lbl_out.toggle_visibility(); // Toggle between visible and hidden
You can also set visibility during creation with:
     lbl_out.with_visibility(false);

Example:
     // Load font once at the beginning of your program
     let font = load_ttf_font("assets/love.ttf").await.unwrap();
     
     // Create label and apply custom font
     let mut lbl_out = Label::new("Hello\nWorld", 50.0, 100.0, 30);
     lbl_out.with_colors(WHITE, Some(DARKGRAY))
            .with_font(font.clone())
            .with_round(8.0)
            .with_border(RED, 1.5)
            .with_fixed_size(250.0, 120.0)
            .with_alignment(objects::label::TextAlign::Center)
            .with_visibility(true); // Explicitly set visibility (default is true)
Otherwise the default system font will be used.

Then in the loop you would use:
    lbl_out.draw();
*/
use macroquad::prelude::*;

pub struct Label {
    text: String,
    x: f32,
    y: f32,
    font_size: u16,
    foreground: Color,
    background: Option<Color>,
    line_spacing: f32,
    font: Option<Font>, // Store the font directly since Font is Clone
    corner_radius: f32, // For rounded corners
    border: bool,       // Whether to draw a border
    border_color: Color, // Color of the border
    border_thickness: f32, // Thickness of the border
    visible: bool,      // Whether the label should be drawn
    
    // Fixed size properties
    fixed_width: Option<f32>,
    fixed_height: Option<f32>,
    text_align: TextAlign,
    
    // Cached values for performance
    cached_lines: Vec<String>,
    cached_line_dimensions: Vec<TextDimensions>,
    cached_max_width: f32,
    cached_total_height: f32,
}

// Enum for text alignment within a fixed-size label
#[allow(unused)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Label {
    // Constructor using x and y separately
    pub fn new<T: Into<String>>(text: T, x: f32, y: f32, font_size: u16) -> Self {
        let mut label = Self {
            text: text.into(),
            x,
            y,
            font_size,
            foreground: BLACK, // Default to black
            background: None,  // No background by default
            line_spacing: 1.2,
            font: None,        // Default to None (use system font)
            corner_radius: 0.0, // Default to no rounded corners
            border: false,      // Default to no border
            border_color: BLACK, // Default border color
            border_thickness: 1.0, // Default border thickness
            visible: true,      // Default to visible
            fixed_width: None, // No fixed width by default
            fixed_height: None, // No fixed height by default
            text_align: TextAlign::Left, // Default to left alignment
            cached_lines: Vec::new(),
            cached_line_dimensions: Vec::new(),
            cached_max_width: 0.0,
            cached_total_height: 0.0,
        };
        
        // Calculate and cache text dimensions
        label.calculate_text_dimensions();
        
        label
    }
    
    // Calculate and cache text dimensions
    fn calculate_text_dimensions(&mut self) {
        // Split text into lines and store for later use
        self.cached_lines = self.text.split('\n').map(String::from).collect();
        let line_height = self.font_size as f32 * self.line_spacing;
        
        // Clear previous cached values
        self.cached_line_dimensions.clear();
        self.cached_max_width = 0.0;
        
        // Calculate dimensions for each line
        for line in &self.cached_lines {
            let dimensions = match &self.font {
                Some(font) => measure_text(line, Some(font), self.font_size, 1.0),
                None => measure_text(line, None, self.font_size, 1.0),
            };
            self.cached_line_dimensions.push(dimensions);
            
            // Only update max_width if we don't have a fixed width
            if self.fixed_width.is_none() {
                self.cached_max_width = self.cached_max_width.max(dimensions.width);
            }
        }
        
        // Calculate total height (only if we don't have fixed height)
        if self.fixed_height.is_none() {
            self.cached_total_height = self.cached_lines.len() as f32 * line_height;
        }
    }

    // Method to set foreground and background colors
    #[allow(unused)]
    pub fn with_colors(&mut self, foreground: Color, background: Option<Color>) -> &mut Self {
        self.foreground = foreground;
        self.background = background;
        self
    }

    // Method to set custom font - taking Font by value since it implements Clone
    #[allow(unused)]
    pub fn with_font(&mut self, font: Font) -> &mut Self {
        self.font = Some(font);
        // Recalculate dimensions since font affects text measurements
        self.calculate_text_dimensions();
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

    // Method to set a fixed size for the label
    #[allow(unused)]
    pub fn with_fixed_size(&mut self, width: f32, height: f32) -> &mut Self {
        self.fixed_width = Some(width);
        self.fixed_height = Some(height);
        
        // Since we now have a fixed size, we don't need to recalculate these
        // but we still need line dimensions for alignment
        if self.cached_line_dimensions.is_empty() {
            self.calculate_text_dimensions();
        }
        
        self
    }
    
    // Method to set text alignment (only applies when using fixed width)
    #[allow(unused)]
    pub fn with_alignment(&mut self, alignment: TextAlign) -> &mut Self {
        self.text_align = alignment;
        self
    }

    // Method to set text - now accepts both String and &str
    #[allow(unused)]
    pub fn set_text<T: Into<String>>(&mut self, new_text: T) -> &mut Self {
        self.text = new_text.into();
        
        // Only recalculate if we need to (when not using fixed dimensions)
        // Even with fixed dimensions, we still need to recalculate line dimensions
        // for proper text alignment
        self.calculate_text_dimensions();
        
        self
    }
     // Getter for width (fixed width or max content width)
    #[allow(unused)]
    pub fn get_width(&self) -> Option<f32> {
        match self.fixed_width {
            Some(width) => Some(width),
            None => Some(self.cached_max_width + 10.0) // Same padding as in draw method
        }
    }
    
    // Getter for height (fixed height or calculated content height)
    #[allow(unused)]
    pub fn get_height(&self) -> Option<f32> {
        match self.fixed_height {
            Some(height) => Some(height),
            None => Some(self.cached_total_height)
        }
    }
    
    // Getter for font size
    #[allow(unused)]
    pub fn get_font_size(&self) -> u16 {
        self.font_size
    }
    
    // Getter for the label's text content
    #[allow(unused)]
    pub fn get_text(&self) -> &str {
        &self.text
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
    
    // Getter for visibility
    #[allow(unused)]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    // Setter for position
    #[allow(unused)]
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }
    
    // Setter for font size
    #[allow(unused)]
    pub fn set_font_size(&mut self, font_size: u16) -> &mut Self {
        self.font_size = font_size;
        
        // Recalculate text dimensions since font size affects text measurements
        self.calculate_text_dimensions();
        
        self
    }

    // Setter for visibility
    #[allow(unused)]
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.visible = visible;
        self
    }
    
    // Method to toggle visibility (returns the new visibility state)
    #[allow(unused)]
    pub fn toggle_visibility(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }
    
    // Method to draw the label
    pub fn draw(&self) {
        // Only draw if the label is visible
        if !self.visible {
            return;
        }
        
        let line_height = self.font_size as f32 * self.line_spacing;
        
        // Determine width and height (using fixed values if set, otherwise use content size)
        let width = self.fixed_width.unwrap_or(self.cached_max_width + 10.0);
        let height = self.fixed_height.unwrap_or(self.cached_total_height);
        
        // Calculate positions for all elements
        let bg_x = self.x - 5.0;
        let bg_y = self.y - self.font_size as f32;
        
        // Draw background first
        if let Some(bg) = self.background {
            // Draw a single background for all lines
            if self.corner_radius > 0.0 {
                draw_round_rect(
                    bg_x, bg_y, width, height,
                    self.corner_radius,
                    bg,
                );
            } else {
                draw_rectangle(
                    bg_x, bg_y, width, height,
                    bg,
                );
            }
        }
        
        // Draw border if enabled
        if self.border {
            // Get background color for the inner part of the border
            let bg_color = self.background.unwrap_or(GRAY);
            
            if self.corner_radius > 0.0 {
                // Draw rounded border with the correct background color
                draw_round_rect_border(
                    bg_x, bg_y, width, height,
                    self.corner_radius,
                    self.border_thickness,
                    self.border_color,
                    bg_color,
                );
            } else {
                // Draw regular rectangular border
                draw_rectangle_border(
                    bg_x, bg_y, width, height,
                    self.border_thickness,
                    self.border_color,
                );
            }
        }

        // Draw each line of text
        for (i, (line, dimensions)) in self.cached_lines.iter().zip(self.cached_line_dimensions.iter()).enumerate() {
            let y = self.y + i as f32 * line_height;
            
            // Calculate x position based on alignment (if fixed width is set)
            let x = if let Some(fixed_width) = self.fixed_width {
                match self.text_align {
                    TextAlign::Left => self.x,
                    TextAlign::Center => self.x + (fixed_width / 2.0) - (dimensions.width / 2.0),
                    TextAlign::Right => self.x + fixed_width - dimensions.width - 10.0, // 10.0 for padding
                }
            } else {
                self.x
            };
            
            // Draw the text - use draw_text_ex if we have a custom font
            match &self.font {
                Some(font) => {
                    draw_text_ex(
                        line,
                        x,
                        y,
                        TextParams {
                            font: Some(font),
                            font_size: self.font_size,
                            color: self.foreground,
                            ..Default::default()
                        },
                    );
                },
                None => {
                    // Use the default draw_text function
                    draw_text(line, x, y, self.font_size as f32, self.foreground);
                }
            }
        }
    }
}

// Function to draw a rectangle with rounded corners - optimized version
#[allow(unused)]
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

// New border drawing method using filled rectangles instead of lines
// This approach ensures consistent border thickness on all sides
#[allow(unused)]
fn draw_rectangle_border(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    // Top border
    draw_rectangle(x, y, w, thickness, color);
    // Bottom border
    draw_rectangle(x, y + h - thickness, w, thickness, color);
    // Left border
    draw_rectangle(x, y + thickness, thickness, h - (thickness * 2.0), color);
    // Right border
    draw_rectangle(x + w - thickness, y + thickness, thickness, h - (thickness * 2.0), color);
}

// New function to draw rounded rectangle borders with consistent thickness
#[allow(unused)]
fn draw_round_rect_border(x: f32, y: f32, w: f32, h: f32, radius: f32, thickness: f32, color: Color, bg_color: Color) {
    if radius <= 0.0 {
        // Use our new rectangle border function for non-rounded corners
        draw_rectangle_border(x, y, w, h, thickness, color);
        return;
    }
    
    // Draw outer rounded rectangle
    draw_round_rect(x, y, w, h, radius, color);
    
    // Draw inner rounded rectangle with background color
    let inner_x = x + thickness;
    let inner_y = y + thickness;
    let inner_w = w - (thickness * 2.0);
    let inner_h = h - (thickness * 2.0);
    let inner_radius = (radius - thickness).max(0.0);
    
    draw_round_rect(inner_x, inner_y, inner_w, inner_h, inner_radius, bg_color);
}
