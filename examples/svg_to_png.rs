extern crate nsvg;
extern crate image;

use std::env;
use std::path::PathBuf;

fn main() {
  // Load the SVG data
  let svg = nsvg::parse_file("examples/spiral.svg", "px", 72.0);

  // Rasterize the loaded SVG and return an ImageBuffer
  let image_buffer = nsvg::rasterize(svg);

  let save_path = env::current_dir().unwrap().join("spiral_output.png");
  let (width, height) = image_buffer.dimensions();

  // Write the image to disk as a PNG
  image::save_buffer(
    save_path,
    &image_buffer.into_raw(),
    width,
    height,
    image::ColorType::RGBA(8),
  ).expect("Failed to save png.");
}
