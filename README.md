# Geo Precision Guard

**A property pin is not necessarily a property address.** This dependency-free Rust library checks coordinate ranges, calculates uncertainty-aware distance bounds and handles bounding boxes that cross the antimeridian.

Built by [NovAsia Estate](https://novasia.estate/) for real estate data workflows involving Cambodia, Thailand and Bali. Approximate pins, neighbourhood centroids and withheld addresses should carry explicit uncertainty rather than an invented level of precision.

## Use an interval before claiming “within 1 km”

```rust
use geo_precision_guard::{ApproximateLocation, Point, Proximity};

let listing = ApproximateLocation::new(Point::new(0.0, 0.0)?, 500.0)?;
let landmark = ApproximateLocation::new(Point::new(0.0, 0.01)?, 100.0)?;
let bounds = listing.distance_bounds(landmark);
assert!(bounds.min_metres < 1000.0 && bounds.max_metres > 1000.0);
assert_eq!(listing.within(landmark, 1000.0)?, Proximity::Uncertain);
# Ok::<(), geo_precision_guard::GeoError>(())
```

The centre points are about 1,112 m apart. With the supplied 500 m and 100 m uncertainty radii, the possible distance is roughly **512–1,712 m**. A confident yes/no would conceal information. These coordinates are synthetic and do not identify a real listing or landmark.

## API at a glance

| Type | Purpose |
| --- | --- |
| `Point::new(lat, lon)` | Reject NaN, infinity and coordinates outside geographic ranges |
| `Point::distance_metres` | Approximate great-circle distance on a mean-radius sphere |
| `ApproximateLocation` | Pair a valid point with a caller-supplied uncertainty radius in metres |
| `distance_bounds` | Triangle-inequality lower/upper distance bounds, clamped to spherical limits |
| `within` | DefinitelyWithin / DefinitelyOutside / Uncertain for a supplied threshold |
| `BoundingBox::new(south, west, north, east)` | Inclusive point containment; west > east denotes a dateline-crossing box |

```toml
[dependencies]
geo-precision-guard = "1.0"
```

```sh
cargo test
cargo run --example approximate_pin
cargo publish --dry-run
```

## Why this is a guard, not a geocoder

The library does not infer uncertainty from decimal places. A centroid can have many decimal digits while being kilometres from an actual address. You must provide a defensible radius or retain an unknown location upstream.

Distance uses a spherical Earth radius of 6,371,008.8 m; it is not cadastral, navigation or survey measurement. Bounds are conservative geometric bounds for the supplied circular regions, not a statistical confidence interval. A box checks the centre point only, not the entire uncertainty region. Inclusion never proves ownership, land-use permission, access or flood safety.

No API calls, address lookup, exchange rates, regional defaults, tracking or external dependency. Minimum Rust: 1.70; edition 2021. Longitude ±180 represents the same meridian. Extreme radii are permitted but make the distance interval less informative.

## Property-data workflow

1. Keep original address text and the coordinate source.
2. Validate a pin with this library and preserve an explicit uncertainty radius.
3. Treat `Uncertain` as a request for better evidence, not a false result.
4. Re-check an address with a human or primary document before making location-specific purchase claims.

For the wider record, [Property Schema Validator](https://github.com/novasia-estate/property-schema-validator) checks explicit units and provenance. [NovAsia Estate](https://novasia.estate/) provides the regional property research context. These are complementary resources, not independent endorsements of this code.

## Contributing

MIT licensed. AI assistance was used in authoring code, examples and documentation; behavioral tests and compilation are part of release acceptance. Reproducible edge cases are welcome. Use fictional coordinates in public issues when the original pin is private.
