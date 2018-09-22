// This file is part of Peaks.
//
// Peaks is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Peaks is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Peaks. If not, see <https://www.gnu.org/licenses/>.

extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate peaks;
extern crate png;
extern crate serde_json;

use docopt::Docopt;
use peaks::io::png::export;
use peaks::ops::linear_to_srgb;
use peaks::{render_threaded, Renderer, Scene, Texture};

use std::fs::File;
use std::io::{stdin, Read, Result};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "
Peaks.

Usage:
    peaks [options] <input> <output>
    peaks [options] <output>
    peaks (-h | --help)
    peaks --version

Options:
    -h, --help              Show this screen.
    --version               Show version.
    --samples=<number>      Number of multi-samples [default: 4].
    --threads=<number>      Number of render threads [default: 4].
    --tile-size=<pixels>    Size of a render tile [default: 8].
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_samples: usize,
    flag_threads: usize,
    flag_tile_size: usize,
    flag_version: bool,
    arg_input: String,
    arg_output: String,
}

fn slurp(file_path: &str) -> Result<String> {
    let mut txt = String::new();
    if file_path.is_empty() {
        try!(stdin().read_to_string(&mut txt));
    } else {
        let mut fp = try!(File::open(file_path));
        try!(fp.read_to_string(&mut txt));
    }
    Ok(txt)
}

fn main() -> Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("v{}", VERSION);
        return Ok(());
    }

    let deff = serde_json::from_str(&slurp(&args.arg_input)?)?;
    let scene = Scene::new(deff);
    let (width, height) = scene.camera.view_plane();
    let renderer = Renderer::new(args.flag_samples, scene);

    let mut surface = Texture::blank(width, height);
    let mut output = Texture::blank(width, height);

    render_threaded(
        &mut surface,
        &renderer,
        args.flag_threads,
        args.flag_tile_size,
    );
    linear_to_srgb(&surface, &mut output);
    export(args.arg_output, &output)
}
