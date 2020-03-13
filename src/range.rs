use {
    crate::TextSize,
    std::{
        cmp, fmt,
        ops::{Bound, Index, IndexMut, Range, RangeBounds},
        u32,
    },
};

/// A range in text, represented as a pair of [`TextSize`][struct@TextSize].
///
/// It is a logical error to have `end() < start()`, but
/// code must not assume this is true for `unsafe` guarantees.
///
/// # Translation from `text_unit`
///
/// - `TextRange::from_to(from, to)`        ⟹ `TextRange::from(from..to)`
/// - `TextRange::offset_len(offset, size)` ⟹ `TextRange::from(offset..offset + size)`
/// - `range.start()`                       ⟹ `range.start()`
/// - `range.end()`                         ⟹ `range.end()`
/// - `range.len()`                         ⟹ `range.len()`<sup>†</sup>
/// - `range.is_empty()`                    ⟹ `range.is_empty()`
/// - `a.is_subrange(b)`                    ⟹ `b.contains(a)`
/// - `a.intersection(b)`                   ⟹ `TextRange::intersection(a, b)`
/// - `a.extend_to(b)`                      ⟹ `TextRange::covering(a, b)`
/// - `range.contains(offset)`              ⟹ `range.contains_exclusive(point)`
/// - `range.contains_inclusive(offset)`    ⟹ `range.contains_inclusive(point)`
///
/// † See the note on [`TextRange::len`] for differing behavior for incorrect reverse ranges.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TextRange {
    // Invariant: start <= end
    start: TextSize,
    end: TextSize,
}

impl fmt::Debug for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start(), self.end())
    }
}

/// Creates a new `TextRange` with given `start` and `end.
///
/// # Panics
///
/// Panics if `end < start`.
#[allow(non_snake_case)]
pub fn TextRange(start: TextSize, end: TextSize) -> TextRange {
    assert!(start <= end);
    TextRange { start, end }
}

/// Identity methods.
impl TextRange {
    /// Creates a zero-length range at the specified offset.
    pub const fn empty(self, offset: TextSize) -> TextRange {
        TextRange {
            start: offset,
            end: offset,
        }
    }

    /// The start point of this range.
    pub const fn start(self) -> TextSize {
        self.start
    }

    /// The end point of this range.
    pub const fn end(self) -> TextSize {
        self.end
    }

    /// The size of this range.
    pub const fn len(self) -> TextSize {
        // HACK for const fn: math on primitives only
        [
            TextSize(self.end.raw - self.start.raw), // true
            TextSize::INF,                           // false
        ][((self.end.raw < u32::MAX) & !self.is_empty()) as usize]
    }

    /// Check if this range empty or reversed.
    ///
    /// When `end() < start()`, this returns false.
    /// Code should prefer `is_empty()` to `len() == 0`.
    pub const fn is_empty(self) -> bool {
        // HACK for const fn: math on primitives only
        self.start().raw == self.end().raw
    }
}

/// Manipulation methods.
impl TextRange {
    /// Check if this range contains an offset.
    ///
    /// The end index is considered excluded.
    pub fn contains(self, offset: TextSize) -> bool {
        self.start() <= offset && offset < self.end()
    }

    /// Check if this range contains an offset.
    ///
    /// The end index is considered included.
    pub fn contains_inclusive(self, offset: TextSize) -> bool {
        let point = offset.into();
        self.start() <= point && point <= self.end()
    }

    /// Check if this range completely contains another range.
    pub fn contains_range(self, other: TextRange) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    /// The range covered by both ranges, if it exists.
    /// If the ranges touch but do not overlap, the output range is empty.
    pub fn intersection(lhs: TextRange, rhs: TextRange) -> Option<TextRange> {
        let start = cmp::max(lhs.start(), rhs.start());
        let end = cmp::min(lhs.end(), rhs.end());
        if end < start {
            return None;
        }
        Some(TextRange(start, end))
    }

    /// The smallest range that completely contains both ranges.
    pub fn covering(lhs: TextRange, rhs: TextRange) -> TextRange {
        let start = cmp::min(lhs.start(), rhs.start());
        let end = cmp::max(lhs.end(), rhs.end());
        TextRange(start, end)
    }
}

impl Index<TextRange> for str {
    type Output = str;
    fn index(&self, index: TextRange) -> &Self::Output {
        let start: usize = index.start().into();
        match index.end {
            TextSize::INF => &self[start..],
            end => &self[start..end.into()],
        }
    }
}

impl IndexMut<TextRange> for str {
    fn index_mut(&mut self, index: TextRange) -> &mut Self::Output {
        let start: usize = index.start().into();
        match index.end {
            TextSize::INF => &mut self[start..],
            end => &mut self[start..end.into()],
        }
    }
}

impl RangeBounds<TextSize> for TextRange {
    fn start_bound(&self) -> Bound<&TextSize> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&TextSize> {
        match &self.end {
            &TextSize::INF => Bound::Unbounded,
            end => Bound::Excluded(end),
        }
    }
}

// now questionable
impl<T> From<TextRange> for Range<T>
where
    T: From<TextSize>,
{
    fn from(r: TextRange) -> Self {
        r.start().into()..r.end().into()
    }
}
