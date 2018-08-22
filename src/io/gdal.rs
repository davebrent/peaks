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

use std::convert::AsRef;
use std::f64::EPSILON;
use std::path::Path;

use gdal::errors::Result;
use gdal::raster::{Dataset, RasterBand};
use gdal::spatial_ref::SpatialRef;

use math::{transform_coords, AffineTransform};
use textures::Texture;

/// Import a region specified in pixel coordinates from a set of raster bands
pub fn import_rect<P, D>(
    path: P,
    bands: &[usize],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> Result<(String, AffineTransform, Vec<Texture<D>>)>
where
    P: AsRef<Path>,
    D: Copy + Clone + Default + PartialEq + GdalRasterType<D>,
{
    let dataset = try!(Dataset::open(path.as_ref()));
    let transform = try!(dataset.geo_transform());
    let spat_ref = try!(SpatialRef::from_wkt(&dataset.projection()));
    let proj4 = try!(spat_ref.to_proj4());

    let (x, y) = (x as isize, y as isize);
    let mut rasters = Vec::with_capacity(bands.len());
    for band in bands {
        let raster = try!(dataset.rasterband(*band as isize));
        let data = try!(D::read_raster(&raster, x, y, width, height));
        rasters.push(Texture::new(width, height, data));
    }

    assert!((transform[2] - 0.0).abs() < EPSILON);
    assert!((transform[4] - 0.0).abs() < EPSILON);

    let pw = transform[1];
    let ph = transform[5] * -1.0;
    let xo = transform[0] + (x as f64 * transform[1]);
    let yo = (transform[3] + (y as f64 * transform[5])) * -1.0;

    Ok((proj4, AffineTransform::new(xo, yo, pw, ph), rasters))
}

/// Import a region specified in spatial coordinates from a set of raster bands
pub fn import_spatial<P, D>(
    path: P,
    bands: &[usize],
    nw: (f64, f64),
    se: (f64, f64),
    inp_proj4: &str,
) -> Result<(String, AffineTransform, Vec<Texture<D>>)>
where
    P: AsRef<Path>,
    D: Copy + Clone + Default + PartialEq + GdalRasterType<D>,
{
    let dataset = try!(Dataset::open(path.as_ref()));
    let transform = try!(dataset.geo_transform());
    let spat_ref = try!(SpatialRef::from_wkt(&dataset.projection()));
    let proj4 = try!(spat_ref.to_proj4());

    let nw = transform_coords(nw.0, nw.1, inp_proj4, &proj4);
    let se = transform_coords(se.0, se.1, inp_proj4, &proj4);

    let transform = AffineTransform::new(
        transform[0],
        transform[3],
        transform[1],
        transform[5],
    );

    let (x1, y1) = transform.inverse(nw.0, nw.1);
    let (x2, y2) = transform.inverse(se.0, se.1);
    let width = (x2 - x1) as usize;
    let height = (y2 - y1) as usize;

    import_rect(path, bands, x1 as usize, y1 as usize, width, height)
}

/// Import all specified raster bands
pub fn import<P, D>(
    path: P,
    bands: &[usize],
) -> Result<(String, AffineTransform, Vec<Texture<D>>)>
where
    P: AsRef<Path>,
    D: Copy + Clone + Default + PartialEq + GdalRasterType<D>,
{
    let dataset = try!(Dataset::open(path.as_ref()));
    let (width, height) = dataset.size();
    import_rect(path, bands, 0, 0, width, height)
}

// XXX: See https://github.com/georust/gdal/issues/48
pub trait GdalRasterType<T>
where
    T: Copy + Default + PartialEq,
{
    fn read_raster(
        raster: &RasterBand,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
    ) -> Result<Vec<T>>;
}

impl GdalRasterType<u8> for u8 {
    fn read_raster(
        raster: &RasterBand,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
    ) -> Result<Vec<u8>> {
        let window = (width, height);
        Ok(try!(raster.read_as::<u8>((x, y), window, window)).data)
    }
}

impl GdalRasterType<f64> for f64 {
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
            None => Default::default(),
        };

        Ok(try!(raster.read_as::<f64>((x, y), window, window))
            .data
            .iter()
            .map(|d| {
                if (*d - nodata).abs() < EPSILON {
                    Default::default()
                } else {
                    *d
                }
            })
            .collect())
    }
}
