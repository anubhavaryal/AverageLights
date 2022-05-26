use scrap::{Capturer, Display};
use std::error::Error;
use std::io::ErrorKind::WouldBlock;
use std::time::Duration;
use tokio::time;

mod api;

const WAIT_TIME: u64 = 100; // wait 100ms between frames

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // find display to use
    let display = Display::primary().unwrap();
    let mut capturer = Capturer::new(display).unwrap();
    let (width, height) = (capturer.width(), capturer.height());

    // create a new light manager
    let mut manager = api::LightManager::new();

    // connect to all lights
    manager.connect_to_lights("Minger", 2, 2000).await?;

    // take a screen capture
    loop {
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // try again
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        // average the pixel values
        let stride = buffer.len() / height;

        let mut r: u32 = 0;
        let mut g: u32 = 0;
        let mut b: u32 = 0;

        for y in 0..height {
            for x in 0..width {
                // 4 channels per pixel (BGRA)
                let i = y * stride + x * 4;
                b += u32::from(buffer[i]);
                g += u32::from(buffer[i + 1]);
                r += u32::from(buffer[i + 2]);
            }
        }

        // find average pixel values for each channel
        r /= (width * height) as u32;
        g /= (width * height) as u32;
        b /= (width * height) as u32;

        // set light color to average color values
        manager.set_color(r as u8, g as u8, b as u8).await?;

        // wait before calculating next frame
        time::sleep(Duration::from_millis(WAIT_TIME)).await;
    }

    Ok(())
}
