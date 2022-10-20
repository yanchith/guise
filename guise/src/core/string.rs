use alloc::vec::Vec;
use core::alloc::Allocator;
use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use core::str::{self, FromStr};

use arrayvec::ArrayVec;

#[derive(Debug)]
pub struct FromStrError;

#[derive(Debug)]
pub struct TextCapacityError;

// TODO(yan): Have indexable string type for for wide characters (UTF-32/char).
//
// This means that Deref<Target = str> won't be sufficient for TextStorage, as
// UTF-32 chars do not deref to str and we'll need a different interface to pass
// it to text layout. Currently the only operations we use from string is
// subslicing, char iteration, and char index iteration.
pub trait TextStorage: Deref<Target = str> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, index: usize) -> char;
    fn set(&mut self, index: usize, c: char);
    fn truncate(&mut self, new_len: usize);
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError>;
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError>;
}

#[derive(Debug)]
pub struct AsciiVec<A: Allocator>(Vec<u8, A>);

impl<A: Allocator> AsciiVec<A> {
    pub fn new_in(allocator: A) -> Self {
        Self(Vec::new_in(allocator))
    }

    pub fn from_ascii_str_in(str: &str, allocator: A) -> Self {
        assert!(str.is_ascii());

        let mut data = Vec::with_capacity_in(str.len(), allocator);
        data.extend_from_slice(str.as_bytes());

        Self(data)
    }

    pub fn from_ascii_bytes_in(bytes: &[u8], allocator: A) -> Self {
        assert!(bytes.is_ascii());

        // <[u8]>::is_ascii should imply valid UTF-8
        debug_assert!(str::from_utf8(bytes).is_ok());

        let mut data = Vec::with_capacity_in(bytes.len(), allocator);
        data.extend_from_slice(bytes);

        Self(data)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> char {
        char::from(self.0[index])
    }

    pub fn set(&mut self, index: usize, c: char) {
        assert!(c.is_ascii());
        self.0[index] = c as u8;
    }

    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    pub fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        assert!(s.is_ascii());
        self.0.extend(s.as_bytes());

        Ok(())
    }

    pub fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        let delete_count = delete;
        let insert_count = insert.len();
        let len = self.0.len();

        assert!(index < len);
        assert!(index < len + insert_count - delete_count);

        if insert_count > delete_count {
            let diff = insert_count - delete_count;

            self.0.resize(len + diff, 0);
            self.0.copy_within(index..len, index + diff);
        }

        if insert_count > 0 {
            self.0[index..index + insert_count].copy_from_slice(insert.as_bytes());
        }

        if delete_count > insert_count {
            let diff = delete_count - insert_count;

            self.0.copy_within(index + diff..len, index);
            self.truncate(len - diff);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

// Implemented manually, because PartialEq/Eq/Hash are not implemented for some
// allocators e.g. Global, and derive wouldn't generate the impl. Also hashing
// strings is a bit special in libcore, so we need to match that behavior.
impl<A: Allocator> PartialEq for AsciiVec<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A: Allocator> PartialEq<str> for AsciiVec<A> {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other.as_bytes())
    }
}

impl<'a, A: Allocator> PartialEq<&'a str> for AsciiVec<A> {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other.as_bytes())
    }
}

impl<A: Allocator> Eq for AsciiVec<A> {}

impl<A: Allocator> Hash for AsciiVec<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // str::hash adds a write_u8(0xff) to its hasher, so we must hash this
        // as a &str, not &[u8]
        Hash::hash(unsafe { str::from_utf8_unchecked(&self.0) }, state)
    }
}

impl<A: Allocator> AsRef<str> for AsciiVec<A> {
    fn as_ref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> Borrow<str> for AsciiVec<A> {
    fn borrow(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> Deref for AsciiVec<A> {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> fmt::Display for AsciiVec<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.0) })
    }
}

impl<A: Allocator> fmt::Write for AsciiVec<A> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_extend(s).map_err(|_| fmt::Error)
    }
}

impl<A: Allocator> TextStorage for AsciiVec<A> {
    #[inline]
    fn len(&self) -> usize {
        AsciiVec::len(self)
    }

    #[inline]
    fn get(&self, index: usize) -> char {
        AsciiVec::get(self, index)
    }

    #[inline]
    fn set(&mut self, index: usize, c: char) {
        AsciiVec::set(self, index, c)
    }

    #[inline]
    fn truncate(&mut self, new_len: usize) {
        AsciiVec::truncate(self, new_len)
    }

    #[inline]
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        AsciiVec::try_extend(self, s)
    }

    #[inline]
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        AsciiVec::try_splice(self, index, delete, insert)
    }
}

#[derive(Debug, Clone)]
pub struct AsciiArrayVec<const N: usize>(ArrayVec<u8, N>);

impl<const N: usize> AsciiArrayVec<N> {
    pub const fn const_new() -> Self {
        Self(ArrayVec::new_const())
    }

    pub fn new() -> Self {
        Self(ArrayVec::new())
    }

    pub fn from_ascii_str(str: &str) -> Self {
        assert!(str.is_ascii());

        let mut data: ArrayVec<u8, N> = ArrayVec::new();
        data.try_extend_from_slice(str.as_bytes()).unwrap();

        Self(data)
    }

    pub fn from_ascii_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.is_ascii());

        // <[u8]>::is_ascii should imply valid UTF-8
        debug_assert!(str::from_utf8(bytes).is_ok());

        let mut data: ArrayVec<u8, N> = ArrayVec::new();
        data.try_extend_from_slice(bytes).unwrap();

        Self(data)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> char {
        char::from(self.0[index])
    }

    pub fn set(&mut self, index: usize, c: char) {
        assert!(c.is_ascii());
        self.0[index] = c as u8;
    }

    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    pub fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        assert!(s.is_ascii());
        self.0
            .try_extend_from_slice(s.as_bytes())
            .map_err(|_| TextCapacityError)
    }

    pub fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        let delete_count = delete;
        let insert_count = insert.len();
        let len = self.0.len();

        assert!(index < len);
        assert!(index < len + insert_count - delete_count);

        if insert_count > delete_count {
            let diff = insert_count - delete_count;

            if len + diff > self.0.capacity() {
                return Err(TextCapacityError);
            }

            // SAFETY: ArrayVec::set_len should be safe, because we immediately
            // initialize the values afterwards (slice::copy_within) and never
            // read the uninitialized parts of the slice, but what do I know.
            //
            // This is Vec::resize in the heap-allocated AsciiVec, but
            // ArrayVec does not provide anything of the sort.
            unsafe {
                self.0.set_len(len + diff);
            }
            self.0.copy_within(index..len, index + diff);
        }

        if insert_count > 0 {
            self.0[index..index + insert_count].copy_from_slice(insert.as_bytes());
        }

        if delete_count > insert_count {
            let diff = delete_count - insert_count;

            self.0.copy_within(index + diff..len, index);
            self.truncate(len - diff);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

// Implemented manually, because to have all possible impls of PartialEq, as
// well as matching string hashing from libcore.
impl<const N: usize> PartialEq for AsciiArrayVec<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<const N: usize> PartialEq<str> for AsciiArrayVec<N> {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other.as_bytes())
    }
}

impl<'a, const N: usize> PartialEq<&'a str> for AsciiArrayVec<N> {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other.as_bytes())
    }
}

impl<const N: usize> Eq for AsciiArrayVec<N> {}

impl<const N: usize> Hash for AsciiArrayVec<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // str::hash adds a write_u8(0xff) to its hasher, so we must hash this
        // as a &str, not &[u8]
        Hash::hash(unsafe { str::from_utf8_unchecked(&self.0) }, state)
    }
}

impl<const N: usize> AsRef<str> for AsciiArrayVec<N> {
    fn as_ref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<const N: usize> Borrow<str> for AsciiArrayVec<N> {
    fn borrow(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<const N: usize> Deref for AsciiArrayVec<N> {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<const N: usize> fmt::Display for AsciiArrayVec<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.0) })
    }
}

impl<const N: usize> FromStr for AsciiArrayVec<N> {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(FromStrError);
        }

        if s.len() > N {
            return Err(FromStrError);
        }

        Ok(AsciiArrayVec::from_ascii_str(s))
    }
}

impl<const N: usize> fmt::Write for AsciiArrayVec<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_extend(s).map_err(|_| fmt::Error)
    }
}

impl<const N: usize> TextStorage for AsciiArrayVec<N> {
    #[inline]
    fn len(&self) -> usize {
        AsciiArrayVec::len(self)
    }

    #[inline]
    fn get(&self, index: usize) -> char {
        AsciiArrayVec::get(self, index)
    }

    #[inline]
    fn set(&mut self, index: usize, c: char) {
        AsciiArrayVec::set(self, index, c)
    }

    #[inline]
    fn truncate(&mut self, new_len: usize) {
        AsciiArrayVec::truncate(self, new_len)
    }

    #[inline]
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        AsciiArrayVec::try_extend(self, s)
    }

    #[inline]
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        AsciiArrayVec::try_splice(self, index, delete, insert)
    }
}
