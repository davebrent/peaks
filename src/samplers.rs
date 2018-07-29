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

pub struct RegularGridSampler {
    samples: Vec<(f64, f64)>,
}

impl RegularGridSampler {
    pub fn new(num: usize) -> RegularGridSampler {
        let mut samples = Vec::with_capacity(num);

        let size = (num as f64).sqrt() as usize;
        let sample_width = 1.0 / size as f64;
        let offset = sample_width / 2.0;

        for y in 0..size {
            let v = offset + (y as f64 * sample_width);
            for x in 0..size {
                let u = offset + (x as f64 * sample_width);
                samples.push((u, v));
            }
        }

        RegularGridSampler { samples }
    }

    pub fn amount(&self) -> usize {
        self.samples.len()
    }

    pub fn samples(&self) -> impl Iterator<Item = &(f64, f64)> {
        self.samples.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_grid_sampler_one() {
        let sampler = RegularGridSampler::new(1);
        assert_eq!(sampler.samples, vec![(0.5, 0.5)]);
    }

    #[test]
    fn regular_grid_sampler() {
        let sampler = RegularGridSampler::new(4);
        assert_eq!(
            sampler.samples,
            vec![(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
        );
    }
}
