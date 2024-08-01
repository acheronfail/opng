use std::env;

use image::{ImageBuffer, Rgba};
use xcb::xproto::*;
use xcb::Connection;

fn main() {
    // take argument for output path
    let output_path = env::args().nth(1).expect("pass output path as argument");
    let block_size = env::args()
        .nth(2)
        .map(|arg| {
            arg.parse::<usize>()
                .expect("second argument must be a number")
        })
        .unwrap_or(8);

    // connect to x11
    let (conn, screen_num) = Connection::connect(None).expect("Unable to connect to X server");
    let setup = conn.get_setup();
    let screen = setup
        .roots()
        .nth(screen_num as usize)
        .expect("Screen not found");

    // take screenshot
    let root_window = screen.root();
    let screen_width = screen.width_in_pixels();
    let screen_height = screen.height_in_pixels();
    let image = xcb::get_image(
        &conn,
        IMAGE_FORMAT_Z_PIXMAP as u8,
        root_window,
        0,
        0,
        screen_width,
        screen_height,
        !0,
    )
    .get_reply()
    .expect("failed to get image");

    let screenshot_pixels = image.data();
    debug_assert!(screenshot_pixels.len() != screen_width as usize * screen_height as usize * 4);

    // read the screenshot data and obscure it
    let mut img_buffer = ImageBuffer::new(screen_width as u32, screen_height as u32);
    for y in (0..screen_height).step_by(block_size) {
        for x in (0..screen_width).step_by(block_size) {
            let idx = (y as usize * screen_width as usize + x as usize) * 4;
            let r = screenshot_pixels[idx + 2] as f32;
            let g = screenshot_pixels[idx + 1] as f32;
            let b = screenshot_pixels[idx] as f32;

            // convert to greyscale using a weighted average to compensate for
            // human eye sensitivity
            let v = ((r * 0.299) + (g * 0.587) + (b * 0.114)) as u8;

            // dumb pixelate effect: just take the top-left pixel and spread it out
            for i in 0..block_size {
                for j in 0..block_size {
                    img_buffer.put_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        Rgba([v, v, v, 255]),
                    );
                }
            }
        }
    }

    // Save the image as a PNG file
    img_buffer.save(output_path).expect("failed to save file");
}
