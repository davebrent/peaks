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

use cameras::Camera;
use math::Vec3;
use ops::{blit, blit_region};
use render::Renderer;
use samplers::Sampler;
use textures::{Texture, TileIterator};

use std::io::{self, Write};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct ProgressCounter {
    out: io::Stdout,
    total: usize,
    count: usize,
    percent: usize,
    width: usize,
    start: Instant,
    elapsed: Duration,
}

impl ProgressCounter {
    pub fn new(width: usize, total: usize) -> ProgressCounter {
        ProgressCounter {
            width,
            out: io::stdout(),
            total,
            count: 0,
            percent: 0,
            start: Instant::now(),
            elapsed: Duration::new(0, 0),
        }
    }

    fn format_duration(&self, dur: Duration) -> String {
        let secs = dur.as_secs();
        let ms = f64::from(dur.subsec_nanos()) / 1_000_000.0;

        if secs > 3600 {
            let mins = secs / 60;
            let secs = secs % 60;
            let hrs = mins / 60;
            let mins = mins % 60;
            format!("{}hrs {}mins {:2}secs {:5.2}ms", hrs, mins, secs, ms)
        } else if secs > 60 {
            let mins = secs / 60;
            let secs = secs % 60;
            format!("{}mins {:2}secs {:5.2}ms", mins, secs, ms)
        } else if secs > 0 {
            format!("{:2}secs {:5.2}ms", secs, ms)
        } else {
            format!("{:.2}ms", ms)
        }
    }

    fn print_bar(&mut self) {
        let dots =
            (self.percent as f64 / 100.0 * self.width as f64).floor() as usize;
        print!("\r");
        for _ in 0..dots {
            print!("#");
        }
        for _ in 0..self.width - dots {
            print!(".");
        }
        print!(
            " {:3}% Elapsed {}",
            self.percent,
            self.format_duration(self.elapsed)
        );
    }

    pub fn update(&mut self, count: usize) {
        self.count = count;
        self.elapsed = self.start.elapsed();
        let percent =
            (self.count as f64 / self.total as f64 * 100.0).floor() as usize;
        if percent != self.percent {
            self.percent = percent;
            self.print_bar();
            self.out.flush().unwrap();
        }
    }

    pub fn finish(&mut self) {
        print!("\r");
        let elapsed = self.start.elapsed();
        print!("\r");
        for _ in 0..80 {
            print!(" ");
        }
        print!("\r");
        println!("Completed in {}", self.format_duration(elapsed));
        self.out.flush().unwrap();
    }
}

pub fn render<C, S>(image: &mut Texture<Vec3>, renderer: &Renderer<C, S>)
where
    C: Camera,
    S: Sampler,
{
    let mut progress = ProgressCounter::new(30, image.width * image.height);
    let mut completed = 0;

    for y in 0..image.height {
        for x in 0..image.width {
            let color = renderer.pixel(x, y);
            image.write1x1(x, y, color);
            completed += 1;
            progress.update(completed);
        }
    }

    progress.finish();
}

struct RenderState {
    surface: Texture<Vec3>,
    tiles: TileIterator,
}

fn worker<C, S>(
    state: &Arc<Mutex<RenderState>>,
    renderer: &Renderer<C, S>,
    sender: &Sender<usize>,
    tile_size: usize,
) where
    C: 'static + Camera + Clone,
    S: 'static + Sampler + Clone,
{
    let mut local = Texture::blank(tile_size, tile_size);
    let mut work = { state.lock().unwrap().tiles.next() };

    while let Some(tile) = work {
        for y in 0..tile.height {
            for x in 0..tile.width {
                let pixel = renderer.pixel(tile.x + x, tile.y + y);
                local.write1x1(x, y, pixel);
            }
        }
        {
            let mut state_ = state.lock().unwrap();
            blit_region(
                &local,
                &mut state_.surface,
                tile.x,
                tile.y,
                tile.width,
                tile.height,
            );
            work = state_.tiles.next();
        }
        sender.send(tile.width * tile.height).unwrap();
    }
}

pub fn render_threaded<C, S>(
    output: &mut Texture<Vec3>,
    renderer: &Renderer<C, S>,
    num_workers: usize,
    tile_size: usize,
) where
    C: 'static + Camera + Clone,
    S: 'static + Sampler + Clone,
{
    let width = output.width;
    let height = output.height;
    let total = width * height;

    let state = Arc::new(Mutex::new(RenderState {
        surface: Texture::blank(width, height),
        tiles: output.tiles(tile_size),
    }));

    let (sender, receiver) = channel();
    let mut workers = Vec::with_capacity(num_workers);
    for _ in 0..num_workers {
        let state_ = state.clone();
        let renderer_ = renderer.clone();
        let sender_ = sender.clone();
        workers.push(thread::spawn(move || {
            worker(&state_, &renderer_, &sender_, tile_size);
        }));
    }

    let mut progress = ProgressCounter::new(30, total);
    let mut completed = 0;
    while let Ok(rendered) = receiver.recv() {
        completed += rendered;
        progress.update(completed);
        if completed == total {
            break;
        }
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let state = state.lock().unwrap();
    blit(&state.surface, output, 0, 0);
    progress.finish();
}
