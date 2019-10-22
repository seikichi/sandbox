mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
    depth: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let width = 20;
        let height = 20;
        let depth = 20;

        let cells = (0..width * height * depth)
            .map(|i| {
                if i % 3 == 0 || i % 5 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            depth,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for z in 0..self.depth {
            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = self.get_index(z, y, x);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(z, y, x);

                    // TODO(seikichi): fix here.
                    let next_cell = match (cell, live_neighbors) {
                        (Cell::Alive, x) if x < 4 => Cell::Dead,
                        (Cell::Alive, x) if 4 <= x && x <= 13 => Cell::Alive,
                        (Cell::Alive, x) if x > 13 => Cell::Dead,
                        (otherwise, _) => otherwise,
                    };

                    next[idx] = next_cell;
                }
            }
        }

        self.cells = next;
    }

    fn get_index(&self, z: u32, y: u32, x: u32) -> usize {
        (z * self.width * self.height + y * self.width + x) as usize
    }

    fn live_neighbor_count(&self, z: u32, y: u32, x: u32) -> u8 {
        let mut count = 0;
        for &delta_z in &[self.depth - 1, 0, 1] {
            for &delta_y in &[self.height - 1, 0, 1] {
                for &delta_x in &[self.width - 1, 0, 1] {
                    if delta_z == 0 && delta_y == 0 && delta_x == 0 {
                        continue;
                    }

                    let neighbor_z = (z + delta_z) % self.depth;
                    let neighbor_y = (y + delta_y) % self.height;
                    let neighbor_x = (x + delta_x) % self.width;
                    let idx = self.get_index(neighbor_z, neighbor_y, neighbor_x);
                    count += self.cells[idx] as u8;
                }
            }
        }
        count
    }
}
