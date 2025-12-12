/*
Made by: Mathew Dusome
Feb 6 2025
Add a grid object for placement
To import you need:

In your mod.rs file located in the modules folder add the following to the end of the file:
    pub mod grid;
    
Then add the following with the use commands:
use crate::modules::grid::draw_grid;

Then in side the loop I would use:
    draw_grid(50.0, BROWN);
    
Note: When using with the scale module, make sure to call draw_grid AFTER use_virtual_resolution
*/
use macroquad::prelude::*;

pub fn draw_grid(grid_size: f32, color: Color) {
    #[cfg(feature = "scale")]
    {
        // When the scale feature is enabled, use virtual resolution-aware grid drawing
        draw_grid_with_scale(grid_size, color);
    }

    #[cfg(not(feature = "scale"))]
    {
        // Default grid drawing when scale feature is not enabled
        draw_grid_standard(grid_size, color);
    }
}

// Standard grid drawing function that uses raw screen dimensions
fn draw_grid_standard(grid_size: f32, color: Color) {
    let screen_width = screen_width();
    let screen_height = screen_height();
    
    // Draw vertical lines and labels
    for x in (0..screen_width as i32).step_by(grid_size as usize) {
        draw_line(x as f32, 0.0, x as f32, screen_height, 1.0, color);
        draw_text(&format!("{}", x), x as f32 + 2.0, 12.0, 16.0, color);
    }
    
    // Draw horizontal lines and labels
    for y in (0..screen_height as i32).step_by(grid_size as usize) {
        draw_line(0.0, y as f32, screen_width, y as f32, 1.0, color);
        draw_text(&format!("{}", y), 2.0, y as f32 + 12.0, 16.0, color);
    }
}

// Scale-aware grid drawing function that respects virtual resolution
#[cfg(feature = "scale")]
fn draw_grid_with_scale(grid_size: f32, color: Color) {
    // When using virtual resolution, we draw grid based on the virtual dimensions
    // Get virtual dimensions from scale module
    if let Ok(resolution) = crate::modules::scale::VIRTUAL_RESOLUTION.try_with(|res| *res.borrow()) {
        let (virtual_width, virtual_height) = resolution;
        
        // Draw vertical lines and labels covering the entire virtual space
        for x in (0..=virtual_width as i32).step_by(grid_size as usize) {
            draw_line(x as f32, 0.0, x as f32, virtual_height, 1.0, color);
            draw_text(&format!("{}", x), x as f32 + 2.0, 12.0, 16.0, color);
        }
        
        // Draw horizontal lines and labels covering the entire virtual space
        for y in (0..=virtual_height as i32).step_by(grid_size as usize) {
            draw_line(0.0, y as f32, virtual_width, y as f32, 1.0, color);
            draw_text(&format!("{}", y), 2.0, y as f32 + 12.0, 16.0, color);
        }
    } else {
        // Fallback to standard grid if we can't access virtual resolution
        draw_grid_standard(grid_size, color);
    }
}
