#![no_std]

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MAX_UNIVERSE_SIZE: usize = 8192;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    buffers: [[Cell; MAX_UNIVERSE_SIZE]; 2],
    index: usize,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let index = 0;
        let mut buffers = [[Cell::Dead; MAX_UNIVERSE_SIZE]; 2];

        for i in 0..width * height {
            buffers[index][i as usize] = if i % 2 == 0 || i % 7 == 0 {
                Cell::Alive
            } else {
                Cell::Dead
            };
        }

        Universe {
            width,
            height,
            buffers,
            index,
        }
    }

    pub fn cells(&self) -> *const Cell {
        self.buffers[self.index].as_ptr()
    }

    pub fn tick(&mut self) {
        let next = (self.index + 1) % 2;

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.buffers[self.index][idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise, _) => otherwise,
                };

                self.buffers[next][idx] = next_cell;
            }
        }

        self.index = next;
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for &delta_row in &[self.height - 1, 0, 1] {
            for &delta_col in &[self.width - 1, 0, 1] {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.buffers[self.index][idx] as u8;
            }
        }
        count
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
}
