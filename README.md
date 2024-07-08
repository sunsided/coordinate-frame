# Simple coordinate frame conversions

[![Crates.io][crates-image]][crates-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][msrv-image]
[![EUPL 1.2 licensed][license-eupl-image]][license-eupl-link]
[![Apache 2.0 licensed][license-apache-image]][license-apache-link]
[![MIT licensed][license-mit-image]][license-mit-link]

This crate aims at supporting simple conversions between different standard and non-standard
coordinate frames. One potential use-case is in prototyping IMU sensor data where multiple
inertial or field sensors may be mounted in different orientations. These can then be expressed
in terms of coordinate frames such as `EastNorthUp` and trivially converted
to whatever basis you prefer, for example `NorthEastDown`.

## Example

```rust
use coordinate_frame::{NorthEastDown, NorthEastUp};

fn example() {
    // Construct a coordinate in one reference frame.
    let neu = NorthEastUp::new(1.0, 2.0, 3.0);
    assert_eq!(neu.north(), 1.0);
    assert_eq!(neu.east(), 2.0);
    assert_eq!(neu.up(), 3.0);

    // Note that "non-native" axes are also available.
    assert_eq!(neu.down(), -3.0);
  
    // You can transform it into a different frame.
    let ned: NorthEastDown<_> = neu.into();
    assert_eq!(ned.north(), 1.0);
    assert_eq!(ned.east(), 2.0);
    assert_eq!(ned.down(), -3.0);

    // Information is available as you'd expect.
    assert_eq!(ned, &[1.0, 2.0, -3.0]);
    assert_eq!(ned.x(), 1.0);
    assert_eq!(ned.z(), -3.0);

    // Base vectors are also provided.
    let axis = NorthEastDown::<f64>::z_axis();
    assert_eq!(axis, [0.0, 0.0, -1.0]);
}
```

## Code of Conduct

We abide by the [Contributor Covenant][cc] and ask that you do as well.

## License

Copyright © 2024 Markus Mayer

Triple licensed under your choice of either of:

- European Union Public Licence, Version 1.2, ([LICENSE-EUPL](LICENSE-EUPL)
  or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

[crates-image]: https://img.shields.io/crates/v/coordinate-frame

[crates-link]: https://crates.io/crates/coordinate-frame

[docs-image]: https://docs.rs/coordinate-frame/badge.svg

[docs-link]: https://docs.rs/coordinate-frame/

[build-image]: https://github.com/sunsided/coordinate-frame/workflows/Rust/badge.svg

[build-link]: https://github.com/sunsided/coordinate-frame/actions

[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg

[safety-link]: https://github.com/rust-secure-code/safety-dance/

[msrv-image]: https://img.shields.io/badge/rustc-1.64+-blue.svg

[license-eupl-image]: https://img.shields.io/badge/license-EUPL_1.2-blue.svg

[license-apache-image]: https://img.shields.io/badge/license-Apache_2.0-blue.svg

[license-mit-image]: https://img.shields.io/badge/license-MIT-blue.svg

[license-apache-link]: https://github.com/sunsided/coordinate-frame/blob/develop/LICENSE-APACHE

[license-mit-link]: https://github.com/sunsided/coordinate-frame/blob/develop/LICENSE-MIT

[license-eupl-link]: https://github.com/sunsided/coordinate-frame/blob/develop/LICENSE-EUPL

[cc]: https://contributor-covenant.org
