mod bindings {
  #![allow(dead_code)]
  #![allow(non_snake_case)]
  #![allow(non_camel_case_types)]
  #![allow(non_upper_case_globals)]
  include!("bindings.rs");
}

extern crate image;

use std::ffi::CString;

use bindings::NSVGimage;

const BYTES_PER_PIXEL: usize = 4;

pub fn parse_file(filename: &str, units: &str, dpi: f32) -> *mut NSVGimage {
  use bindings::nsvgParseFromFile;

  let filename_chars = CString::new(filename).unwrap();
  let unit_chars = CString::new(units).unwrap();

  unsafe {
    nsvgParseFromFile(filename_chars.as_ptr(), unit_chars.as_ptr(), dpi)
  }
}

pub fn rasterize(image: *mut NSVGimage, scale: f32) -> image::RgbaImage {
  use bindings::nsvgCreateRasterizer;
  use bindings::nsvgRasterize;

  let width = unsafe { (*image).width * scale } as usize;
  let height = unsafe { (*image).height * scale } as usize;
  let capacity = BYTES_PER_PIXEL * width * height;
  let mut dst = Vec::with_capacity(capacity);
  let stride = width * BYTES_PER_PIXEL;

  unsafe {
    // Not sure if we care about reusing this or not...
    let r = nsvgCreateRasterizer();

    nsvgRasterize(      // Rasterizes SVG image, returns RGBA image (non-premultiplied alpha)
      r,                //   r - pointer to rasterizer context
      image,            //   image - pointer to image to rasterize
      0.0, 0.0,         //   tx,ty - image offset (applied after scaling)
      scale,            //   scale - image scale
      dst.as_mut_ptr(), //   dst - pointer to destination image data, 4 bytes per pixel (RGBA)
      width as i32,     //   w - width of the image to render
      height as i32,    //   h - height of the image to render
      stride as i32     //   stride - number of bytes per scaleline in the destination buffer
    );

    // Need to manually set the length of the vector to match the data that's been put in it
    dst.set_len(capacity);
  }

  image::RgbaImage::from_raw(width as u32, height as u32, dst).unwrap()
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_parse_file() {
    let svg = parse_file("examples/spiral.svg", "px", 72.0);

    unsafe {
      assert_eq!((*svg).width, 256.0);
      assert_eq!((*svg).height, 256.0);
    }
  }

  #[test]
  fn can_rasterize() {
    let svg = parse_file("examples/spiral.svg", "px", 72.0);
    let image = rasterize(svg, 1.0);

    assert_eq!(image.dimensions(), (256, 256));
  }

  #[test]
  fn can_rasterize_and_scale() {
    let svg = parse_file("examples/spiral.svg", "px", 72.0);
    let image = rasterize(svg, 2.0);

    assert_eq!(image.dimensions(), (512, 512));
  }
}

