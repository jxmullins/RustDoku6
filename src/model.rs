// use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Default)]
pub struct Cell {
    pub value: Option<u8>,
    pub is_fixed: bool,
    pub marks: [bool; 6],
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Pencil,
}

pub enum GameState {
    Playing,
    Won,
    About,
}

pub struct Grid {
    pub cells: [[Cell; 6]; 6],
}

use rand::prelude::*;

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::default(); 6]; 6],
        }
    }
    
    // Backtracking solver to fill the grid randomly
    pub fn fill_randomly(&mut self) -> bool {
        let mut rng = rand::rng();
        let mut numbers: [u8; 6] = [1, 2, 3, 4, 5, 6];
        
        for r in 0..6 {
            for c in 0..6 {
                if self.cells[r][c].value.is_none() {
                    numbers.shuffle(&mut rng);
                    for &n in &numbers {
                        if self.is_valid_move(r, c, n) {
                            self.cells[r][c].value = Some(n);
                            if self.fill_randomly() {
                                return true;
                            }
                            self.cells[r][c].value = None;
                        }
                    }
                    return false;
                }
            }
        }
        true
    }

    // Check if placing `value` at (row, col) is valid
    pub fn is_valid_move(&self, row: usize, col: usize, value: u8) -> bool {
        // Row check
        for c in 0..6 {
            if c != col {
                if let Some(v) = self.cells[row][c].value {
                    if v == value {
                        return false;
                    }
                }
            }
        }

        // Col check
        for r in 0..6 {
            if r != row {
                if let Some(v) = self.cells[r][col].value {
                    if v == value {
                        return false;
                    }
                }
            }
        }

        // 2x3 Box check (Standard 6x6 is usually 2 rows x 3 cols regions)
        // Regions are:
        // (0,0)-(1,2), (0,3)-(1,5)
        // (2,0)-(3,2), (2,3)-(3,5)
        // (4,0)-(5,2), (4,3)-(5,5)
        
        let start_row = (row / 2) * 2;
        let start_col = (col / 3) * 3;

        for r in start_row..start_row + 2 {
            for c in start_col..start_col + 3 {
                if r != row || c != col {
                    if let Some(v) = self.cells[r][c].value {
                        if v == value {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn is_full(&self) -> bool {
        for row in 0..6 {
            for col in 0..6 {
                if self.cells[row][col].value.is_none() {
                    return false;
                }
            }
        }
        true
    }
    
    // Check if the board is completely full AND valid (Win condition)
    pub fn is_solved(&self) -> bool {
        if !self.is_full() {
            return false;
        }
        
        for r in 0..6 {
            for c in 0..6 {
                // Safe because is_full() returned true, but use expect for clarity
                let val = self.cells[r][c].value.expect("Cell should have value when grid is full");
                if !self.is_valid_move(r, c, val) {
                    return false;
                }
            }
        }
        true
    }
}

pub struct Game {
    pub grid: Grid,
    pub solution: [[u8; 6]; 6],
    pub cursor: (usize, usize),
    pub state: GameState,
    pub mode: InputMode,
    pub mistakes: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut grid = Grid::new();
        
        // 1. Generate full board
        // Note: fill_randomly should always succeed for valid Sudoku rules,
        // but we verify to prevent potential panics
        let mut success = grid.fill_randomly();
        if !success {
            // This should never happen with valid Sudoku logic,
            // but if it does, try again with a new grid
            grid = Grid::new();
            success = grid.fill_randomly();
            
            // If it fails twice, panic with a clear message
            if !success {
                panic!("Failed to generate a valid Sudoku grid after multiple attempts. This indicates a critical bug in the generation algorithm.");
            }
        }
        
        // 2. Capture Solution
        let mut solution = [[0; 6]; 6];
        for r in 0..6 {
            for c in 0..6 {
                // Safe to unwrap here because fill_randomly succeeded
                solution[r][c] = grid.cells[r][c].value.expect("Grid should be fully filled after successful generation");
            }
        }
        
        // 3. Mark all filled cells as fixed (initially)
        for r in 0..6 {
            for c in 0..6 {
                if grid.cells[r][c].value.is_some() {
                    grid.cells[r][c].is_fixed = true;
                }
            }
        }
        
        // 4. Remove random cells to create puzzle
        let mut rng = rand::rng();
        let mut removed_count = 0;
        let target_removed = 20; // 16 clues left
        
        while removed_count < target_removed {
            let r = rng.random_range(0..6);
            let c = rng.random_range(0..6);
            
            if grid.cells[r][c].value.is_some() {
                grid.cells[r][c].value = None;
                grid.cells[r][c].is_fixed = false;
                removed_count += 1;
            }
        }

        Self {
            grid,
            solution,
            cursor: (0, 0),
            state: GameState::Playing,
            mode: InputMode::Normal,
            mistakes: 0,
        }
    }
    
    // Check if the value matches the solution
    pub fn is_correct_move(&self, row: usize, col: usize, value: u8) -> bool {
        self.solution[row][col] == value
    }

    pub fn move_cursor(&mut self, dr: i8, dc: i8) {
        let new_r = (self.cursor.0 as i8 + dr).clamp(0, 5) as usize;
        let new_c = (self.cursor.1 as i8 + dc).clamp(0, 5) as usize;
        self.cursor = (new_r, new_c);
    }

    pub fn handle_input(&mut self, num: u8) {
        // Validate input is in valid range
        if !(1..=6).contains(&num) {
            return;
        }
        
        let (r, c) = self.cursor;
        if self.grid.cells[r][c].is_fixed {
            return;
        }

        match self.mode {
            InputMode::Normal => {
                // Track mistakes before setting the value
                // Use saturating_add to prevent overflow. In normal gameplay, reaching u32::MAX
                // (4+ billion mistakes) is impossible, but this prevents undefined behavior
                // if the counter is somehow incremented excessively.
                if !self.is_correct_move(r, c, num) {
                    self.mistakes = self.mistakes.saturating_add(1);
                }
                
                self.grid.cells[r][c].value = Some(num);
                // Clear marks on set
                self.grid.cells[r][c].marks = [false; 6];
                
                if self.grid.is_solved() {
                    self.state = GameState::Won;
                }
            }
            InputMode::Pencil => {
                // Toggle mark (num is already validated to be 1..=6)
                let idx = (num - 1) as usize;
                self.grid.cells[r][c].marks[idx] = !self.grid.cells[r][c].marks[idx];
            }
        }
    }
    
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            InputMode::Normal => InputMode::Pencil,
            InputMode::Pencil => InputMode::Normal,
        };
    }
    
    pub fn clear_cell(&mut self) {
        let (r, c) = self.cursor;
        if self.grid.cells[r][c].is_fixed {
            return;
        }
        self.grid.cells[r][c].value = None;
        self.grid.cells[r][c].marks = [false; 6];
    }
}
