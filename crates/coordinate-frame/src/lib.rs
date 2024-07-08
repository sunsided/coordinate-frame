//! # Simple coordinate frame conversions
//!
//! This crate aims at supporting simple conversions between different standard and non-standard
//! coordinate frames. One potential use-case is in prototyping IMU sensor data where multiple
//! inertial or field sensors may be mounted in different orientations. These can then be expressed
//! in terms of coordinate frames such as [`EastNorthUp`] and trivially converted
//! to whatever basis you prefer, for example [`NorthEastDown`].

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod traits;

use coordinate_frame_derive::CoordinateFrame;
pub use traits::*;

/// A coordinate frame type.
#[derive(CoordinateFrame, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum CoordinateFrameType {
    /// Common aerospace reference frame.
    /// See [`NorthEastDown`].
    #[default]
    NorthEastDown,
    /// See [`NorthEastUp`].
    NorthEastUp,
    /// See [`NorthWestDown`].
    NorthWestDown,
    /// See [`NorthWestUp`].
    NorthWestUp,
    /// See [`NorthDownEast`].
    NorthDownEast,
    /// See [`NorthDownWest`].
    NorthDownWest,
    /// See [`NorthUpEast`].
    NorthUpEast,
    /// See [`NorthUpWest`].
    NorthUpWest,
    /// See [`EastNorthDown`].
    EastNorthDown,
    /// Common geography reference frame.
    /// See [`EastNorthUp`].
    EastNorthUp,
    /// See [`EastSouthDown`].
    EastSouthDown,
    /// See [`EastSouthUp`].
    EastSouthUp,
    /// See [`EastDownNorth`].
    EastDownNorth,
    /// See [`EastDownSouth`].
    EastDownSouth,
    /// See [`EastUpNorth`].
    EastUpNorth,
    /// See [`EastUpSouth`].
    EastUpSouth,
    /// See [`SouthEastDown`].
    SouthEastDown,
    /// See [`SouthEastUp`].
    SouthEastUp,
    /// See [`SouthWestDown`].
    SouthWestDown,
    /// See [`SouthWestUp`].
    SouthWestUp,
    /// See [`SouthDownEast`].
    SouthDownEast,
    /// See [`SouthDownWest`].
    SouthDownWest,
    /// See [`SouthUpEast`].
    SouthUpEast,
    /// See [`SouthUpWest`].
    SouthUpWest,
    /// See [`WestNorthDown`].
    WestNorthDown,
    /// See [`WestNorthUp`].
    WestNorthUp,
    /// See [`WestSouthDown`].
    WestSouthDown,
    /// See [`WestSouthUp`].
    WestSouthUp,
    /// See [`WestDownNorth`].
    WestDownNorth,
    /// See [`WestDownSouth`].
    WestDownSouth,
    /// See [`WestUpNorth`].
    WestUpNorth,
    /// See [`WestUpSouth`].
    WestUpSouth,
    /// See [`DownNorthEast`].
    DownNorthEast,
    /// See [`DownNorthWest`].
    DownNorthWest,
    /// See [`DownEastNorth`].
    DownEastNorth,
    /// See [`DownEastSouth`].
    DownEastSouth,
    /// See [`DownSouthEast`].
    DownSouthEast,
    /// See [`DownSouthWest`].
    DownSouthWest,
    /// See [`DownWestNorth`].
    DownWestNorth,
    /// See [`DownWestSouth`].
    DownWestSouth,
    /// See [`UpNorthEast`].
    UpNorthEast,
    /// See [`UpNorthWest`].
    UpNorthWest,
    /// See [`UpEastNorth`].
    UpEastNorth,
    /// See [`UpEastSouth`].
    UpEastSouth,
    /// See [`UpSouthEast`].
    UpSouthEast,
    /// See [`UpSouthWest`].
    UpSouthWest,
    /// See [`UpWestNorth`].
    UpWestNorth,
    /// See [`UpWestSouth`].
    UpWestSouth,
    /// An orientation represented by a rotation matrix.
    Other,
}

#[cfg(test)]
mod tests {
    use crate::{EastNorthUp, NorthEastDown, NorthEastUp};

    #[test]
    fn neu_to_ned() {
        let neu = NorthEastUp::new(0.0, 2.0, 3.0);
        let neu = neu.with_north(1.0);

        assert_eq!(neu.north(), 1.0);
        assert_eq!(neu.east(), 2.0);
        assert_eq!(neu.up(), 3.0);

        // Generated
        assert_eq!(neu.down(), -3.0);

        assert_eq!(neu.north_ref(), &1.0);
        assert_eq!(neu.east_ref(), &2.0);
        assert_eq!(neu.up_ref(), &3.0);

        let ned: NorthEastDown<_> = neu.into();
        assert_eq!(ned.north(), 1.0);
        assert_eq!(ned.east(), 2.0);
        assert_eq!(ned.down(), -3.0);

        assert_eq!(ned.0, [1.0, 2.0, -3.0]);
        assert_eq!(ned.x(), 1.0);
        assert_eq!(ned.z(), -3.0);

        let axis = NorthEastDown::<f64>::z_axis();
        assert_eq!(axis, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn ned_to_enu() {
        let neu = NorthEastDown([1.0, 2.0, 3.0]);
        let enu: EastNorthUp<_> = neu.into();
        assert_eq!(enu.0, [2.0, 1.0, -3.0]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_permutations() {
        const LATERAL: [&str; 2] = ["East", "West"];
        const LONGITUDINAL: [&str; 2] = ["North", "South"];
        const VERTICAL: [&str; 2] = ["Down", "Up"];
        const MUTUALLY_EXCLUSIVE: [[&str; 2]; 3] = [LATERAL, LONGITUDINAL, VERTICAL];
        const DIRECTIONS: [&str; 6] = ["North", "East", "South", "West", "Down", "Up"];
        let mut permutations = Vec::new();
        for x in DIRECTIONS.iter() {
            'y: for y in DIRECTIONS.iter() {
                for pair in MUTUALLY_EXCLUSIVE {
                    if pair.contains(y) && pair.contains(x) {
                        continue 'y;
                    }
                }

                let permutation = format!("{x}{y}");
                'z: for z in DIRECTIONS.iter() {
                    for pair in MUTUALLY_EXCLUSIVE {
                        if pair.contains(z) && (pair.contains(x) || pair.contains(y)) {
                            continue 'z;
                        }
                    }
                    permutations.push(format!("{permutation}{z}"));
                }
            }
        }
        // let str = permutations.join(",");
        assert_eq!(permutations.len(), 48);
    }
}
