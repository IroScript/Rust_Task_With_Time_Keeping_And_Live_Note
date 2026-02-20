# Distributing "Daily Motivation"

Because this project utilizes a custom 3D background rendering engine built with Bevy, the application is actually composed of **two separate executables** that communicate with each other natively.

## How to Package the Application for End-Users

To distribute the app, you only need to give the user a single folder containing both executables:

1. Create a new empty folder called `Daily Motivation App`
2. Compile both applications in release mode:
   - Run `cargo build --release` in the main folder (`rustV10`).
   - Run `cargo build --release` inside the `background` folder.
3. Copy **`target/release/daily-motivation.exe`** into your new folder.
4. Copy **`background/target/release/quantum_logo.exe`** into the *exact same folder*.

**That's it!** 
When the user runs `daily-motivation.exe`, it will automatically detect `quantum_logo.exe` sitting right next to it and launch the 3D background flawlessly. You can ZIP this folder and send it to anyone!
