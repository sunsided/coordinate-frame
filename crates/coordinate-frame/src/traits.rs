use crate::{CoordinateFrameType, EastNorthUp, NorthEastDown};

/// A coordinate frame.
pub trait CoordinateFrame {
    /// The type of each coordinate value.
    type Type;

    /// The coordinate frame type.
    const COORDINATE_FRAME: CoordinateFrameType;

    /// Returns the coordinate frame of this instance.
    fn coordinate_frame(&self) -> CoordinateFrameType;

    /// Converts this type to a [`NorthEastDown`] instance.
    fn to_ned(&self) -> NorthEastDown<Self::Type>
    where
        Self::Type: Copy + SaturatingNeg<Output = Self::Type>;

    /// Converts this type to an [`EastNorthUp`] instance.
    fn to_enu(&self) -> EastNorthUp<Self::Type>
    where
        Self::Type: Copy + SaturatingNeg<Output = Self::Type>;
}

/// Performs a saturating negation.
pub trait SaturatingNeg {
    type Output;

    /// Like [`Neg`](core::ops::Neg), but returns the maximum value on negation of
    /// a minimum value instead of overflowing.
    fn saturating_neg(self) -> Self::Output;
}

impl SaturatingNeg for i8 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i16 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}
impl SaturatingNeg for i32 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i64 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i128 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for f32 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        -self
    }
}

impl SaturatingNeg for f64 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        -self
    }
}
