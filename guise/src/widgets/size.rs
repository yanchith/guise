use core::str::FromStr;

#[derive(Debug)]
pub struct TryFromStrError;

enum PositionType {
    Absolute,
    Relative,
}

enum SizeType {
    Absolute,
    AbsoluteNegative,
    Relative,
}

pub struct Position(PositionType, f32);

impl Position {
    pub const fn new_absolute(value: f32) -> Self {
        Self(PositionType::Absolute, value)
    }

    pub const fn new_relative(value: f32) -> Self {
        Self(PositionType::Relative, value)
    }

    pub fn resolve(&self, parent_size: f32) -> f32 {
        match self.0 {
            PositionType::Absolute => self.1,
            PositionType::Relative => self.1 * parent_size,
        }
    }
}

impl From<f32> for Position {
    fn from(value: f32) -> Self {
        Self(PositionType::Absolute, value)
    }
}

impl TryFrom<&str> for Position {
    type Error = TryFromStrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.ends_with('%') {
            let percent = &value[0..value.len() - 1];
            match f32::from_str(percent) {
                Ok(value) => Ok(Self(PositionType::Relative, 0.01 * value)),
                Err(_) => Err(TryFromStrError),
            }
        } else {
            Err(TryFromStrError)
        }
    }
}

pub struct Size(SizeType, f32);

impl Size {
    pub const fn new_absolute(value: f32) -> Self {
        if value.is_sign_positive() {
            Self(SizeType::Absolute, value)
        } else {
            Self(SizeType::AbsoluteNegative, value)
        }
    }

    pub const fn new_relative(value: f32) -> Self {
        Self(SizeType::Relative, value)
    }

    pub fn resolve(&self, parent_size: f32) -> f32 {
        match self.0 {
            SizeType::Absolute => self.1,
            SizeType::AbsoluteNegative => self.1 + parent_size,
            SizeType::Relative => self.1 * parent_size,
        }
    }
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        if value.is_sign_positive() {
            Self(SizeType::Absolute, value)
        } else {
            Self(SizeType::AbsoluteNegative, value)
        }
    }
}

impl TryFrom<&str> for Size {
    type Error = TryFromStrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.ends_with('%') {
            let percent = &value[0..value.len() - 1];
            match f32::from_str(percent) {
                Ok(value) => Ok(Self(SizeType::Relative, 0.01 * value)),
                Err(_) => Err(TryFromStrError),
            }
        } else {
            Err(TryFromStrError)
        }
    }
}
