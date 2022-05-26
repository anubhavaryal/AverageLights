use scrap::{Capturer, Display};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::io::ErrorKind::WouldBlock;
use std::time::Duration;
use tokio::time;
use toml::Value;

mod manager;

#[derive(Deserialize)]
struct Config {
    prefix: String,
    num_lights: usize,
    light_wait_millis: u64,
    capture_wait_millis: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // read config file
    let contents = fs::read_to_string("config.toml").expect("Unable to read 'config.toml' file.");
    let config: Config = toml::from_str(&contents).unwrap();

    println!("Successfully loaded 'config.toml' file with the following values.");
    println!("prefix: {}", config.prefix);
    println!("num_lights: {}", config.num_lights);
    println!("light_wait_millis: {}", config.light_wait_millis);
    println!("capture_wait_millis: {}", config.capture_wait_millis);
    println!();

    // find display to use
    println!("Finding display.");
    let display = Display::primary().expect("Unable to find display.");

    println!("Create new capturer.");
    let mut capturer = Capturer::new(display).expect("Unable to find capturer.");

    let (width, height) = (capturer.width(), capturer.height());

    // create a new light manager
    let mut manager = manager::LightManager::new();

    // connect to all lights
    println!("Attempting to connect to all lights.");
    manager
        .connect_to_lights(&config.prefix, config.num_lights, config.light_wait_millis)
        .await?;

    println!("Connected to all lights.");
    println!("Starting program loop.");

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
        time::sleep(Duration::from_millis(config.capture_wait_millis)).await;
    }

    Ok(())
}
