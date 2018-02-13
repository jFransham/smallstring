extern crate smallvec;

use std::str;
use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};
use std::borrow::Borrow;
use std::iter::{FromIterator, IntoIterator};
use smallvec::{Array, SmallVec};

#[derive(Clone, Default)]
pub struct SmallString<B: Array<Item = u8>> {
    buffer: SmallVec<B>,
}

impl<'a, B: Array<Item = u8>> SmallString<B> {
    /// Construct an empty string.
    pub fn new() -> Self {
        SmallString {
            buffer: SmallVec::new(),
        }
    }

    /// Constructs an empty string with enough capacity pre-allocated to store
    /// at least `n` bytes worth of characters.
    ///
    /// Will create a heap allocation if and only if `n` is larger than the
    /// inline capacity.
    pub fn with_capacity(n: usize) -> Self {
        SmallString {
            buffer: SmallVec::with_capacity(n),
        }
    }

    /// Constructs a new `SmallString` from a `String` without copying elements.
    pub fn from_string(string: String) -> Self {
        SmallString {
            buffer: SmallVec::from_vec(string.into()),
        }
    }

    /// The maximum number of bytes this string can hold inline.
    pub fn inline_size(&self) -> usize {
        self.buffer.inline_size()
    }

    /// The length of this string in bytes.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// The maximum number of bytes this string can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns `true` if the string has spilled into a heap-allocated buffer.
    pub fn spilled(&self) -> bool {
        self.buffer.spilled()
    }

    /// Appends the given `char` to the end of this string.
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.buffer.push(ch as u8),
            _ => self.buffer
                .extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    /// Removes the last character from the string buffer and returns it.
    ///
    /// Returns `None` if this string is empty.
    pub fn pop(&mut self) -> Option<char> {
        // copied from String::pop implementation.
        let ch = match self.chars().rev().next() {
            Some(ch) => ch,
            None => return None,
        };

        let new_len = self.len() - ch.len_utf8();

        // self.buffer.set_len might be more efficient, but this *should*
        // compile down to the same thing, and it is more safe in case
        // SmallVec::set_len's implementation changes.
        self.buffer.truncate(new_len);

        Some(ch)
    }

    /// Appends a given string slice onto the end of this string.
    pub fn push_str(&mut self, string: &str) {
        self.buffer.extend_from_slice(string.as_bytes())
    }

    /// Reserve capacity for `additional` bytes to be inserted.
    ///
    /// May reserve more space to avoid frequent reallocations.
    ///
    /// If the new capacity would overflow `usize` then it will be set to
    /// `usize::max_value()` instead. (This means that inserting additional new
    /// elements is not guaranteed to be possible after calling this function.)
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional)
    }

    /// Reserve the minimum capacity for `additional` more bytes to be inserted.
    ///
    /// Panics if new capacity overflows `usize`.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.buffer.reserve_exact(additional)
    }

    /// Shrink the capacity of this `String` to match its length.
    ///
    /// When possible, this will move data from an external heap buffer to the
    /// string's inline storage.
    pub fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit()
    }

    /// Shortens this `String` to the specified length.
    ///
    /// If `new_len > len()`, this has no effect.
    ///
    /// Note that this method has no effect on the allocated capacity of the string
    ///
    /// # Panics
    ///
    /// Panics if `new_len` does not lie on a `char` boundary.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len() {
            assert!(self.is_char_boundary(new_len));
            self.buffer.truncate(new_len);
        }
    }

    /// Removes all text from the string.
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
}

impl<B: Array<Item = u8>> std::hash::Hash for SmallString<B> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let s: &str = self;
        s.hash(state)
    }
}

impl<B: Array<Item = u8>> std::cmp::PartialEq for SmallString<B> {
    fn eq(&self, other: &Self) -> bool {
        let (s1, s2): (&str, &str) = (self, other);
        s1 == s2
    }
}

impl<B: Array<Item = u8>> std::cmp::Eq for SmallString<B> {}

impl<'a, B: Array<Item = u8>> PartialEq<SmallString<B>> for &'a str {
    fn eq(&self, other: &SmallString<B>) -> bool {
        *self == (other as &str)
    }
}

impl<B: Array<Item = u8>> std::fmt::Display for SmallString<B> {
    fn fmt(&self, fm: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let s: &str = SmallString::deref(self);
        s.fmt(fm)
    }
}

impl<B: Array<Item = u8>> std::fmt::Debug for SmallString<B> {
    fn fmt(&self, fm: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: &str = SmallString::deref(self);
        s.fmt(fm)
    }
}

impl<B: Array<Item = u8>> Deref for SmallString<B> {
    type Target = str;

    fn deref(&self) -> &str {
        // We only allow `buffer` to be created from an existing valid string,
        // so this is safe.
        unsafe { str::from_utf8_unchecked(self.buffer.as_ref()) }
    }
}

impl<B: Array<Item = u8>> DerefMut for SmallString<B> {
    fn deref_mut(&mut self) -> &mut str {
        // We only allow `buffer` to be created from an existing valid string,
        // so this is safe.
        unsafe {
            // we would use this method, but it's Rust 1.20+ only.
            // str::from_utf8_unchecked_mut(self.buffer.as_mut())
            // Instead, let's do what String::deref_mut() did before
            // this method existed:
            // https://doc.rust-lang.org/1.3.0/src/collections/string.rs.html#1023-1027
            std::mem::transmute::<&mut [u8], &mut str>(&mut self.buffer[..])
        }
    }
}

impl<B: Array<Item = u8>> AsRef<str> for SmallString<B> {
    fn as_ref(&self) -> &str {
        self // forward to Deref
    }
}

impl<B: Array<Item = u8>> AsMut<str> for SmallString<B> {
    fn as_mut(&mut self) -> &mut str {
        self // forward to DerefMut
    }
}

impl<B: Array<Item = u8>> Extend<char> for SmallString<B> {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        for ch in iterator {
            self.push(ch);
        }
    }
}

impl<'a, B: Array<Item = u8>> Extend<&'a str> for SmallString<B> {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for s in iter {
            self.push_str(s);
        }
    }
}

impl<B: Array<Item = u8>> FromIterator<char> for SmallString<B> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut buf = SmallString::new();
        buf.extend(iter);
        buf
    }
}

impl<'a, B: Array<Item = u8>> FromIterator<&'a str> for SmallString<B> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut buf = SmallString::new();
        buf.extend(iter);
        buf
    }
}

impl<B: Array<Item = u8>> AsRef<OsStr> for SmallString<B> {
    fn as_ref(&self) -> &OsStr {
        let s: &str = self.as_ref();
        s.as_ref()
    }
}

impl<B: Array<Item = u8>> Borrow<str> for SmallString<B> {
    fn borrow(&self) -> &str {
        &self
    }
}

impl<'a, B: Array<Item = u8>> From<&'a str> for SmallString<B> {
    fn from(s: &str) -> Self {
        SmallString {
            buffer: SmallVec::from_slice(s.as_bytes()),
        }
    }
}

impl<B: Array<Item = u8>> From<String> for SmallString<B> {
    fn from(s: String) -> Self {
        SmallString {
            buffer: SmallVec::from_vec(s.into_bytes()),
        }
    }
}

impl<B: Array<Item = u8>> From<SmallString<B>> for String {
    fn from(s: SmallString<B>) -> String {
        unsafe { String::from_utf8_unchecked(s.buffer.into_vec()) }
    }
}
