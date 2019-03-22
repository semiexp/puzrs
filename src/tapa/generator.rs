use super::super::{Grid, X, Y};
use super::*;

use rand::distributions::Distribution;
use rand::{distributions, Rng};
use std::cmp;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClueConstraint {
    Any,
    Forced,
    Prohibited,
}

pub struct GeneratorOption {
    pub clue_constraint: Grid<ClueConstraint>,
    pub max_clue: Option<i32>,
    pub use_trial_and_error: bool,
}

pub fn generate<R: Rng>(
    opts: &GeneratorOption,
    dic: &Dictionary,
    consecutive_dic: &ConsecutiveRegionDictionary,
    rng: &mut R,
) -> Option<Grid<Clue>> {
    let height = opts.clue_constraint.height();
    let width = opts.clue_constraint.width();
    let mut has_clue = Grid::new(height, width, false);

    let mut problem = Grid::new(height, width, NO_CLUE);
    let mut field = Field::new(height, width, dic, consecutive_dic);
    let mut current_energy = 0i32;
    let mut n_clues = 0;

    for y in 0..height {
        for x in 0..width {
            if opts.clue_constraint[(Y(y), X(x))] == ClueConstraint::Forced {
                has_clue[(Y(y), X(x))] = true;
                n_clues += 1;
            }
        }
    }

    let n_step = height * width * 10;
    let mut temperature = 20f64;

    for s in 0..n_step {
        let mut update_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                if field.cell((Y(y), X(x))) == Cell::Black
                    || opts.clue_constraint[(Y(y), X(x))] == ClueConstraint::Prohibited
                {
                    continue;
                }
                let y2 = height - 1 - y;
                let x2 = width - 1 - x;
                if -1 <= y - y2 && y - y2 <= 1 && -1 <= x - x2 && x - x2 <= 1
                    && (y != y2 || x != x2)
                {
                    continue;
                }
                let mut isok = true;
                for dy in -1..2 {
                    for dx in -1..2 {
                        if (dy != 0 || dx != 0) && has_clue.is_valid_coord((Y(y + dy), X(x + dx)))
                            && has_clue[(Y(y + dy), X(x + dx))]
                        {
                            isok = false;
                        }
                    }
                }
                let mut isok2 = false;
                for dy in -2..3 {
                    for dx in -2..3 {
                        let loc = (Y(y + dy), X(x + dx));
                        if field.cell_checked(loc) == Cell::Undecided {
                            isok2 = true;
                        }
                    }
                }
                if (has_clue[(Y(y), X(x))] && problem[(Y(y), X(x))] == NO_CLUE)
                    || (isok && isok2
                        && (problem[(Y(y), X(x))] != NO_CLUE || rng.gen::<f64>() < 1.0))
                {
                    for v in (-1)..(CLUE_TYPES as i32) {
                        if v == -1 && opts.clue_constraint[(Y(y), X(x))] == ClueConstraint::Forced {
                            continue;
                        }
                        if v == 0 || v == 21 || v == 22 {
                            continue;
                        }
                        let next_clue = Clue(v);
                        if problem[(Y(y), X(x))] != next_clue {
                            update_cand.push(((Y(y), X(x)), next_clue));
                        }
                    }
                }
            }
        }

        rng.shuffle(&mut update_cand);

        let mut updated = false;

        for &(loc, clue) in &update_cand {
            let previous_clue = problem[loc];
            let mut n_clues2 = n_clues;
            let (Y(y), X(x)) = loc;
            let loc2 = (Y(height - 1 - y), X(width - 1 - x));
            if clue == NO_CLUE {
                if loc == loc2 {
                    n_clues2 -= 1;
                } else if problem[loc2] == NO_CLUE {
                    n_clues2 -= 2;
                }
            } else {
                if !has_clue[loc] {
                    n_clues2 += 1;
                }
                if loc != loc2 && !has_clue[loc2] {
                    n_clues2 += 1;
                }
            }
            if let Some(max_clue) = opts.max_clue {
                if max_clue < n_clues2 {
                    continue;
                }
            }

            problem[loc] = clue;
            let next_field = if previous_clue == NO_CLUE {
                let mut f = field.clone();
                f.add_clue(loc, clue);
                f.solve();
                if opts.use_trial_and_error {
                    f.trial_and_error();
                }
                f
            } else {
                solve_test(
                    &problem,
                    &has_clue,
                    opts.use_trial_and_error,
                    dic,
                    consecutive_dic,
                )
            };
            let energy = next_field.decided_cells() - 4 * n_clues2;

            let update = !next_field.inconsistent()
                && (current_energy < energy
                    || rng.gen::<f64>() < ((energy - current_energy) as f64 / temperature).exp());

            if update {
                current_energy = energy;
                n_clues = n_clues2;
                field = next_field;

                let loc2 = (Y(height - 1 - y), X(width - 1 - x));
                if clue == NO_CLUE {
                    if problem[loc2] == NO_CLUE {
                        has_clue[loc] = false;
                        has_clue[loc2] = false;
                    }
                } else {
                    has_clue[loc] = true;
                    has_clue[loc2] = true;
                }

                let mut clue_filled = true;
                for y in 0..height {
                    for x in 0..width {
                        if has_clue[(Y(y), X(x))] && problem[(Y(y), X(x))] == NO_CLUE {
                            clue_filled = false;
                        }
                    }
                }
                if field.fully_solved() && clue_filled {
                    return Some(problem);
                }
                updated = true;
                break;
            }
            problem[loc] = previous_clue;
        }

        if !updated {
            break;
        }

        temperature *= 0.995f64;
    }

    None
}

fn solve_test<'a, 'b>(
    problem: &Grid<Clue>,
    has_clue: &Grid<bool>,
    use_trial_and_error: bool,
    dic: &'a Dictionary,
    consecutive_dic: &'b ConsecutiveRegionDictionary,
) -> Field<'a, 'b> {
    let height = problem.height();
    let width = problem.width();
    let mut ret = Field::new(height, width, dic, consecutive_dic);

    for y in 0..height {
        for x in 0..width {
            let clue = problem[(Y(y), X(x))];
            if clue != NO_CLUE {
                ret.add_clue((Y(y), X(x)), clue);
            } else if has_clue[(Y(y), X(x))] {
                ret.decide((Y(y), X(x)), Cell::White);
            }

            if ret.inconsistent() {
                return ret;
            }
        }
    }

    ret.solve();
    if use_trial_and_error {
        ret.trial_and_error();
    }

    ret
}
