use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::coord::Coord;
use crate::errors::TBError;

fn abs_difference<T: std::ops::Sub<Output = T> + Ord>(x: T, y: T) -> T {
    if x < y {
        y - x
    } else {
        x - y
    }
}

fn find_attack_points(targets: &HashSet<Coord>, dim: usize) -> HashSet<Coord> {
    let mut attack_points = HashSet::new();
    for target in targets {
        for i in 0..dim {
            attack_points.insert(Coord::from((target.x, i)));
            attack_points.insert(Coord::from((i, target.y)));
            if target.x + i < dim {
                if target.y + i < dim {
                    attack_points.insert(Coord::from((target.x + i, target.y + i)));
                }
                if target.y > i {
                    attack_points.insert(Coord::from((target.x + i, target.y - i)));
                }
            }
            if target.x > i {
                if target.y + i < dim {
                    attack_points.insert(Coord::from((target.x - i, target.y + i)));
                }
                if target.y > i {
                    attack_points.insert(Coord::from((target.x - i, target.y - i)));
                }
            }
        }
    }
    attack_points
}

fn find_bombs(targets: &HashSet<Coord>, dim: usize) -> Option<(Coord, Coord)> {
    let b1_attack_points = find_attack_points(targets, dim);
    for b1_attack_point in b1_attack_points {
        let b1_remaining_targets = test_explode(&b1_attack_point, targets);
        log::debug!("bomb 1 ({}) missed {} targets", b1_attack_point, b1_remaining_targets.len());
        let b2_attack_points = find_attack_points(&b1_remaining_targets, dim);
        for b2_attack_point in b2_attack_points {
            let b2_remaining_targets = test_explode(&b2_attack_point, &b1_remaining_targets);
            log::debug!("bomb 2 ({}) missed {} targets", b2_attack_point, b2_remaining_targets.len());
            if b2_remaining_targets.is_empty() {
                log::debug!("Winner! Bomb 1: {} Bomb2: {}", b1_attack_point, b2_attack_point);
                return Some((b1_attack_point, b2_attack_point));
            }
        }
    }
    /*
    for x1 in 0..self.size {
        for y1 in 0..self.size {
            for x2 in 0..self.size {
                for y2 in 0..self.size {
                    let bomb1 = Coord::from((x1, y1));
                    let bomb2 = Coord::from((x2, y2));
                    if self.test_solution(&bomb1, &bomb2) {
                        self.set(&bomb1, TBCell::Bomb)?;
                        self.set(&bomb2, TBCell::Bomb)?;
                        return Ok(Some((bomb1, bomb2)));
                    }
                }
            }
        }
    }
    None
    */
    None
}

fn test_explode(attack_point: &Coord, targets: &HashSet<Coord>) -> HashSet<Coord> {
    targets
        .iter()
        .filter(|t| {
            t.x != attack_point.x
                && t.y != attack_point.y
                && (abs_difference(t.x, attack_point.x) != abs_difference(t.y, attack_point.y))
        })
        .cloned()
        .collect()
}

/*
fn test_solution(&self, bomb1: &Coord, bomb2: &Coord) -> bool {
    let mut test_field = self.cells.clone();
    for i in 0..self.size {
        test_field.remove(&Coord::from((i, bomb1.y)));
        test_field.remove(&Coord::from((bomb1.x, i)));
        if bomb1.x >= i && bomb1.y >= i {
            test_field.remove(&Coord::from((
                bomb1.x.saturating_sub(i),
                bomb1.y.saturating_sub(i),
            )));
        }
        if (bomb1.x + i) < self.size && (bomb1.y + i) < self.size {
            test_field.remove(&Coord::from((bomb1.x + i, bomb1.y + i)));
        }
        if bomb1.x >= i && bomb1.y >= i {
            test_field.remove(&Coord::from((bomb1.x.saturating_sub(i), bomb1.y + i)));
        }
        if (bomb1.x + i) < self.size && bomb1.y >= i {
            test_field.remove(&Coord::from((bomb1.x + i, bomb1.y.saturating_sub(i))));
        }

        test_field.remove(&Coord::from((i, bomb2.y)));
        test_field.remove(&Coord::from((bomb2.x, i)));
        if bomb2.x >= i && bomb2.y >= i {
            test_field.remove(&Coord::from((
                bomb2.x.saturating_sub(i),
                bomb2.y.saturating_sub(i),
            )));
        }
        if (bomb2.x + i) < self.size && (bomb2.y + i) < self.size {
            test_field.remove(&Coord::from((bomb2.x + i, bomb2.y + i)));
        }
        if bomb2.x >= i && (bomb2.y + i) < self.size {
            test_field.remove(&Coord::from((bomb2.x.saturating_sub(i), bomb2.y + i)));
        }
        if (bomb1.x + i) < self.size && bomb1.y >= i {
            test_field.remove(&Coord::from((bomb1.x + i, bomb1.y.saturating_sub(i))));
        }
    }
    test_field.len() == 0
}
*/

// Allow default debug output display
#[derive(Debug)]
// Allow us to do equality tests on enum members, needed for hashing
#[derive(PartialEq, Eq)]
// Allow hashing of TBCell
#[derive(Hash)]
// Give it copy semantics
#[derive(Clone, Copy)]
pub enum TBCell {
    Bomb,
    Target,
}

impl fmt::Display for TBCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TBCell::Bomb => write!(f, "X"),
            TBCell::Target => write!(f, "O"),
        }
    }
}

#[derive(Debug)]
pub struct TBBoard {
    pub cells: HashMap<Coord, TBCell>,
    pub size: usize,
}

// Box drawing chars from https://en.wikipedia.org/wiki/Box-drawing_character
impl fmt::Display for TBBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size = self.get_size();
        let indent: String = "    ".to_string();

        // Write column numbers
        let mut line = indent.clone();
        for x in 0..size {
            line.push(' ');
            line.push_str(x.to_string().as_str());
        }
        writeln!(f, "{}", line)?;

        for y in 0..size {
            if y == 0 {
                // Draw the top of the board
                line = indent.clone();
                line.push_str(self.render_board_top().as_str());
                writeln!(f, "{}", line)?;
            } else {
                // Draw a row separator
                line = indent.clone();
                line.push_str(self.render_board_row_sep().as_str());
                writeln!(f, "{}", line)?;
            }

            // print row of board cells
            // indent + row number
            line = format!("{: >3} ", y);
            line.push_str(self.render_board_row(y).as_str());
            writeln!(f, "{}", line)?;
        }

        // Draw the bottom of the board
        line = indent.clone();
        line.push_str(self.render_board_bottom().as_str());
        writeln!(f, "{}", line)
    }
}

impl TBBoard {
    pub fn render_board_top(&self) -> String {
        // starting and ending char, a char for each board cell, and a char separating each board
        // cell
        let mut line = String::with_capacity(2 + (self.get_size() * 2) - 1);
        for x in 0..self.get_size() {
            if x == 0 {
                line.push('╭');
            } else {
                line.push('┬');
            }
            line.push('─');
        }
        line.push('╮');
        line
    }

    pub fn render_board_bottom(&self) -> String {
        // starting and ending char, a char for each board cell, and a char separating each board
        // cell
        let mut line = String::with_capacity(2 + (self.get_size() * 2) - 1);
        for x in 0..self.get_size() {
            if x == 0 {
                line.push('╰');
            } else {
                line.push('┴');
            }
            line.push('─');
        }
        line.push('╯');
        line
    }

    pub fn render_board_row(&self, y: usize) -> String {
        // starting and ending char, a char for each board cell, and a char separating each board
        // cell
        let mut line = String::with_capacity(2 + (self.get_size() * 2) - 1);
        for x in 0..self.get_size() {
            line.push('│');
            match self.fetch(&Coord { x, y }) {
                Some(TBCell::Bomb) => line.push('X'),
                Some(TBCell::Target) => line.push('O'),
                None => line.push(' '),
            }
        }
        line.push('│');
        line
    }

    pub fn render_board_row_sep(&self) -> String {
        let mut line = String::with_capacity(2 + (self.get_size() * 2) - 1);
        for x in 0..self.get_size() {
            if x == 0 {
                line.push('├');
            } else {
                line.push('┼');
            }
            line.push('─');
        }
        line.push('┤');
        line
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn fetch(&self, coord: &Coord) -> Option<&TBCell> {
        self.cells.get(coord)
    }

    pub fn set(&mut self, coord: &Coord, new_state: TBCell) -> Result<TBCell, TBError> {
        if coord.x <= self.size && coord.y <= self.size {
            self.cells.insert(*coord, new_state);
            Ok(new_state)
        } else {
            Err(TBError::InvalidCellPosition(*coord))
        }
    }

    pub fn clear(&mut self, coord: &Coord) {
        self.cells.remove(coord);
    }

    pub fn mark_target(&mut self, coord: &Coord) {
        if self.cells.get(coord).is_some() {
            self.clear(coord);
        } else {
            self.set(coord, TBCell::Target)
                .expect("Invalid board coordinate for target!");
        }
    }

    pub fn init(&mut self) {
        // reset the board
        self.cells.clear();
        self.cells.reserve((self.size * self.size) as usize);
    }

    pub fn new_anysize(size: usize) -> Self {
        let mut _self = TBBoard {
            cells: HashMap::new(),
            size,
        };
        _self.init();
        _self
    }

    pub fn find_bombs(&mut self) -> Option<(Coord, Coord)> {
        let targets: HashSet<Coord> = self.cells.keys().cloned().collect();
        let bombs = find_bombs(&targets, self.get_size());
        if let Some((b1, b2)) = &bombs {
            self.set(b1, TBCell::Bomb)
                .expect("Invalid board coordinate for bomb!");
            self.set(b2, TBCell::Bomb)
                .expect("Invalid board coordinate for bomb!");
        }

        bombs
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        TBBoard::new_anysize(3)
    }
}

pub struct TBGame {
    pub board: TBBoard,
}

impl TBGame {
    #[allow(dead_code)]
    pub fn get_board(&self) -> &TBBoard {
        &self.board
    }

    pub fn new_anysize(size: usize) -> Self {
        TBGame {
            board: TBBoard::new_anysize(size),
        }
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        TBGame::new_anysize(3)
    }
}
