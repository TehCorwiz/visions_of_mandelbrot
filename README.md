# Visions of Mandelbrot

This project represents my first steps into WASM. I've been tinkering in Rust for a while, and I wanted a more complex
goal. This project's goals are to implement a reasonably efficient Mandelbrot Set renderer with interactive controls
that targets both the desktop and web. Rendering to the screen and input are handled by
the [Pixels](https://github.com/parasyte/pixels) library. Since I've used this library before I used
the [minimal-web](https://github.com/parasyte/pixels/tree/main/examples/minimal-web) example as a starting point to
avoid writing boilerplate. I'm using the Wikipedia
article [Plotting algorithms for the Mandelbrot set](https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set)
as my main reference for implementation hints and optimization strategies.

## TODO:
- [X] Basic bulb
- [X] Color
- [X] Zoom
- [X] Retargeting
- [ ] Split render from screen drawing
- [ ] UI
- [ ] Preset or custom palettes

## Dev env setup

1. Install Rust + Cargo.
2. Clone the repo.
3. Install dependencies: `cargo install --locked wasm-bindgen-cli just miniserve`

## Running

### For Desktop

`cargo run --package visions_of_mandelbrot --bin visions_of_mandelbrot`

### For Web

1. `just serve visions_of_mandelbrot`
2. Visit `http://localhost:8080/` in a web browser.
