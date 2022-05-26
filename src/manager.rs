use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Manager, Peripheral};
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x000102030405060708090a0b0c0d2b11); // characteristic for the lights

struct Light {
    light: Peripheral,
    characteristic: Characteristic,
}

/// Manager for all light operations.
pub struct LightManager {
    lights: Vec<Light>,
}

impl LightManager {
    /// Returns a new LightManager.
    pub fn new() -> LightManager {
        LightManager { lights: vec![] }
    }

    /// Establishes a connection to `num_lights` lights with names beginning in `prefix`.
    /// This function scans for available lights for `wait_millis` milliseconds.
    ///
    /// # Panics
    /// Panics if `num_lights` lights were not found after `wait_millis` milliseconds have passed.
    pub async fn connect_to_lights(
        &mut self,
        prefix: &str,
        num_lights: usize,
        wait_millis: u64,
    ) -> Result<(), Box<dyn Error>> {
        // obtain manager
        let manager = Manager::new().await.unwrap();

        // get adapter
        let central = manager
            .adapters()
            .await
            .unwrap()
            .into_iter()
            .nth(0)
            .unwrap();

        // store all lights
        let mut lights: Vec<Peripheral> = vec![];

        // start scanning for available devices
        central.start_scan(ScanFilter::default()).await?;

        // wait for specified time
        time::sleep(Duration::from_millis(wait_millis)).await;

        // check if lights were found
        for peripheral in central.peripherals().await.unwrap() {
            match peripheral.properties().await.unwrap().unwrap().local_name {
                Some(name) => {
                    // add light to lights if prefix matches
                    if name.starts_with(prefix) {
                        lights.push(peripheral);
                    }
                }
                None => (),
            }
        }

        // stop execution if not all lights were found
        if lights.len() < num_lights {
            panic!("some lights were not found");
        }

        // find characteristic for each light
        for _ in 0..lights.len() {
            let light = lights.swap_remove(0);

            // connect to the light
            light.connect().await?;

            // locate characteristic
            light.discover_services().await?;
            let characteristics = light.characteristics();

            // locate command characteristic
            let command_characteristic = characteristics
                .iter()
                .find(|c| c.uuid == CHARACTERISTIC_UUID)
                .expect("command characteristic was not located");

            self.lights.push(Light {
                light: light,
                characteristic: Characteristic {
                    uuid: command_characteristic.uuid,
                    service_uuid: command_characteristic.service_uuid,
                    properties: command_characteristic.properties,
                },
            });
        }

        Ok(())
    }

    async fn send_command(&self, frame: Vec<u8>) -> Result<(), Box<dyn Error>> {
        // send the specified command to each light
        for i in 0..self.lights.len() {
            let light = &self.lights[i];
            light
                .light
                .write(&light.characteristic, &frame, WriteType::WithoutResponse)
                .await?;
        }

        Ok(())
    }

    /// Turns on/off the lights depending on the value of `power`.
    pub async fn set_power(&self, power: bool) -> Result<(), Box<dyn Error>> {
        let mut command = vec![0x33, 0x01, power as u8];
        create_frame(&mut command);
        self.send_command(command).await?;

        Ok(())
    }

    /// Sets the brightness of the lights to `brightness`.
    pub async fn set_brightness(&self, brightness: u8) -> Result<(), Box<dyn Error>> {
        let mut command = vec![0x33, 0x04, brightness];
        create_frame(&mut command);
        self.send_command(command).await?;

        Ok(())
    }

    /// Sets the color of the lights to the RGB value given by `r`, `g`, and `b`.
    pub async fn set_color(&self, r: u8, g: u8, b: u8) -> Result<(), Box<dyn Error>> {
        let mut command = vec![0x33, 0x05, 0x02, r, g, b];
        create_frame(&mut command);
        self.send_command(command).await?;

        Ok(())
    }
}

fn create_frame(bytes: &mut Vec<u8>) {
    // each frame has a length of 20
    bytes.resize(19, 0);

    // calculate checksum by xoring all values
    let checksum = bytes.iter().fold(0, |a, x| a ^ x);
    bytes.push(checksum);
}
