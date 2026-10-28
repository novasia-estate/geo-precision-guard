use geo_precision_guard::{ApproximateLocation, Point};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    // Invented points: these are not a real property or a verified landmark.
    let listing = ApproximateLocation::new(Point::new(0.0,0.0)?,500.0)?;
    let landmark = ApproximateLocation::new(Point::new(0.0,0.01)?,100.0)?;
    println!("Distance interval: {:?}",listing.distance_bounds(landmark));
    println!("Within 1 km? {:?}",listing.within(landmark,1000.0)?);
    Ok(())
}
