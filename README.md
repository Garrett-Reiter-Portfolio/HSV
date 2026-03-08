# HSV

Garrett Reiter 2026

read hue, saturation and values from a potentiometer, convert these into RBG 
color ratios, and output with a RGB LED. Potentiometer and RGB LED will be 
connected to the controlling micro:bit through a breadboard.

[Watch the video](https://github.com/Garrett-Reiter-Portfolio/HSV/raw/refs/heads/main/hsvThing.mp4)

## Build and Run

Instructions are provided in the embedded [micro:bit Discovery](https://docs.rust-embedded.org/discovery-mb2/index.html) for setting up a build environment for the micro:bit.

from the cloned repo on the controlling computer run:

`cargo embed --release`

to flash and run

Another command is:

`cargo run --release`

## License

This work is made available under the MIT License

## Acknowledgements

Thanks to Bart Massey and Claude AI for the function that convert HSV to RGB

