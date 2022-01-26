use core::str::FromStr;

enum SizeType {
    Absolute,
    Relative,
}

pub struct Size(SizeType, f32);

impl Size {
    pub const fn new_absolute(value: f32) -> Self {
        Self(SizeType::Absolute, value)
    }

    pub const fn new_relative(value: f32) -> Self {
        Self(SizeType::Relative, value)
    }

    pub fn resolve(&self, absolute_parent_size: f32) -> f32 {
        match self.0 {
            SizeType::Absolute => self.1,
            SizeType::Relative => self.1 * absolute_parent_size,
        }
    }
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        Self(SizeType::Absolute, value)
    }
}

#[derive(Debug)]
pub struct TryFromStrError;

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
