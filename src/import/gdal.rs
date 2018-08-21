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

use gdal::errors::Result;
use gdal::raster::{Dataset, RasterBand};
use gdal::spatial_ref::SpatialRef;
use std::f64::EPSILON;
use std::path::Path;

use math::{transform_coords, AffineTransform};
use textures::Texture;

pub struct GdalRasterImporter;

impl GdalRasterImporter {
    /// Import an entire raster file
    pub fn import(
        path: &Path,
        band: usize,
    ) -> Result<(String, AffineTransform, Texture<f64>)> {
        let dataset = try!(Dataset::open(path));
        let raster = try!(dataset.rasterband(band as isize));
        let transform = try!(dataset.geo_transform());
        let (width, height) = dataset.size();

        let data = try!(Self::read_raster(&raster, 0, 0, width, height));

        assert!((transform[2] - 0.0).abs() < EPSILON);
        assert!((transform[4] - 0.0).abs() < EPSILON);

        let x_origin = transform[0];
        let y_origin = transform[3] * -1.0;
        let pixel_width = transform[1];
        let pixel_height = transform[5] * -1.0;
        let transform =
            AffineTransform::new(x_origin, y_origin, pixel_width, pixel_height);

        let spat_ref = try!(SpatialRef::from_wkt(&dataset.projection()));
        let proj4 = try!(spat_ref.to_proj4());

        Ok((proj4, transform, Texture::new(width, height, data)))
    }

    /// Import a bounding box of a raster file
    pub fn import_bbox(
        path: &Path,
        band: usize,
        nw: (f64, f64),
        se: (f64, f64),
        inp_proj4: &str,
    ) -> Result<(String, AffineTransform, Texture<f64>)> {
        let dataset = try!(Dataset::open(path));
        let raster = try!(dataset.rasterband(band as isize));
        let transform = try!(dataset.geo_transform());
        let spat_ref = try!(SpatialRef::from_wkt(&dataset.projection()));
        let proj4 = try!(spat_ref.to_proj4());

        let nw = transform_coords(nw.0, nw.1, inp_proj4, &proj4);
        let se = transform_coords(se.0, se.1, inp_proj4, &proj4);

        let (x1, y1) = Self::world_to_raster(nw.0, nw.1, &transform);
        let (x2, y2) = Self::world_to_raster(se.0, se.1, &transform);
        let width = (x2 - x1) as usize;
        let height = (y2 - y1) as usize;

        let data = try!(Self::read_raster(&raster, x1, y1, width, height));

        assert!((transform[2] - 0.0).abs() < EPSILON);
        assert!((transform[4] - 0.0).abs() < EPSILON);

        let pixel_width = transform[1];
        let pixel_height = transform[5] * -1.0;
        let x_origin = transform[0] + (x1 as f64 * transform[1]);
        let y_origin = (transform[3] + (y1 as f64 * transform[5])) * -1.0;
        let transform =
            AffineTransform::new(x_origin, y_origin, pixel_width, pixel_height);

        Ok((proj4, transform, Texture::new(width, height, data)))
    }

    fn world_to_raster(x: f64, y: f64, transform: &[f64; 6]) -> (isize, isize) {
        assert!((transform[2] - 0.0).abs() < EPSILON);
        assert!((transform[4] - 0.0).abs() < EPSILON);
        let x_origin = transform[0];
        let y_origin = transform[3];
        let pixel_width = transform[1];
        let pixel_height = -transform[5];
        let col = (x - x_origin) / pixel_width;
        let row = (y_origin - y) / pixel_height;
        (col.floor() as isize, row.floor() as isize)
    }

    fn read_raster(
        raster: &RasterBand,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
    ) -> Result<Vec<f64>> {
        let window = (width, height);
        let nodata = match raster.no_data_value() {
            Some(val) => val,
            None => 0.0,
        };

        let data = try!(raster.read_as::<f64>((x, y), window, window))
            .data
            .iter()
            .map(|e| {
                if (*e - nodata).abs() < EPSILON {
                    0.0
                } else {
                    *e
                }
            })
            .collect();

        Ok(data)
    }
}
