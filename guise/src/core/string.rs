use alloc::vec::Vec;
use core::alloc::Allocator;
use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use core::str;

use arrayvec::ArrayVec;

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
    fn get(&self, index: usize) -> char;
    fn set(&mut self, index: usize, c: char);
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError>;
    fn truncate(&mut self, new_len: usize);
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError>;
}

pub struct AsciiVec<A: Allocator>(Vec<u8, A>);

impl<A: Allocator> AsciiVec<A> {
    pub fn new_in(allocator: A) -> Self {
        Self(Vec::new_in(allocator))
    }

    pub fn from_str_in(str: &str, allocator: A) -> Self {
        assert!(str.is_ascii());

        let mut data = Vec::with_capacity_in(str.len(), allocator);
        data.extend_from_slice(str.as_bytes());

        Self(data)
    }

    pub fn from_bytes_in(bytes: &[u8], allocator: A) -> Self {
        assert!(bytes.is_ascii());

        // NB: <[u8]>::is_ascii should imply valid UTF-8
        debug_assert!(str::from_utf8(bytes).is_ok());

        let mut data = Vec::with_capacity_in(bytes.len(), allocator);
        data.extend_from_slice(bytes);

        Self(data)
    }

    pub fn clear(&mut self) {
        self.0.clear();
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

// NB: Implemented manually, so it doesn't fail trying to find a Hash impl for
// the allocator.
impl<A: Allocator> Hash for AsciiVec<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NB: str::hash adds a write_u8(0xff) to its hasher, so we must hash
        // this as a &str, not &[u8]
        Hash::hash(unsafe { str::from_utf8_unchecked(&self.0) }, state)
    }
}

// NB: Implemented manually, because Eq isn't implemented for some allocators
// e.g. Global, and derive wouldn't generate the impl.
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

impl<A: Allocator> TextStorage for AsciiVec<A> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> char {
        char::from(self.0[index])
    }

    fn set(&mut self, index: usize, c: char) {
        assert!(c.is_ascii());
        self.0[index] = c as u8;
    }

    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        assert!(s.is_ascii());
        self.0.extend(s.as_bytes());

        Ok(())
    }

    fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    fn try_splice(
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
}

pub struct AsciiArrayVec<const N: usize>(ArrayVec<u8, N>);

impl<const N: usize> AsciiArrayVec<N> {
    pub fn new() -> Self {
        Self(ArrayVec::new())
    }

    pub fn from_str(str: &str) -> Self {
        assert!(str.is_ascii());

        let mut data: ArrayVec<u8, N> = ArrayVec::new();
        data.try_extend_from_slice(str.as_bytes()).unwrap();

        Self(data)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.is_ascii());

        // NB: <[u8]>::is_ascii should imply valid UTF-8
        debug_assert!(str::from_utf8(bytes).is_ok());

        let mut data: ArrayVec<u8, N> = ArrayVec::new();
        data.try_extend_from_slice(bytes).unwrap();

        Self(data)
    }

    pub fn clear(&mut self) {
        self.0.clear();
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

// NB: Implemented manually, so it doesn't fail trying to find a Hash impl for
// the allocator.
impl<const N: usize> Hash for AsciiArrayVec<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NB: str::hash adds a write_u8(0xff) to its hasher, so we must hash
        // this as a &str, not &[u8]
        Hash::hash(unsafe { str::from_utf8_unchecked(&self.0) }, state)
    }
}

// NB: Implemented manually, because Eq isn't implemented for some allocators
// e.g. Global, and derive wouldn't generate the impl.
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

impl<const N: usize> TextStorage for AsciiArrayVec<N> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> char {
        char::from(self.0[index])
    }

    fn set(&mut self, index: usize, c: char) {
        assert!(c.is_ascii());
        self.0[index] = c as u8;
    }

    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        assert!(s.is_ascii());
        self.0
            .try_extend_from_slice(s.as_bytes())
            .map_err(|_| TextCapacityError)
    }

    fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    fn try_splice(
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
}
