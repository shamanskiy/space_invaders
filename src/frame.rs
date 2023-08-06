use crate::{NUM_COLS, NUM_ROWS};

pub type Frame = Vec<Vec<&'static str>>;

pub fn new_frame() -> Frame {
    let mut cols = Vec::with_capacity(NUM_COLS);
    for _ in 0..NUM_COLS {
        let col: Vec<&str> = vec![" "; NUM_ROWS];
        cols.push(col)
    }
    cols
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
