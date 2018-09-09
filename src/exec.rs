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
use render::Renderer;
use samplers::Sampler;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use textures::Texture;

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

pub fn render<C, S>(
    width: usize,
    height: usize,
    renderer: &Renderer<C, S>,
    image: &mut Texture<Vec3>,
) -> bool
where
    C: 'static + Camera,
    S: 'static + Sampler,
{
    let mut progress = ProgressCounter::new(30, width * height);
    let mut completed = 0;

    for y in 0..height {
        for x in 0..width {
            let color = renderer.pixel(x, y);
            image.write1x1(x, y, color);
            completed += 1;
            progress.update(completed);
        }
    }

    progress.finish();
    true
}
