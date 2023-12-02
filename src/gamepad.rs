use std::fmt::{self, Display};

use rand::prelude::*;
use wasm_bindgen::prelude::*;

pub const CELL_SIZE: usize = 5;

#[wasm_bindgen]
pub fn render_universe(width: usize, height: usize) {
    let mut universe = Universe::new(width, height);
    universe.rand();
    let window = web_sys::window().unwrap_throw();
    let document = window.document().expect_throw("no document in the window");
    // Set the size of the canvas
    let canvas: web_sys::HtmlCanvasElement = match document.get_element_by_id("game-of-life-canvas")
    {
        Some(canvas) => canvas
            .dyn_into()
            .expect_throw("failed to convert the element to a canvas"),
        None => {
            let canvas = document.create_element("canvas").unwrap_throw();
            document
                .body()
                .unwrap_throw()
                .append_child(&canvas)
                .unwrap_throw();
            canvas.dyn_into().unwrap_throw()
        }
    };
    canvas.set_width((CELL_SIZE * universe.width()) as u32);
    canvas.set_height((CELL_SIZE * universe.height()) as u32);
    // Draw the initial universe without cells
    let context: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .expect_throw("failed to get 2d context from canvas")
        .unwrap_throw()
        .dyn_into()
        .expect_throw("failed to convert 2d context to CanvasRenderingContext2d");
    const GRID_COLOR: &str = "#CCCCCC";
    context.set_fill_style(&GRID_COLOR.into());
    context.fill_rect(
        0.0,
        0.0,
        (universe.width() * CELL_SIZE) as f64,
        (universe.height() * CELL_SIZE) as f64,
    );
    // Draw the cells in initial universe
    draw_cells(&context, &universe, false);
    // Start animation loop
    window
        .request_animation_frame(
            Closure::once_into_js(move || {
                render_loop(context, universe);
            })
            .as_ref()
            .unchecked_ref(),
        )
        .expect_throw("failed to request animation frame");
}

fn render_loop(context: web_sys::CanvasRenderingContext2d, mut universe: Universe) {
    universe.tick();
    draw_cells(&context, &universe, true);
    web_sys::window()
        .unwrap_throw()
        .request_animation_frame(
            Closure::once_into_js(move || {
                render_loop(context, universe);
            })
            .as_ref()
            .unchecked_ref(),
        )
        .expect_throw("failed to request animation frame");
}

fn draw_cells(context: &web_sys::CanvasRenderingContext2d, universe: &Universe, dirty: bool) {
    const DEAD_COLOR: &str = "#FFFFFF";
    const ALIVE_COLOR: &str = "#000000";
    for row in 0..universe.height() {
        for col in 0..universe.width() {
            if dirty && !universe.is_dirty(row, col) {
                continue;
            }
            let color = if universe.is_cell_live(row, col) {
                ALIVE_COLOR
            } else {
                DEAD_COLOR
            };
            context.set_fill_style(&color.into());
            context.fill_rect(
                (col * CELL_SIZE + 1) as f64,
                (row * CELL_SIZE + 1) as f64,
                (CELL_SIZE - 2) as f64,
                (CELL_SIZE - 2) as f64,
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<u8>,
    dirty_ring: Vec<u8>,
}

impl Universe {
    pub fn new(width: usize, height: usize) -> Universe {
        assert!(width > 0, "width must be > 0");
        assert!(height > 0, "height must be > 0");
        let len = (width * height).div_ceil(8);
        Universe {
            width,
            height,
            cells: vec![0; len],
            dirty_ring: vec![0; len],
        }
    }

    pub fn rand(&mut self) {
        rand::thread_rng().fill_bytes(&mut self.cells);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn index(&self, row: usize, col: usize) -> (usize, u8) {
        debug_assert!(row < self.height, "row must be < {}", self.height);
        debug_assert!(col < self.width, "column must be < {}", self.width);
        let offset = row * self.width + col;
        (offset / 8, 1 << (offset % 8))
    }

    pub fn is_cell_live(&self, row: usize, col: usize) -> bool {
        let (index, mask) = self.index(row, col);
        (self.cells[index] & mask) != 0
    }

    pub fn is_dirty(&self, row: usize, col: usize) -> bool {
        let (index, mask) = self.index(row, col);
        (self.dirty_ring[index] & mask) != 0
    }

    pub fn toggle_cell(&mut self, row: usize, col: usize) {
        let (index, mask) = self.index(row, col);
        self.cells[index] ^= mask;
    }

    pub fn kill_cell(&mut self, row: usize, col: usize) {
        let (index, mask) = self.index(row, col);
        self.cells[index] &= !mask;
    }

    pub fn revive_cell(&mut self, row: usize, col: usize) {
        let (index, mask) = self.index(row, col);
        self.cells[index] |= mask;
    }

    pub fn live_neighbors(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].into_iter() {
            for delta_col in [self.width - 1, 0, 1].into_iter() {
                if delta_row | delta_col == 0 {
                    continue;
                }
                if self.is_cell_live(
                    (row + delta_row) % self.height,
                    (col + delta_col) % self.width,
                ) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Tick the universe to the next generation.
    /// Returns a vector of cells that have changed.
    pub fn tick(&mut self) {
        let cur = self.clone();
        for row in 0..self.height {
            for column in 0..self.width {
                let self_live = cur.is_cell_live(row, column);
                let live_neighbors = cur.live_neighbors(row, column);
                match (self_live, live_neighbors) {
                    (true, 0..=1) => self.kill_cell(row, column),
                    (true, 4..=7) => self.kill_cell(row, column),
                    (false, 3) => self.revive_cell(row, column),
                    _ => continue,
                }
            }
        }
        for (i, cell) in cur.cells.into_iter().enumerate() {
            self.dirty_ring[i] = cell ^ self.cells[i];
        }
    }
}

impl Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        const CELL_CHAR: [char; 2] = ['□', '■'];
        let mut s = String::with_capacity(self.width * self.height + self.height);
        for row in 0..self.height {
            for column in 0..self.width {
                s.push(CELL_CHAR[self.is_cell_live(row, column) as usize]);
            }
            s.push('\n');
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let u = Universe::new(8, 8);
        assert_eq!(u.index(0, 0), (0, 1));
        assert_eq!(u.index(0, 1), (0, 2));
        assert_eq!(u.index(0, 7), (0, 128));
        assert_eq!(u.index(1, 0), (1, 1));
        assert_eq!(u.index(1, 1), (1, 2));
        assert_eq!(u.index(1, 7), (1, 128));
        assert_eq!(u.index(2, 0), (2, 1));
        assert_eq!(u.index(2, 1), (2, 2));
        assert_eq!(u.index(2, 7), (2, 128));
        assert_eq!(u.index(3, 0), (3, 1));
        assert_eq!(u.index(3, 1), (3, 2));
        assert_eq!(u.index(3, 7), (3, 128));
        assert_eq!(u.index(4, 0), (4, 1));
        assert_eq!(u.index(4, 1), (4, 2));
        assert_eq!(u.index(4, 7), (4, 128));
        assert_eq!(u.index(5, 0), (5, 1));
        assert_eq!(u.index(5, 1), (5, 2));
        assert_eq!(u.index(5, 7), (5, 128));
        assert_eq!(u.index(6, 0), (6, 1));
        assert_eq!(u.index(6, 1), (6, 2));
        assert_eq!(u.index(6, 7), (6, 128));
        assert_eq!(u.index(7, 0), (7, 1));
        assert_eq!(u.index(7, 1), (7, 2));
        assert_eq!(u.index(7, 7), (7, 128));
    }

    #[test]
    fn test_is_cell_live() {
        let mut u = Universe::new(8, 8);
        assert!(!u.is_cell_live(0, 0));
        u.revive_cell(0, 0);
        assert!(u.is_cell_live(0, 0));
        u.kill_cell(0, 0);
        assert!(!u.is_cell_live(0, 0));
    }

    #[test]
    fn test_toggle_cell() {
        let mut u = Universe::new(8, 8);
        assert!(!u.is_cell_live(0, 0));
        u.toggle_cell(0, 0);
        assert!(u.is_cell_live(0, 0));
        u.toggle_cell(0, 0);
        assert!(!u.is_cell_live(0, 0));
    }

    #[test]
    fn test_live_neighbors() {
        let mut u = Universe::new(8, 8);
        assert_eq!(u.live_neighbors(0, 0), 0);
        u.revive_cell(0, 1);
        assert_eq!(u.live_neighbors(0, 0), 1);
        u.revive_cell(1, 0);
        assert_eq!(u.live_neighbors(0, 0), 2);
        u.revive_cell(1, 1);
        assert_eq!(u.live_neighbors(0, 0), 3);
        u.revive_cell(0, 0);
        assert_eq!(u.live_neighbors(0, 0), 3);
        u.revive_cell(0, 7);
        assert_eq!(u.live_neighbors(0, 0), 4);
        u.revive_cell(2, 0);
        assert_eq!(u.live_neighbors(0, 0), 4);
        u.revive_cell(7, 7);
        assert_eq!(u.live_neighbors(0, 0), 5);
        u.revive_cell(7, 0);
        assert_eq!(u.live_neighbors(0, 0), 6);
        u.revive_cell(7, 1);
        assert_eq!(u.live_neighbors(0, 0), 7);
        u.revive_cell(1, 7);
        assert_eq!(u.live_neighbors(0, 0), 8);
    }

    #[test]
    fn test_tick() {
        let mut u = Universe::new(5, 5);
        u.revive_cell(1, 2);
        u.revive_cell(2, 2);
        u.revive_cell(3, 2);
        u.tick();
        assert!(u.is_cell_live(2, 1));
        assert!(u.is_cell_live(2, 2));
        assert!(u.is_cell_live(2, 3));
        u.tick();
        assert!(u.is_cell_live(1, 2));
        assert!(u.is_cell_live(2, 2));
        assert!(u.is_cell_live(3, 2));
    }

    #[test]
    fn test_spaceship() {
        let mut u = Universe::new(5, 5);
        u.revive_cell(1, 2);
        u.revive_cell(2, 3);
        u.revive_cell(3, 1);
        u.revive_cell(3, 2);
        u.revive_cell(3, 3);
        u.tick();
        assert!(u.is_cell_live(2, 1));
        assert!(u.is_cell_live(2, 3));
        assert!(u.is_cell_live(3, 2));
        assert!(u.is_cell_live(3, 3));
        assert!(u.is_cell_live(4, 2));
        u.tick();
        assert!(u.is_cell_live(2, 3));
        assert!(u.is_cell_live(3, 1));
        assert!(u.is_cell_live(3, 3));
        assert!(u.is_cell_live(4, 2));
        assert!(u.is_cell_live(4, 3));
    }
}
