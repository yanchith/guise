use core::fmt::Debug;

/// Losslessly converts `n` to `u16` using `TryFrom` or panics.
///
/// # Panics
///
/// Panics if the conversion errors.
pub fn cast_u16<T>(n: T) -> u16
where
    T: TryInto<u16>,
    <T as TryInto<u16>>::Error: Debug,
{
    n.try_into().expect("Expected N to fit in u16")
}

/// Losslessly converts `n` to `u32` using `TryFrom` or panics.
///
/// # Panics
///
/// Panics if the conversion errors.
pub fn cast_u32<T>(n: T) -> u32
where
    T: TryInto<u32>,
    <T as TryInto<u32>>::Error: Debug,
{
    n.try_into().expect("Expected N to fit in u32")
}

/// Losslessly converts `n` to `usize` using `TryFrom` or panics.
///
/// # Panics
///
/// Panics if the conversion errors.
pub fn cast_usize<T>(n: T) -> usize
where
    T: TryInto<usize>,
    <T as TryInto<usize>>::Error: Debug,
{
    n.try_into().expect("Expected N to fit in usize")
}
