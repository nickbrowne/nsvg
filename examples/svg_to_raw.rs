extern crate nsvg;

fn main() {
  // Load the SVG data
  let svg = nsvg::parse_str(include_str!("example.svg"), nsvg::Units::Pixel, 96.0).unwrap();

  // Rasterize the loaded SVG and return dimensions and a RGBA buffer
  let (width, height, raw_rgba) = svg.rasterize_to_raw_rgba(2.0).unwrap();

  println!("Rasterized to a RGBA buffer of size {}", raw_rgba.len());
  println!("Image width is {}, and height is {}", width, height);
}
