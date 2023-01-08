use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use core::str::{self, Utf8Error};

use arrayvec::ArrayString;

#[derive(Debug)]
pub struct TextCapacityError;

// TODO(yan): Have indexable string type for for wide characters (UTF-32/char).
//
// This means that Deref<Target = str> won't be sufficient for TextStorage, as
// UTF-32 chars do not deref to str and we'll need a different interface to pass
// it to text layout. Currently the only operations we use from string is
// subslicing, char iteration, and char index iteration.
pub trait TextStorage: Deref<Target = str> {
    fn truncate(&mut self, new_len: usize);
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError>;
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError>;
}

// TODO(yan): VecString only exists to store strings with an allocator. This
// should eventually be removed, once Rust has String<A>.
#[derive(Debug)]
pub struct VecString<A: Allocator>(Vec<u8, A>);

impl<A: Allocator> VecString<A> {
    pub fn new_in(allocator: A) -> Self {
        Self(Vec::new_in(allocator))
    }

    pub fn from_str_in(str: &str, allocator: A) -> Self {
        let mut data = Vec::with_capacity_in(str.len(), allocator);
        data.extend_from_slice(str.as_bytes());

        Self(data)
    }

    pub fn from_utf8_in(bytes: &[u8], allocator: A) -> Result<Self, Utf8Error> {
        match str::from_utf8(bytes) {
            Ok(s) => Ok(Self::from_str_in(s, allocator)),
            Err(e) => Err(e),
        }
    }

    pub fn from_utf8_vec(vec: Vec<u8, A>) -> Result<Self, Utf8Error> {
        match str::from_utf8(&vec) {
            Ok(_) => Ok(Self(vec)),
            Err(e) => Err(e),
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        // SAFETY: We preserve the invariant that the data always has to be
        // valid UTF8, the same as String does, so we can cast this before we
        // make any modifications.
        let s = unsafe { str::from_utf8_unchecked(&self.0) };

        // Maintain invariant: the truncated string must stay valid.
        assert!(s.is_char_boundary(new_len));

        self.0.truncate(new_len);
    }

    pub fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        self.0.extend(s.as_bytes());
        Ok(())
    }

    pub fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        {
            // SAFETY: We preserve the invariant that the data always has to be
            // valid UTF8, the same as String does, so we can cast this before
            // we make any modifications.
            let s = unsafe { str::from_utf8_unchecked(&self.0) };

            // Maintain invariant: spliced string must stay valid.
            assert!(s.is_char_boundary(index));
            assert!(s.is_char_boundary(index + delete));
        }

        let delete_byte_count = delete;
        let insert_byte_count = insert.len();
        let len = self.0.len();

        // TODO(yan): @Correctness @Hack This should be able to handle inserting
        // multiple chars at the end of (empty) storage, but currently it first
        // fails on our own assert, and then maybe also somewhere else, so we
        // just redirect it to another method in this case. We should totally
        // audit the entirety of this and cover it with tests.
        if index == len && delete_byte_count == 0 {
            // TODO(yan): Return TextCapacityError here, if we can't reserve.
            return self.try_extend(insert);
        }

        assert!(delete_byte_count <= len);
        assert!(index < len);

        let new_len = len + insert_byte_count - delete_byte_count;
        assert!(index <= new_len);

        if insert_byte_count > delete_byte_count {
            let range = index..len;
            let dst = index + insert_byte_count - delete_byte_count;

            self.0.resize(new_len, 0);
            self.0.copy_within(range, dst);
        }

        if insert_byte_count > 0 {
            self.0[index..index + insert_byte_count].copy_from_slice(insert.as_bytes());
        }

        if delete_byte_count > insert_byte_count {
            let range = index + delete_byte_count..len;
            let dst = index + insert_byte_count;

            self.0.copy_within(range, dst);
            self.0.truncate(new_len);
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
impl<A: Allocator> PartialEq for VecString<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A: Allocator> PartialEq<str> for VecString<A> {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other.as_bytes())
    }
}

impl<'a, A: Allocator> PartialEq<&'a str> for VecString<A> {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other.as_bytes())
    }
}

impl<A: Allocator> Eq for VecString<A> {}

impl<A: Allocator> Hash for VecString<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // str::hash adds a write_u8(0xff) to its hasher, so we must hash this
        // as a &str, not &[u8]
        Hash::hash(unsafe { str::from_utf8_unchecked(&self.0) }, state)
    }
}

impl<A: Allocator> AsRef<str> for VecString<A> {
    fn as_ref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> Borrow<str> for VecString<A> {
    fn borrow(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> Deref for VecString<A> {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> fmt::Display for VecString<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.0) })
    }
}

impl<A: Allocator> fmt::Write for VecString<A> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_extend(s).map_err(|_| fmt::Error)
    }
}

impl<A: Allocator> TextStorage for VecString<A> {
    #[inline]
    fn truncate(&mut self, new_len: usize) {
        VecString::truncate(self, new_len)
    }

    #[inline]
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        VecString::try_extend(self, s)
    }

    #[inline]
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        VecString::try_splice(self, index, delete, insert)
    }
}

impl TextStorage for String {
    #[inline]
    fn truncate(&mut self, new_len: usize) {
        String::truncate(self, new_len)
    }

    #[inline]
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        // TODO(yan): Use fallibale allocation and String::try_reserve.
        String::push_str(self, s);
        Ok(())
    }

    #[inline]
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        // Maintain invariant: spliced string must stay valid.
        assert!(self.is_char_boundary(index));
        assert!(self.is_char_boundary(index + delete));

        let delete_byte_count = delete;
        let insert_byte_count = insert.len();
        let len = self.len();

        // TODO(yan): @Correctness @Hack This should be able to handle inserting
        // multiple chars at the end of (empty) storage, but currently it first
        // fails on our own assert, and then maybe also somewhere else, so we
        // just redirect it to another method in this case. We should totally
        // audit the entirety of this and cover it with tests.
        if index == len && delete_byte_count == 0 {
            // TODO(yan): Return TextCapacityError here, if we can't reserve.
            return self.try_extend(insert);
        }

        assert!(delete_byte_count <= len);
        assert!(index < len);

        let new_len = len + insert_byte_count - delete_byte_count;
        assert!(index <= new_len);

        // SAFETY: We preserve the invariant that the data always has to be
        // valid UTF8, the same as String does... or we at least try to, if
        // there are no bugs.
        let v = unsafe { self.as_mut_vec() };

        if insert_byte_count > delete_byte_count {
            let range = index..len;
            let dst = index + insert_byte_count - delete_byte_count;

            v.resize(new_len, 0);
            v.copy_within(range, dst);
        }

        if insert_byte_count > 0 {
            v[index..index + insert_byte_count].copy_from_slice(insert.as_bytes());
        }

        if delete_byte_count > insert_byte_count {
            let range = index + delete_byte_count..len;
            let dst = index + insert_byte_count;

            v.copy_within(range, dst);
            self.truncate(new_len);
        }

        Ok(())
    }
}

impl<const N: usize> TextStorage for ArrayString<N> {
    #[inline]
    fn truncate(&mut self, new_len: usize) {
        self.truncate(new_len);
    }

    #[inline]
    fn try_extend(&mut self, s: &str) -> Result<(), TextCapacityError> {
        match self.try_push_str(s) {
            Ok(()) => Ok(()),
            Err(_) => Err(TextCapacityError),
        }
    }

    #[inline]
    fn try_splice(
        &mut self,
        index: usize,
        delete: usize,
        insert: &str,
    ) -> Result<(), TextCapacityError> {
        // Maintain invariant: spliced string must stay valid.
        assert!(self.is_char_boundary(index));
        assert!(self.is_char_boundary(index + delete));

        let delete_byte_count = delete;
        let insert_byte_count = insert.len();
        let len = self.len();

        // TODO(yan): @Correctness @Hack This should be able to handle inserting
        // multiple chars at the end of (empty) storage, but currently it first
        // fails on our own assert, and then maybe also somewhere else, so we
        // just redirect it to another method in this case. We should totally
        // audit the entirety of this and cover it with tests.
        if index == len && delete_byte_count == 0 {
            // TODO(yan): Return TextCapacityError here, if we can't reserve.
            return self.try_extend(insert);
        }

        assert!(delete_byte_count <= len);
        assert!(index < len);

        let new_len = len + insert_byte_count - delete_byte_count;
        assert!(index <= new_len);

        if new_len > self.capacity() {
            return Err(TextCapacityError);
        }

        if insert_byte_count > delete_byte_count {
            let range = index..len;
            let dst = index + insert_byte_count - delete_byte_count;

            // SAFETY: ArrayString::set_len should be safe, because we check for
            // capacity beforehand, and because we immediately initialize the
            // values afterwards (slice::copy_within) and never read the
            // uninitialized parts of the slice, but what do I know.
            //
            // This is Vec::resize in the heap-allocated AsciiVec, but
            // ArrayVec does not provide anything of the sort.
            unsafe { self.set_len(new_len) };

            // SAFETY: Safe as long as we preserve the invariant that the data
            // always has to be valid UTF8.
            unsafe { self.as_bytes_mut().copy_within(range, dst) };
        }

        if insert_byte_count > 0 {
            // SAFETY: Safe as long as we preserve the invariant that the data
            // always has to be valid UTF8.
            unsafe {
                let b = self.as_bytes_mut();
                b[index..index + insert_byte_count].copy_from_slice(insert.as_bytes());
            }
        }

        if delete_byte_count > insert_byte_count {
            let range = index + delete_byte_count..len;
            let dst = index + insert_byte_count;

            // SAFETY: Safe as long as we preserve the invariant that the data
            // always has to be valid UTF8.
            unsafe {
                let b = self.as_bytes_mut();
                b.copy_within(range, dst);
            }

            self.truncate(new_len);
        }

        Ok(())
    }
}
