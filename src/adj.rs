use crate::util;

#[cfg(test)]
fn drain<T: AdjacentIterator>(mut iter: T) -> Vec<(isize, isize)> {
    let mut v = Vec::with_capacity(16);
    while let Some(p) = iter.next() {
        v.push(p);
    }
    for _ in 0..10 {
        assert_eq!(iter.next(), None);
    }
    assert!(util::is_sorted(&v));
    v
}

// additionally required to iterate in lexicographic sorted order and not have duplicates (see unit tests below)
pub trait AdjacentIterator: Iterator<Item = (isize, isize)> + Clone {
    type Open: OpenIterator;
    type Closed: ClosedIterator;

    fn class(row: isize, col: isize) -> usize;
    fn with_class(row: isize, col: isize, class: usize) -> Self;

    fn new(row: isize, col: isize) -> Self {
        Self::with_class(row, col, Self::class(row, col))
    }
    fn at(pos: (isize, isize)) -> Self {
        Self::new(pos.0, pos.1)
    }

    const CLASSES: &'static [(isize, isize)];
}

// some logic might require open/closed so have a special tag for them to use for trait bounds
pub trait OpenIterator: AdjacentIterator {}
pub trait ClosedIterator: AdjacentIterator {}

#[derive(Clone, Copy)]
pub struct ClosedKing {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for ClosedKing {
    type Open = OpenKing;
    type Closed = Self;

    fn class(_: isize, _: isize) -> usize {
        0
    }
    fn with_class(row: isize, col: isize, _: usize) -> Self {
        Self { row, col, state: 0 }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0)];
}
impl Iterator for ClosedKing {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let v = match self.state {
            0 => (self.row - 1, self.col - 1),
            1 => (self.row - 1, self.col),
            2 => (self.row - 1, self.col + 1),
            3 => (self.row, self.col - 1),
            4 => (self.row, self.col),
            5 => (self.row, self.col + 1),
            6 => (self.row + 1, self.col - 1),
            7 => (self.row + 1, self.col),
            8 => (self.row + 1, self.col + 1),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl ClosedIterator for ClosedKing {}
#[test]
fn test_closed_king() {
    assert_eq!(drain(ClosedKing::new(0, 0)), &[(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 0), (0, 1), (1, -1), (1, 0), (1, 1)]);
    assert_eq!(drain(ClosedKing::new(6, -11)), &[(5, -12), (5, -11), (5, -10), (6, -12), (6, -11), (6, -10), (7, -12), (7, -11), (7, -10)]);
}

#[derive(Clone, Copy)]
pub struct OpenKing {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for OpenKing {
    type Open = Self;
    type Closed = ClosedKing;

    fn class(_: isize, _: isize) -> usize {
        0
    }
    fn with_class(row: isize, col: isize, _: usize) -> Self {
        Self { row, col, state: 0 }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0)];
}
impl Iterator for OpenKing {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let v = match self.state {
            0 => (self.row - 1, self.col - 1),
            1 => (self.row - 1, self.col),
            2 => (self.row - 1, self.col + 1),
            3 => (self.row, self.col - 1),
            4 => (self.row, self.col + 1),
            5 => (self.row + 1, self.col - 1),
            6 => (self.row + 1, self.col),
            7 => (self.row + 1, self.col + 1),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl OpenIterator for OpenKing {}
#[test]
fn test_open_king() {
    assert_eq!(drain(OpenKing::new(0, 0)), &[(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]);
    assert_eq!(drain(OpenKing::new(6, -11)), &[(5, -12), (5, -11), (5, -10), (6, -12), (6, -10), (7, -12), (7, -11), (7, -10)]);
}

#[derive(Clone, Copy)]
pub struct ClosedGrid {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for ClosedGrid {
    type Open = OpenGrid;
    type Closed = Self;

    fn class(_: isize, _: isize) -> usize {
        0
    }
    fn with_class(row: isize, col: isize, _: usize) -> Self {
        Self { row, col, state: 0 }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0)];
}
impl Iterator for ClosedGrid {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.row;
        let c = self.col;

        let v = match self.state {
            0 => (r - 1, c),
            1 => (r, c - 1),
            2 => (r, c),
            3 => (r, c + 1),
            4 => (r + 1, c),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl ClosedIterator for ClosedGrid {}
#[test]
fn test_closed_grid() {
    assert_eq!(drain(ClosedGrid::new(0, 0)), &[(-1, 0), (0, -1), (0, 0), (0, 1), (1, 0)]);
    assert_eq!(drain(ClosedGrid::new(6, -11)), &[(5, -11), (6, -12), (6, -11), (6, -10), (7, -11)]);
}

#[derive(Clone, Copy)]
pub struct OpenGrid {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for OpenGrid {
    type Open = Self;
    type Closed = ClosedGrid;

    fn class(_: isize, _: isize) -> usize {
        0
    }
    fn with_class(row: isize, col: isize, _: usize) -> Self {
        Self { row, col, state: 0 }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0)];
}
impl Iterator for OpenGrid {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.row;
        let c = self.col;

        let v = match self.state {
            0 => (r - 1, c),
            1 => (r, c - 1),
            2 => (r, c + 1),
            3 => (r + 1, c),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl OpenIterator for OpenGrid {}
#[test]
fn test_open_grid() {
    assert_eq!(drain(OpenGrid::new(0, 0)), &[(-1, 0), (0, -1), (0, 1), (1, 0)]);
    assert_eq!(drain(OpenGrid::new(6, -11)), &[(5, -11), (6, -12), (6, -10), (7, -11)]);
}

#[derive(Clone, Copy)]
pub struct ClosedTri {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for ClosedTri {
    type Open = OpenTri;
    type Closed = Self;

    fn class(_: isize, _: isize) -> usize {
        0
    }
    fn with_class(row: isize, col: isize, _: usize) -> Self {
        Self { row, col, state: 0 }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0)];
}
impl Iterator for ClosedTri {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.row;
        let c = self.col;

        let v = match self.state {
            0 => (r - 1, c - 1),
            1 => (r - 1, c),
            2 => (r, c - 1),
            3 => (r, c),
            4 => (r, c + 1),
            5 => (r + 1, c),
            6 => (r + 1, c + 1),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl ClosedIterator for ClosedTri {}
#[test]
fn test_closed_tri() {
    assert_eq!(drain(ClosedTri::new(0, 0)), &[(-1, -1), (-1, 0), (0, -1), (0, 0), (0, 1), (1, 0), (1, 1)]);
    assert_eq!(drain(ClosedTri::new(6, -11)), &[(5, -12), (5, -11), (6, -12), (6, -11), (6, -10), (7, -11), (7, -10)]);
}

#[derive(Clone, Copy)]
pub struct OpenTri {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for OpenTri {
    type Open = Self;
    type Closed = ClosedTri;

    fn class(_: isize, _: isize) -> usize {
        0
    }
    fn with_class(row: isize, col: isize, _: usize) -> Self {
        Self { row, col, state: 0 }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0)];
}
impl Iterator for OpenTri {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.row;
        let c = self.col;

        let v = match self.state {
            0 => (r - 1, c - 1),
            1 => (r - 1, c),
            2 => (r, c - 1),
            3 => (r, c + 1),
            4 => (r + 1, c),
            5 => (r + 1, c + 1),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl OpenIterator for OpenTri {}
#[test]
fn test_open_tri() {
    assert_eq!(drain(OpenTri::new(0, 0)), &[(-1, -1), (-1, 0), (0, -1), (0, 1), (1, 0), (1, 1)]);
    assert_eq!(drain(OpenTri::new(6, -11)), &[(5, -12), (5, -11), (6, -12), (6, -10), (7, -11), (7, -10)]);
}

#[derive(Clone, Copy)]
pub struct ClosedHex {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for ClosedHex {
    type Open = OpenHex;
    type Closed = Self;

    fn class(row: isize, col: isize) -> usize {
        if (row + col) % 2 == 0 { 0 } else { 1 }
    }
    fn with_class(row: isize, col: isize, class: usize) -> Self {
        Self {
            row, col,
            state: [0, 5][class],
        }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0), (0, 1)];
}
impl Iterator for ClosedHex {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let v = match self.state {
            0 => (self.row - 1, self.col),
            1 | 5 => (self.row, self.col - 1),
            2 | 6 => (self.row, self.col),
            3 | 7 => (self.row, self.col + 1),
            8 => (self.row + 1, self.col),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl ClosedIterator for ClosedHex {}
#[test]
fn test_closed_hex() {
    assert_eq!(drain(ClosedHex::new(0, 0)), &[(-1, 0), (0, -1), (0, 0), (0, 1)]);
    assert_eq!(drain(ClosedHex::new(1, 0)), &[(1, -1), (1, 0), (1, 1), (2, 0)]);
    assert_eq!(drain(ClosedHex::new(2, 0)), &[(1, 0), (2, -1), (2, 0), (2, 1)]);
    assert_eq!(drain(ClosedHex::new(-3, 0)), &[(-3, -1), (-3, 0), (-3, 1), (-2, 0)]);
    assert_eq!(drain(ClosedHex::new(-10, 0)), &[(-11, 0), (-10, -1), (-10, 0), (-10, 1)]);
    assert_eq!(drain(ClosedHex::new(0, 1)), &[(0, 0), (0, 1), (0, 2), (1, 1)]);
    assert_eq!(drain(ClosedHex::new(1, 1)), &[(0, 1), (1, 0), (1, 1), (1, 2)]);
    assert_eq!(drain(ClosedHex::new(-3, -1)), &[(-4, -1), (-3, -2), (-3, -1), (-3, 0)]);
}

#[derive(Clone, Copy)]
pub struct OpenHex {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for OpenHex {
    type Open = Self;
    type Closed = ClosedHex;

    fn class(row: isize, col: isize) -> usize {
        if (row + col) % 2 == 0 { 0 } else { 1 }
    }
    fn with_class(row: isize, col: isize, class: usize) -> Self {
        Self {
            row, col,
            state: [0, 4][class], 
        }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0), (0, 1)];
}
impl Iterator for OpenHex {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let v = match self.state {
            0 => (self.row - 1, self.col),
            1 | 4 => (self.row, self.col - 1),
            2 | 5 => (self.row, self.col + 1),
            6 => (self.row + 1, self.col),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl OpenIterator for OpenHex {}
#[test]
fn test_open_hex() {
    assert_eq!(drain(OpenHex::new(0, 0)), &[(-1, 0), (0, -1), (0, 1)]);
    assert_eq!(drain(OpenHex::new(1, 0)), &[(1, -1), (1, 1), (2, 0)]);
    assert_eq!(drain(OpenHex::new(2, 0)), &[(1, 0), (2, -1), (2, 1)]);
    assert_eq!(drain(OpenHex::new(-3, 0)), &[(-3, -1), (-3, 1), (-2, 0)]);
    assert_eq!(drain(OpenHex::new(-10, 0)), &[(-11, 0), (-10, -1), (-10, 1)]);
    assert_eq!(drain(OpenHex::new(0, 1)), &[(0, 0), (0, 2), (1, 1)]);
    assert_eq!(drain(OpenHex::new(1, 1)), &[(0, 1), (1, 0), (1, 2)]);
    assert_eq!(drain(OpenHex::new(-3, -1)), &[(-4, -1), (-3, -2), (-3, 0)]);
}

#[derive(Clone, Copy)]
pub struct ClosedTMB {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for ClosedTMB {
    type Open = OpenTMB;
    type Closed = Self;

    fn class(row: isize, col: isize) -> usize {
        util::modulus(row + col, 3)
    }
    fn with_class(row: isize, col: isize, class: usize) -> Self {
        Self {
            row, col,
            state: [0, 8, 13][class],
        }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0), (0, 1), (0, 2)];
}
impl Iterator for ClosedTMB {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let v = match self.state {
            0 | 13 => (self.row - 1, self.col - 1),
            1 | 8 => (self.row - 1, self.col),
            2 | 9 => (self.row, self.col - 1),
            3 | 10 | 14 => (self.row, self.col),
            4 | 15 => (self.row, self.col + 1),
            5 | 16 => (self.row + 1, self.col),
            6 | 11 => (self.row + 1, self.col + 1),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl ClosedIterator for ClosedTMB {}
#[test]
fn test_closed_tmb() {
    assert_eq!(drain(ClosedTMB::new(0, 0)), &[(-1, -1), (-1, 0), (0, -1), (0, 0), (0, 1), (1, 0), (1, 1)]);
    assert_eq!(drain(ClosedTMB::new(0, 1)), &[(-1, 1), (0, 0), (0, 1), (1, 2)]);
    assert_eq!(drain(ClosedTMB::new(0, 2)), &[(-1, 1), (0, 2), (0, 3), (1, 2)]);
    assert_eq!(drain(ClosedTMB::new(-1, -2)), &[(-2, -3), (-2, -2), (-1, -3), (-1, -2), (-1, -1), (0, -2), (0, -1)]);
    assert_eq!(drain(ClosedTMB::new(1, 3)), &[(0, 3), (1, 2), (1, 3), (2, 4)]);
    assert_eq!(drain(ClosedTMB::new(3, 2)), &[(2, 1), (3, 2), (3, 3), (4, 2)]);
    assert_eq!(drain(ClosedTMB::new(-2, -2)), &[(-3, -3), (-2, -2), (-2, -1), (-1, -2)]);
    assert_eq!(drain(ClosedTMB::new(-2, -3)), &[(-3, -3), (-2, -4), (-2, -3), (-1, -2)]);
    assert_eq!(drain(ClosedTMB::new(-2, 2)), &[(-3, 1), (-3, 2), (-2, 1), (-2, 2), (-2, 3), (-1, 2), (-1, 3)]);
    assert_eq!(drain(ClosedTMB::new(1, -2)), &[(0, -3), (1, -2), (1, -1), (2, -2)]);
}

#[derive(Clone, Copy)]
pub struct OpenTMB {
    row: isize,
    col: isize,
    state: usize,
}
impl AdjacentIterator for OpenTMB {
    type Open = Self;
    type Closed = ClosedTMB;

    fn class(row: isize, col: isize) -> usize {
        util::modulus(row + col, 3)
    }
    fn with_class(row: isize, col: isize, class: usize) -> Self {
        Self {
            row, col,
            state: [0, 7, 11][class],
        }
    }

    const CLASSES: &'static [(isize, isize)] = &[(0, 0), (0, 1), (0, 2)];
}
impl Iterator for OpenTMB {
    type Item = (isize, isize);
    fn next(&mut self) -> Option<Self::Item> {
        let v = match self.state {
            0 | 11 => (self.row - 1, self.col - 1),
            1 | 7 => (self.row - 1, self.col),
            2 | 8 => (self.row, self.col - 1),
            3 | 12 => (self.row, self.col + 1),
            4 | 13 => (self.row + 1, self.col),
            5 | 9 => (self.row + 1, self.col + 1),
            _ => return None,
        };
        self.state += 1;
        Some(v)
    }
}
impl OpenIterator for OpenTMB {}
#[test]
fn test_open_tmb() {
    assert_eq!(drain(OpenTMB::new(0, 0)), &[(-1, -1), (-1, 0), (0, -1), (0, 1), (1, 0), (1, 1)]);
    assert_eq!(drain(OpenTMB::new(0, 1)), &[(-1, 1), (0, 0), (1, 2)]);
    assert_eq!(drain(OpenTMB::new(0, 2)), &[(-1, 1), (0, 3), (1, 2)]);
    assert_eq!(drain(OpenTMB::new(-1, -2)), &[(-2, -3), (-2, -2), (-1, -3), (-1, -1), (0, -2), (0, -1)]);
    assert_eq!(drain(OpenTMB::new(1, 3)), &[(0, 3), (1, 2), (2, 4)]);
    assert_eq!(drain(OpenTMB::new(3, 2)), &[(2, 1), (3, 3), (4, 2)]);
    assert_eq!(drain(OpenTMB::new(-2, -2)), &[(-3, -3), (-2, -1), (-1, -2)]);
    assert_eq!(drain(OpenTMB::new(-2, -3)), &[(-3, -3), (-2, -4), (-1, -2)]);
    assert_eq!(drain(OpenTMB::new(-2, 2)), &[(-3, 1), (-3, 2), (-2, 1), (-2, 3), (-1, 2), (-1, 3)]);
    assert_eq!(drain(OpenTMB::new(1, -2)), &[(0, -3), (1, -1), (2, -2)]);
}
