use super::super::{Coord, Grid, X, Y};
use super::*;
use grid_loop::{Edge, GridLoop, GridLoopField};
use std::cmp;

#[derive(Clone)]
pub struct Field {
    grid_loop: GridLoop,
    clue: Grid<Clue>,
    cell: Grid<Cell>,
}

const FOUR_NEIGHBORS: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];

impl Field {
    pub fn new(clue: Grid<Clue>) -> Field {
        let height = clue.height();
        let width = clue.width();

        let mut cell = Grid::new(height, width, Cell::Undecided);
        let mut grid_loop = GridLoop::new(height - 1, width - 1);

        {
            let mut handle = GridLoop::get_handle(&mut grid_loop);
            for y in 0..height {
                for x in 0..width {
                    let c = clue[(Y(y), X(x))];
                    if c != Clue::NoClue {
                        cell[(Y(y), X(x))] = Cell::Clue;
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2 - 1), X(x * 2)), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2 + 1), X(x * 2)), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2), X(x * 2 - 1)), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2), X(x * 2 + 1)), Edge::Blank);
                    }
                }
            }
        }
        Field {
            grid_loop,
            clue,
            cell,
        }
    }
    pub fn height(&self) -> i32 {
        self.clue.height()
    }
    pub fn width(&self) -> i32 {
        self.clue.width()
    }
    pub fn inconsistent(&self) -> bool {
        self.grid_loop.inconsistent()
    }
    pub fn set_inconsistent(&mut self) {
        self.grid_loop.set_inconsistent()
    }
    pub fn fully_solved(&self) -> bool {
        self.grid_loop.fully_solved()
    }
    pub fn get_cell(&self, cd: Coord) -> Cell {
        self.cell[cd]
    }
    pub fn get_cell_safe(&self, cd: Coord) -> Cell {
        if self.cell.is_valid_coord(cd) {
            self.cell[cd]
        } else {
            // The outside of the field can be identified with a (meaningless) clue
            Cell::Clue
        }
    }
    pub fn get_edge(&self, cd: Coord) -> Edge {
        self.grid_loop.get_edge(cd)
    }
    pub fn get_edge_safe(&self, cd: Coord) -> Edge {
        self.grid_loop.get_edge_safe(cd)
    }

    pub fn set_cell(&mut self, cd: Coord, v: Cell) {
        let mut handle = GridLoop::get_handle(self);
        handle.set_cell_internal(cd, v);
    }
    pub fn check_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut handle = GridLoop::get_handle(self);
        for y in 0..height {
            for x in 0..width {
                GridLoop::check(&mut *handle, (Y(y * 2), X(x * 2)));
            }
        }
    }

    fn set_cell_internal(&mut self, cd: Coord, v: Cell) {
        let current = self.cell[cd];
        if current != Cell::Undecided {
            if current != v {
                self.set_inconsistent();
            }
            return;
        }

        self.cell[cd] = v;
        let (Y(y), X(x)) = cd;
        match v {
            Cell::Undecided => (),
            Cell::Clue => (), // don't do this!
            Cell::Line => GridLoop::check(self, (Y(y * 2), X(x * 2))),
            Cell::Blocked => for &(dy, dx) in &FOUR_NEIGHBORS {
                if self.get_cell_safe((Y(y + dy), X(x + dx))) != Cell::Clue {
                    self.set_cell_internal((Y(y + dy), X(x + dx)), Cell::Line);
                }
                GridLoop::decide_edge(self, (Y(y * 2 + dy), X(x * 2 + dx)), Edge::Blank);
            },
        }
    }
    fn inspect_clue(&mut self, cell_cd: Coord) {
        let (Y(y), X(x)) = cell_cd;
        let clue = self.clue[cell_cd];
        let (dy, dx, mut n) = match clue {
            Clue::NoClue | Clue::Empty => return,
            Clue::Up(n) => (-1, 0, n),
            Clue::Left(n) => (0, -1, n),
            Clue::Right(n) => (0, 1, n),
            Clue::Down(n) => (1, 0, n),
        };
        let mut involving_cells = 0;
        {
            let mut y = y + dy;
            let mut x = x + dx;
            while self.clue.is_valid_coord((Y(y), X(x))) {
                let c = self.clue[(Y(y), X(x))];
                if c.same_shape(clue) {
                    n -= c.clue_number();
                    break;
                }
                y += dy;
                x += dx;
                involving_cells += 1;
            }
        }
        let mut dp_left = vec![(0, 0); (involving_cells + 1) as usize];
        let mut dp_right = vec![(0, 0); (involving_cells + 1) as usize];

        for i in 0..involving_cells {
            let c = self.get_cell((Y(y + dy * (i + 1)), X(x + dx * (i + 1))));
            dp_left[(i + 1) as usize] = match c {
                Cell::Undecided => {
                    let (lo, hi) = dp_left[cmp::max(0, i - 1) as usize];
                    (lo, hi + 1)
                }
                Cell::Clue | Cell::Line => dp_left[i as usize],
                Cell::Blocked => {
                    let (lo, hi) = dp_left[cmp::max(0, i - 1) as usize];
                    (lo + 1, hi + 1)
                }
            };
        }
        for i in 0..involving_cells {
            let i = involving_cells - 1 - i;
            let c = self.get_cell((Y(y + dy * (i + 1)), X(x + dx * (i + 1))));
            dp_right[i as usize] = match c {
                Cell::Undecided => {
                    let (lo, hi) = dp_right[cmp::min(involving_cells, i + 2) as usize];
                    (lo, hi + 1)
                }
                Cell::Clue | Cell::Line => dp_right[(i + 1) as usize],
                Cell::Blocked => {
                    let (lo, hi) = dp_right[cmp::min(involving_cells, i + 2) as usize];
                    (lo + 1, hi + 1)
                }
            };
        }
        for i in 0..involving_cells {
            let (_, left_hi) = dp_left[i as usize];
            let (_, right_hi) = dp_right[(i + 1) as usize];

            if left_hi + right_hi < n - 1 {
                self.set_inconsistent();
                return;
            } else if left_hi + right_hi == n - 1 {
                self.set_cell_internal((Y(y + dy * (i + 1)), X(x + dx * (i + 1))), Cell::Blocked);
            }
        }
    }
}

impl GridLoopField for Field {
    fn grid_loop(&mut self) -> &mut GridLoop {
        &mut self.grid_loop
    }
    fn check_neighborhood(&mut self, (Y(y), X(x)): Coord) {
        if y % 2 == 0 {
            GridLoop::check(self, (Y(y), X(x - 1)));
            GridLoop::check(self, (Y(y), X(x + 1)));
        } else {
            GridLoop::check(self, (Y(y - 1), X(x)));
            GridLoop::check(self, (Y(y + 1), X(x)));
        }
    }
    fn inspect(&mut self, cd: Coord) {
        let (Y(y), X(x)) = cd;
        if !(y % 2 == 0 && x % 2 == 0) {
            return;
        }
        let cell = self.get_cell((Y(y / 2), X(x / 2)));
        if cell == Cell::Line || cell == Cell::Undecided {
            let (n_line, n_undecided) = self.grid_loop.neighbor_summary(cd);

            if cell == Cell::Line {
                if n_line + n_undecided <= 1 {
                    self.set_inconsistent();
                    return;
                } else if n_line + n_undecided <= 2 {
                    for &(dy, dx) in &FOUR_NEIGHBORS {
                        if self.get_edge_safe((Y(y + dy), X(x + dx))) != Edge::Blank {
                            GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), Edge::Line);
                        }
                    }
                }
            } else {
                if n_line == 0 && n_undecided == 2 {
                    for &(dy, dx) in &FOUR_NEIGHBORS {
                        if self.get_edge_safe((Y(y + dy), X(x + dx))) == Edge::Undecided {
                            self.set_cell_internal((Y(y / 2 + dy), X(x / 2 + dx)), Cell::Line);
                        }
                    }
                }
            }
        } else if cell == Cell::Clue {
            self.inspect_clue((Y(y / 2), X(x / 2)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yajilin_clue() {
        let mut problem = Grid::new(5, 5, Clue::NoClue);
        problem[(Y(2), X(1))] = Clue::Right(2);

        let mut field = Field::new(problem);
        field.check_all_cell();

        assert_eq!(field.inconsistent(), false);
        assert_eq!(field.fully_solved(), true);
        assert_eq!(field.get_cell((Y(2), X(2))), Cell::Blocked);
        assert_eq!(field.get_cell((Y(2), X(4))), Cell::Blocked);
        assert_eq!(field.get_edge((Y(3), X(6))), Edge::Line);
        assert_eq!(field.get_edge((Y(5), X(6))), Edge::Line);
        assert_eq!(field.get_edge((Y(2), X(3))), Edge::Line);
        assert_eq!(field.get_edge((Y(3), X(0))), Edge::Line);
    }
}
