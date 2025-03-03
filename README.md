### Archival notice
This rewrite uses a library that has halted development, and is also somewhat limiting. I've also been busy with life and university at this time and I completely stopped working on this rewrite.

This repository will be archived for preservation. I will still remake the game whenever I can, but it won't be using Tetra.

# upfall-rs
This is a rewrite of my game jam game [Upfall](https://github.com/Raoul1808/Upfall) in Rust using Tetra.

## Building
Pre-requisites:
- Rust (preferably installed with rustup)
- SDL2 installed on your machine

Building steps:
1. Clone this repo
2. Run `cargo build --release`
3. Grab the binary in `target/release` and the `res` directory and place them somewhere if you want
4. WINDOWS-ONLY: place SDL2.dll right next to the game executable
5. Profit

You can also run the game directly in this repo's root directory by running `cargo run --release` instead. You will still need to place SDL2.dll on Windows though.

## License

This project is licensed under the [MIT License](LICENSE)
