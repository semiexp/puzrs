use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Y(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X(pub i32);

pub type Coord = (Y, X);

#[derive(Debug, Clone)]
pub struct Grid<T: Clone> {
    height: i32,
    width: i32,
    data: Vec<T>,
}
impl<T: Clone> Grid<T> {
    pub fn new(height: i32, width: i32, default: T) -> Grid<T> {
        Grid {
            height: height,
            width: width,
            data: vec![default; (height * width) as usize],
        }
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn is_valid_coord(&self, (Y(y), X(x)): Coord) -> bool {
        0 <= y && y < self.height && 0 <= x && x < self.width
    }
    pub fn copy_from(&mut self, src: &Grid<T>) where T: Copy {
        assert_eq!(self.height, src.height);
        assert_eq!(self.width, src.width);
        self.data.copy_from_slice(&src.data);
    }
    pub fn index(&self, (Y(y), X(x)): Coord) -> usize {
        (y * self.width + x) as usize
    }
    pub fn coord(&self, idx: usize) -> Coord {
        let idx = idx as i32;
        (Y(idx / self.width), X(idx % self.width))
    }
}
impl<T: Clone> Index<Coord> for Grid<T> {
    type Output = T;
    fn index<'a>(&'a self, idx: Coord) -> &'a T {
        let idx = self.index(idx);
        &self.data[idx]
    }
}
impl<T: Clone> IndexMut<Coord> for Grid<T> {
    fn index_mut<'a>(&'a mut self, idx: Coord) -> &'a mut T {
        let idx = self.index(idx);
        &mut self.data[idx]
    }
}
impl<T: Clone> Index<usize> for Grid<T> {
    type Output = T;
    fn index<'a>(&'a self, idx: usize) -> &'a T {
        &self.data[idx]
    }
}
impl<T: Clone> IndexMut<usize> for Grid<T> {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut T {
        &mut self.data[idx]
    }
}

#[derive(Clone)]
pub struct FiniteSearchQueue {
    top: usize,
    end: usize,
    size: usize,
    queue: Vec<usize>,
    stored: Vec<bool>,
    is_started: bool,
}
impl FiniteSearchQueue {
    pub fn new(max_elem: usize) -> FiniteSearchQueue {
        FiniteSearchQueue {
            top: 0,
            end: 0,
            size: max_elem + 1,
            queue: vec![0; max_elem + 1],
            stored: vec![false; max_elem],
            is_started: false,
        }
    }
    pub fn is_started(&self) -> bool {
        self.is_started
    }
    pub fn start(&mut self) {
        self.is_started = true;
    }
    pub fn finish(&mut self) {
        self.is_started = false;
    }
    pub fn push(&mut self, v: usize) {
        if !self.stored[v] {
            self.stored[v] = true;
            let loc = self.end;
            self.end += 1;
            if self.end == self.size {
                self.end = 0;
            }
            self.queue[loc] = v;
        }
    }
    pub fn pop(&mut self) -> usize {
        let ret = self.queue[self.top];
        self.top += 1;
        if self.top == self.size {
            self.top = 0;
        }
        self.stored[ret] = false;
        ret
    }
    pub fn empty(&mut self) -> bool {
        self.top == self.end
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Symmetry {
    pub dyad: bool,         // 180-degree symmetry
    pub tetrad: bool,       // 90-degree symmetry
    pub horizontal: bool,   // horizontal line symmetry
    pub vertical: bool,     // vertical line symmetry
}

impl Symmetry {
    pub fn none() -> Symmetry {
        Symmetry {
            dyad: false,
            tetrad: false,
            horizontal: false,
            vertical: false,
        }
    }
}

#[cfg(test)]
pub fn vec_to_grid<T>(v: &Vec<Vec<T>>) -> Grid<T>
where
    T: Copy,
{
    if v.len() == 0 {
        panic!("Attempted to convert empty Vec to Grid");
    }
    let ref_len = v[0].len();
    for r in v {
        if r.len() != ref_len {
            panic!("Each element in v must contain the same number of elements");
        }
    }
    let mut flat = vec![];
    for r in v {
        for &x in r {
            flat.push(x);
        }
    }
    Grid {
        height: v.len() as i32,
        width: ref_len as i32,
        data: flat,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid() {
        let mut grid = Grid::new(3, 3, 0);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid[(Y(1), X(1))], 0);
        grid[(Y(1), X(1))] = 4;
        assert_eq!(grid[(Y(1), X(1))], 4);
        assert_eq!(grid[(Y(1), X(0))], 0);
        assert_eq!(grid[(Y(2), X(1))], 0);
        assert_eq!(grid[4], 4);
    }
}
