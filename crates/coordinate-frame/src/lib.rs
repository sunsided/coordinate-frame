#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

mod traits;

use coordinate_frame_derive::CoordinateFrame;
pub use traits::*;

#[derive(CoordinateFrame, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum CoordinateFrameType {
    /// Aerospace.
    #[default]
    NorthEastDown,
    NorthEastUp,
    NorthWestDown,
    NorthWestUp,
    NorthDownEast,
    NorthDownWest,
    NorthUpEast,
    NorthUpWest,
    EastNorthDown,
    /// Geography.
    EastNorthUp,
    EastSouthDown,
    EastSouthUp,
    EastDownNorth,
    EastDownSouth,
    EastUpNorth,
    EastUpSouth,
    SouthEastDown,
    SouthEastUp,
    SouthWestDown,
    SouthWestUp,
    SouthDownEast,
    SouthDownWest,
    SouthUpEast,
    SouthUpWest,
    WestNorthDown,
    WestNorthUp,
    WestSouthDown,
    WestSouthUp,
    WestDownNorth,
    WestDownSouth,
    WestUpNorth,
    WestUpSouth,
    DownNorthEast,
    DownNorthWest,
    DownEastNorth,
    DownEastSouth,
    DownSouthEast,
    DownSouthWest,
    DownWestNorth,
    DownWestSouth,
    UpNorthEast,
    UpNorthWest,
    UpEastNorth,
    UpEastSouth,
    UpSouthEast,
    UpSouthWest,
    UpWestNorth,
    UpWestSouth,
    /// An orientation represented by a rotation matrix.
    Other,
}

/// * **X** represents the longitudinal axis with positive values representing "forward".
/// * **Y** represents the lateral axis with positive values representing "right".
/// * **Z** represents the vertical axis with positive values representing "down".
// pub struct NorthEastDown<T>([T; 3]);

/// * **X** represents the longitudinal axis with positive values representing "forward".
/// * **Y** represents the lateral axis with positive values representing "right".
/// * **Z** represents the vertical axis with positive values representing "up".
// pub struct NorthEastUp<T>([T; 3]);

/// * **X** represents the lateral axis with positive values representing "right".
/// * **Y** represents the longitudinal axis with positive values representing "forward".
/// * **Z** represents the vertical axis with positive values representing "up".
// pub struct EastNorthUp<T>([T; 3]);

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
