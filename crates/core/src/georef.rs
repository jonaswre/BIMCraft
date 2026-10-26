//! Where the project stands on the earth, as Revit's Location and its shared coordinates say: the map (a projected
//! coordinate reference system, an EPSG code such as `EPSG:25832` for ETRS89 / UTM zone 32N), where the model's origin
//! lies on it and how far its +y is turned from the map's grid north, true north, and the site's latitude, longitude and
//! elevation. The model keeps its own coordinates; this is what IFC4 writes as an `IfcMapConversion` to an
//! `IfcProjectedCRS`, the context's `TrueNorth`, and the site's `RefLatitude`, `RefLongitude` and `RefElevation`
//! (ISO 16739), and what a scan in map coordinates is placed by (`point_cloud` with `origin: map`).

use serde::{Deserialize, Serialize};

use crate::geom::P2;
use crate::model::{MAX_COORD, MAX_TEXT};

/// How far a map coordinate may be from the map's origin, mm: past any projection's eastings and northings.
pub const MAX_MAP: f64 = 1e11;

/// Where the project stands on the earth. Lengths in mm, as everywhere; angles in degrees.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Georeference {
    /// The map: its coordinate reference system's name, an EPSG code (`EPSG:25832`) or another name the map goes by.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub crs: String,
    /// The map's geodetic datum (`ETRS89`), vertical datum (`DHHN2016`), projection (`UTM`) and zone (`32N`), as far as
    /// its name doesn't say them.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub geodetic_datum: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub vertical_datum: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub map_projection: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub map_zone: String,
    /// Where the model's origin lies on the map: its easting and northing, and the height of the model's 0 over the
    /// vertical datum.
    #[serde(default)]
    pub eastings: f64,
    #[serde(default)]
    pub northings: f64,
    #[serde(default)]
    pub orthogonal_height: f64,
    /// How far the model's +y (its project north) is turned clockwise from the map's grid north, degrees.
    #[serde(default)]
    pub rotation: f64,
    /// How far the model's +y is turned clockwise from true north, degrees; none, grid north stands for it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub true_north: Option<f64>,
    /// The site's latitude and longitude, decimal degrees (north and east positive), and its elevation, mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
}

/// An angle in degrees brought into (-180, 180].
pub fn turn(deg: f64) -> f64 {
    let a = deg.rem_euclid(360.0);
    if a > 180.0 { a - 360.0 } else { a }
}

impl Georeference {
    /// Whether it places the model on a map: it names one.
    pub fn mapped(&self) -> bool {
        !self.crs.trim().is_empty()
    }

    /// Whether it says nothing at all.
    pub fn is_empty(&self) -> bool {
        *self == Georeference::default()
    }

    /// The way the model's +x runs on the map: IFC's `XAxisAbscissa` and `XAxisOrdinate`.
    pub fn x_axis(&self) -> P2 {
        let (sin, cos) = (-self.rotation).to_radians().sin_cos();
        P2(cos, sin)
    }

    /// The rotation (clockwise from grid north to the model's +y) of a model whose +x runs along `(abscissa, ordinate)`.
    pub fn rotation_of(abscissa: f64, ordinate: f64) -> f64 {
        turn(-ordinate.atan2(abscissa).to_degrees())
    }

    /// The way to true north in the model's plan: IFC's `TrueNorth`. Without a `true_north` of its own, grid north's.
    pub fn true_north_way(&self) -> P2 {
        let (sin, cos) = self.true_north.unwrap_or(self.rotation).to_radians().sin_cos();
        P2(-sin, cos)
    }

    /// How far the model's +y is turned clockwise from what `way` points at in its plan (true north, say).
    pub fn turned_from(way: P2) -> f64 {
        turn((-way.0).atan2(way.1).to_degrees())
    }

    /// A point on the map (easting, northing; mm) in the model's plan.
    pub fn to_model(&self, map: P2) -> P2 {
        let d = P2(map.0 - self.eastings, map.1 - self.northings);
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        P2(d.0 * cos - d.1 * sin, d.0 * sin + d.1 * cos)
    }

    /// What breaks a rule, said so it can be put right.
    pub fn check(&self) -> Result<(), String> {
        let texts = [
            ("crs", &self.crs),
            ("geodetic_datum", &self.geodetic_datum),
            ("vertical_datum", &self.vertical_datum),
            ("map_projection", &self.map_projection),
            ("map_zone", &self.map_zone),
        ];
        for (name, t) in texts {
            if t.chars().count() > MAX_TEXT {
                return Err(format!("the georeference's `{name}` is longer than {MAX_TEXT} characters"));
            }
        }
        for (name, x, most) in [
            ("eastings", self.eastings, MAX_MAP),
            ("northings", self.northings, MAX_MAP),
            ("orthogonal_height", self.orthogonal_height, MAX_COORD),
            ("elevation", self.elevation.unwrap_or(0.0), MAX_COORD),
        ] {
            if !(x.is_finite() && x.abs() <= most) {
                return Err(format!("the georeference's `{name}` must be a number of mm within ±{most:e}, not {x}"));
            }
        }
        for (name, x, most) in [
            ("rotation", self.rotation, 180.0),
            ("true_north", self.true_north.unwrap_or(0.0), 180.0),
            ("latitude", self.latitude.unwrap_or(0.0), 90.0),
            ("longitude", self.longitude.unwrap_or(0.0), 180.0),
        ] {
            if !(x.is_finite() && x.abs() <= most) {
                return Err(format!("the georeference's `{name}` must be degrees within ±{most}, not {x}"));
            }
        }
        let placed = self.eastings != 0.0 || self.northings != 0.0 || self.orthogonal_height != 0.0 || self.rotation != 0.0;
        if placed && !self.mapped() {
            return Err("the georeference places the model on a map but names none: give its `crs` (an EPSG code, `EPSG:25832`)".into());
        }
        Ok(())
    }
}

/// Decimal degrees as IFC's compound plane angle: whole degrees, minutes, seconds and millionths of a second, each
/// with the angle's sign.
pub fn dms(deg: f64) -> [i64; 4] {
    let micro = (deg.abs() * 3600.0 * 1e6).round() as i64;
    let sign = if deg < 0.0 { -1 } else { 1 };
    let (d, rest) = (micro / 3_600_000_000, micro % 3_600_000_000);
    [sign * d, sign * (rest / 60_000_000), sign * (rest % 60_000_000 / 1_000_000), sign * (rest % 1_000_000)]
}

/// IFC's compound plane angle (degrees, minutes, seconds, and millionths of a second if given) as decimal degrees.
pub fn degrees(parts: &[i64]) -> Option<f64> {
    let [d, m, s, rest @ ..] = parts else { return None };
    let micro = rest.first().copied().unwrap_or(0);
    let deg = *d as f64 + *m as f64 / 60.0 + *s as f64 / 3600.0 + micro as f64 / 3.6e9;
    deg.is_finite().then_some(deg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latitudes_go_to_degrees_minutes_seconds_and_back() {
        assert_eq!(dms(54.175), [54, 10, 30, 0]);
        assert_eq!(dms(-9.5000005), [-9, -30, 0, -1800]);
        for x in [54.17531234, -9.5000005, 0.0, 179.999999, -89.123456] {
            let back = degrees(&dms(x)).unwrap();
            assert!((back - x).abs() < 1e-9, "{x}: {back}");
        }
        assert_eq!(degrees(&[1, 2]), None);
    }

    #[test]
    fn a_model_turned_from_grid_north_runs_its_x_axis_so_on_the_map() {
        let g = Georeference { crs: "EPSG:25832".into(), rotation: 21.9, ..Georeference::default() };
        let x = g.x_axis();
        // Its +y at 21.9° east of north, so its +x at 111.9°: east and a little south.
        assert!((x.0 - 21.9f64.to_radians().cos()).abs() < 1e-12 && (x.1 + 21.9f64.to_radians().sin()).abs() < 1e-12);
        assert!((Georeference::rotation_of(x.0, x.1) - 21.9).abs() < 1e-9);
        // True north, which grid north stands for, lies 21.9° to the left of its +y.
        let n = g.true_north_way();
        assert!((Georeference::turned_from(n) - 21.9).abs() < 1e-9);
        assert!(n.0 < 0.0 && n.1 > 0.0);
        // A point 1 m due north of its origin on the map lies 1 m toward grid north in its plan.
        let q = g.to_model(P2(0.0, 1000.0));
        assert!((q.0 - n.0 * 1000.0).abs() < 1e-9 && (q.1 - n.1 * 1000.0).abs() < 1e-9);
    }
}
