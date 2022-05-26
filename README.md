# AverageLights

A program for _Govee H6001 Smart LED Light Bulbs_.

The program will take frequent screen captures and set the RGB value of nearby lights to the average pixel value of the screen.

## Installation

To begin, first ensure you have a working installation of [Rust and Cargo](https://www.rust-lang.org/learn/get-started).

1. Clone this repository onto your local machine.
2. Edit the `config.toml` file and set your preferred values. The default prefix should work fine for _Govee H6001 Smart LED Light Bulbs_, but you may want to change the number of lights and wait time(s) to suit your needs.
3. Open a terminal at `src` and run `cargo build --release`. This will create an executable of the program in the `target/release` directory which you can run on any machine.
4. Open `target/release` and execute the `averagelights.exe` file.

Once the program is started, you can stop execution at any time with `Ctrl + C` or exiting the terminal. The program will take a screen capture every `capture_wait_millis` (value set in `config.toml`) milliseconds and change the RGB value of all lights to the average pixel value of the screen capture.
