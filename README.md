# nsvg [![Build Status](https://travis-ci.org/nickbrowne/nsvg.svg?branch=master)](https://travis-ci.org/nickbrowne/nsvg) [![Build status](https://ci.appveyor.com/api/projects/status/nr4jsaibmh5i3fxw/branch/master?svg=true)](https://ci.appveyor.com/project/nickbrowne/nsvg/branch/master) [![Crates.io](https://img.shields.io/crates/v/nsvg.svg)](https://crates.io/crates/nsvg)

A friendly Rust wrapper around the excellent NanoSVG C library.

Offers a simple way to parse and rasterize SVG, at whichever scale you want. It was written as a convenient way to have scaled UI elements in video games that are suited to the user's selected resolution, but can also be used for simple file conversions.

NanoSVG supports a wide range of SVG features, with most of the vector elements fully supported.

The rasterizer runs entirely on the CPU and has no external dependencies. The quality will be fairly equivalent to exporting a bitmap from Inkscape. The raserizer is based on the one used by `stb_truetype`, all rasters will be anti-aliased. You can read more about the `stb_truetype` rasterizer [here](https://nothings.org/gamedev/rasterize/).

There are faster GPU based solutions to rendering vector graphics out there, but the simplicity of NanoSVG and it's lack of dependencies is a huge benefit, and should run just fine cross platform.

## Usage

Include `nsvg` in your `Cargo.toml` dependencies.

```toml
[dependencies]
nsvg = "0.5.0"
```

To include `nsvg` without the `image` dependency, add it to your `Cargo.toml` without default dependencies.

```toml
[dependencies.nsvg]
version = "0.5.0"
default-features = false
```

Now you can parse and rasterize SVGs. Use the scale argument to produce larger or smaller rasterised images. The aspect ratio will always remain the same.


```rust
extern crate nsvg;

use std::path::Path;

fn main() {
  let path = Path::new("examples/example.svg");

  // Load and parse the svg
  let svg = nsvg::parse_file(path, nsvg::Units::Pixel, 96.0).unwrap();

  // Create a scaled raster
  let scale = 2.0;
  let image = svg.rasterize(scale);
}

```

## Unsupported SVG elements

As it is mostly intended to be used for parsing and rasterising vector graphics, some SVG features are not supported by nsvg:

 - Text elements are ignored, although text can simply be converted to a path and it will work just fine

 - Embedded bitmap images are ignored

 - Scripts are ignored

 - Animations are ignored

If you encounter anything that does not rasterize as you would expect, try converting it to a path first.

## Developing

By default nsvg will use prebuilt FFI bindings, but you can regenerate them manually by running:

```
cargo build --features bindgen
```

Which will also copy the bindings into the `src` directory. If the bindings need to be updated (when upgrading NanoSVG, for example) they should be checked in to version control.

Make sure you have `libclang-dev` available so bindgen works.

`cargo test` to run tests.

There is one provided example, which you can run with the following command:
```
cargo run --example svg_to_png
```

This will convert the `example.svg` vector into a PNG. The PNG will be written to the working directory as `example_output.png`.

## See also

https://github.com/memononen/nanosvg

https://github.com/rezrov/libnanosvg

## License

NanoSVG is licensed under [zlib license](lib/LICENSE.txt)

Sutte Hakkun logo is drawn by [Nico Vliek](https://www.behance.net/gallery/63535513/300-Super-Nintendo-Logos-Fully-Remastered), and is a registered trademark of Nintendo. Used for demonstration purposes only.

Mr Disk is drawn by [Eoin Stanley](http://www.eoinstanley.com/nintendo/index.htm), and is a registered trademark of Nintendo. Used for demonstration purposes only.

Anything else is MIT
