//! Miller cylindrical projection onto the texture.

use std::f64::consts::{FRAC_PI_4, PI, TAU};

/// Miller ordinate (in radians of the unit sphere) for a latitude in degrees.
fn miller_y(lat_deg: f64) -> f64 {
    1.25 * (FRAC_PI_4 + 0.4 * lat_deg.to_radians()).tan().ln()
}

/// Latitude in degrees for a Miller ordinate.
fn miller_lat(y: f64) -> f64 {
    (2.5 * (0.8 * y).exp().atan() - 0.625 * PI).to_degrees()
}

/// Texture frame: longitude -180..180 across `width`, latitude `lat_top` down to `lat_bottom`.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// Pixels per radian of longitude.
    scale: f64,
    y_top: f64,
}

impl Frame {
    /// Builds the frame for `width`; the height is rounded to whole pixels, so the true bottom
    /// latitude is [`Frame::lat_bottom`], within half a pixel of `lat_bottom_wanted`.
    pub fn new(width: u32, lat_top: f64, lat_bottom_wanted: f64) -> Self {
        let scale = f64::from(width) / TAU;
        let y_top = miller_y(lat_top);
        let height = ((y_top - miller_y(lat_bottom_wanted)) * scale).round() as u32;
        Self {
            width,
            height,
            scale,
            y_top,
        }
    }

    pub fn lat_top(&self) -> f64 {
        miller_lat(self.y_top)
    }

    pub fn lat_bottom(&self) -> f64 {
        miller_lat(self.y_top - f64::from(self.height) / self.scale)
    }

    /// Texture coordinates (pixels, origin at the top-left corner) of a lon/lat point.
    pub fn to_pixel(self, lon: f64, lat: f64) -> (f64, f64) {
        let x = (lon.to_radians() + PI) * self.scale;
        let y = (self.y_top - miller_y(lat)) * self.scale;
        (x, y)
    }

    /// Lon/lat of a texture point.
    #[cfg(test)]
    pub fn to_lon_lat(self, x: f64, y: f64) -> (f64, f64) {
        let lon = (x / self.scale - PI).to_degrees();
        let lat = miller_lat(self.y_top - y / self.scale);
        (lon, lat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let frame = Frame::new(8192, 84.0, -58.0);
        for &(lon, lat) in &[
            (0.0, 0.0),
            (-180.0, 84.0),
            (179.9, -57.9),
            (-47.9, -15.8),
            (139.7, 35.7),
        ] {
            let (x, y) = frame.to_pixel(lon, lat);
            let (lon2, lat2) = frame.to_lon_lat(x, y);
            assert!((lon - lon2).abs() < 1e-9, "{lon} {lon2}");
            assert!((lat - lat2).abs() < 1e-9, "{lat} {lat2}");
        }
    }

    #[test]
    fn frame_edges() {
        let frame = Frame::new(8192, 84.0, -58.0);
        assert_eq!(frame.height, 4103);
        assert!((frame.lat_top() - 84.0).abs() < 1e-9);
        assert!((frame.lat_bottom() + 58.0).abs() < 0.05);
        let (x, y) = frame.to_pixel(-180.0, 84.0);
        assert!(x.abs() < 1e-9 && y.abs() < 1e-9);
        let (x, y) = frame.to_pixel(180.0, frame.lat_bottom());
        assert!((x - 8192.0).abs() < 1e-9 && (y - 4103.0).abs() < 1e-6);
        // Equator sits below the middle: the frame reaches further north than south.
        let (_, y_eq) = frame.to_pixel(0.0, 0.0);
        assert!(y_eq > 2052.0);
    }
}
