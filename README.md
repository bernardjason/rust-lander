# Rust Lander

A 3d rust game using OpenGl and Emscripten to build for the wasm32-unknown-emscripten.
It can also run standalone, developed and tested on Linux but will
work on Windows, see some of the other Rust projects in this repo.

![screenshot](screenshot.png)

YouTube demo
[![youtube video](https://img.youtube.com/vi/Kmm_hlNORZU/0.jpg)](https://youtu.be/Kmm_hlNORZU)


I hosted the WASM version here https://bjason.org/en/wasm/rust-lander/

to run standalone
```
cargo build
cargo run
```

For web deployment
```
cargo build --target=wasm32-unknown-emscripten 
```

To try web version locally having built to emscripten target try script
```
./run_wasm.sh
```
