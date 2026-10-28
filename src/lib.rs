//! Validate approximate property coordinates without pretending an imprecise pin is an address.
//! No geocoding, network requests or inferred cadastral/legal boundaries.
use std::fmt;

/// Mean spherical Earth radius. Distance is approximate, not survey-grade.
pub const EARTH_RADIUS_METRES: f64 = 6_371_008.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeoError { Latitude, Longitude, Radius, Bounds }
impl fmt::Display for GeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {Self::Latitude=>"latitude must be finite and between -90 and 90",Self::Longitude=>"longitude must be finite and between -180 and 180",Self::Radius=>"radius must be finite and non-negative",Self::Bounds=>"south must not exceed north"})
    }
}
impl std::error::Error for GeoError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point { latitude: f64, longitude: f64 }
impl Point {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, GeoError> {
        if !latitude.is_finite() || !(-90.0..=90.0).contains(&latitude) {return Err(GeoError::Latitude);}
        if !longitude.is_finite() || !(-180.0..=180.0).contains(&longitude) {return Err(GeoError::Longitude);}
        Ok(Self {latitude, longitude})
    }
    pub fn latitude(self) -> f64 {self.latitude}
    pub fn longitude(self) -> f64 {self.longitude}
    pub fn distance_metres(self, other: Self) -> f64 {
        let a = self.latitude.to_radians();
        let b = other.latitude.to_radians();
        let dlat = b-a;
        let dlon = (other.longitude-self.longitude).to_radians();
        let h = ((dlat/2.0).sin().powi(2) + a.cos()*b.cos()*(dlon/2.0).sin().powi(2)).clamp(0.0,1.0);
        2.0*EARTH_RADIUS_METRES*h.sqrt().atan2((1.0-h).sqrt())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ApproximateLocation { point: Point, uncertainty_metres: f64 }
impl ApproximateLocation {
    pub fn new(point: Point, uncertainty_metres: f64) -> Result<Self, GeoError> {
        radius(uncertainty_metres)?;
        Ok(Self {point, uncertainty_metres})
    }
    /// Triangle-inequality bounds for two supplied circular uncertainty regions.
    pub fn distance_bounds(self, other: Self) -> DistanceBounds {
        let centre = self.point.distance_metres(other.point);
        let uncertainty = self.uncertainty_metres + other.uncertainty_metres;
        DistanceBounds {min_metres: (centre-uncertainty).max(0.0), max_metres: (centre+uncertainty).min(std::f64::consts::PI*EARTH_RADIUS_METRES)}
    }
    pub fn within(self, other: Self, threshold_metres: f64) -> Result<Proximity, GeoError> {
        radius(threshold_metres)?;
        let bounds = self.distance_bounds(other);
        Ok(if bounds.max_metres <= threshold_metres {Proximity::DefinitelyWithin}
        else if bounds.min_metres > threshold_metres {Proximity::DefinitelyOutside}
        else {Proximity::Uncertain})
    }
}

fn radius(value: f64) -> Result<(),GeoError> {
    if !value.is_finite() || value < 0.0 {Err(GeoError::Radius)} else {Ok(())}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceBounds {pub min_metres: f64, pub max_metres: f64}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proximity {DefinitelyWithin, DefinitelyOutside, Uncertain}

/// Inclusive longitude box; west > east explicitly means crossing the antimeridian.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {south: f64, west: f64, north: f64, east: f64}
impl BoundingBox {
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Result<Self,GeoError> {
        Point::new(south,west)?; Point::new(north,east)?;
        if south>north {return Err(GeoError::Bounds);}
        Ok(Self {south,west,north,east})
    }
    pub fn contains(self, point: Point) -> bool {
        let latitude = point.latitude>=self.south && point.latitude<=self.north;
        let in_range = |lon:f64| if self.west<=self.east {lon>=self.west && lon<=self.east} else {lon>=self.west || lon<=self.east};
        // -180 and +180 refer to the same meridian.
        latitude && (in_range(point.longitude) || (point.longitude.abs()==180.0 && in_range(-point.longitude)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(a:f64,b:f64)->Point {Point::new(a,b).unwrap()}
    fn approx(a:f64,b:f64,r:f64)->ApproximateLocation {ApproximateLocation::new(p(a,b),r).unwrap()}
    #[test] fn reject_invalid_coordinates(){for a in [91.0,-91.0,f64::NAN,f64::INFINITY] {assert!(Point::new(a,0.0).is_err());}assert!(Point::new(0.0,181.0).is_err());}
    #[test] fn reject_bad_radii(){for r in [-1.0,f64::NAN,f64::INFINITY] {assert!(ApproximateLocation::new(p(0.0,0.0),r).is_err());}}
    #[test] fn distance_identity(){assert_eq!(p(10.0,20.0).distance_metres(p(10.0,20.0)),0.0);}
    #[test] fn known_equator_distance(){let d=p(0.0,0.0).distance_metres(p(0.0,1.0));assert!((d-111195.08).abs()<0.1);}
    #[test] fn symmetric_and_finite_antipodes(){let a=p(90.0,0.0);let b=p(-90.0,180.0);assert!(a.distance_metres(b).is_finite());assert!((a.distance_metres(b)-b.distance_metres(a)).abs()<0.01);}
    #[test] fn uncertainty_changes_decision(){let a=approx(0.0,0.0,500.0);let b=approx(0.0,0.01,100.0);assert_eq!(a.within(b,1000.0).unwrap(),Proximity::Uncertain);assert_eq!(a.within(b,2000.0).unwrap(),Proximity::DefinitelyWithin);assert_eq!(a.within(b,100.0).unwrap(),Proximity::DefinitelyOutside);}
    #[test] fn overlapping_bounds_start_at_zero(){let b=approx(0.0,0.0,100.0).distance_bounds(approx(0.0,0.0,200.0));assert_eq!(b.min_metres,0.0);assert_eq!(b.max_metres,300.0);}
    #[test] fn wrap_box(){let b=BoundingBox::new(-10.0,170.0,10.0,-170.0).unwrap();assert!(b.contains(p(0.0,179.0)));assert!(b.contains(p(0.0,-179.0)));assert!(!b.contains(p(0.0,0.0)));}
    #[test] fn normal_box_and_boundary(){let b=BoundingBox::new(-1.0,100.0,1.0,110.0).unwrap();assert!(b.contains(p(1.0,110.0)));assert!(!b.contains(p(0.0,120.0)));assert!(BoundingBox::new(2.0,0.0,1.0,1.0).is_err());}
    #[test] fn equivalent_dateline(){let b=BoundingBox::new(-1.0,170.0,1.0,180.0).unwrap();assert!(b.contains(p(0.0,-180.0)));}
}
