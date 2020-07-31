use derive_more::{Add, AddAssign, Into, Mul, MulAssign, Not};

#[derive(
    Default, Debug, Copy, Clone, PartialEq, Eq, Add, AddAssign, Mul, MulAssign, Not, Into, Hash,
)]
pub struct Pos(pub isize, pub isize);

impl Pos {
    pub fn in_range(&self, row1: isize, row2: isize, col1: isize, col2: isize) -> Option<Pos> {
        if self.0 >= row1 && self.0 <= row2 && self.1 >= col1 && self.1 <= col2 {
            Some(*self)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash)]
pub struct Line(pub Pos, pub Pos);

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 || self.0 == other.1 && self.1 == other.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConnectState {
    Open,
    Locked,
    Shortcut,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Connection {
    pub line: Line,
    pub state: ConnectState,
}
