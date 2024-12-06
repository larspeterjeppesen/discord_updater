# discord_updater
This program is meant for debian users who do not wish to manually update discord every other day.


# Usage
In main.rs, update the variable discord_path to whereever you want to keep the discord installation.
Then, compile using `cargo build --release`.
Running the program will install discord if it is not installed, and update it manually if it is required.
