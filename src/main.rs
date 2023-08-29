extern crate rand;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use rand::Rng;

fn main() {
    println!("Hello, world!");
    let sdl_context = sdl2::init().unwrap();

    let video = sdl_context.video().unwrap();

    let window_dim: (u32, u32) = (640, 640);
    let mut grid: Grid = Grid::new(window_dim.0, window_dim.1, 64, 64, 2, 400);
    let window = video
        .window("game of life", window_dim.0, window_dim.1)
        .position(0, 0)
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut step: u8 = 1;
    let mut paused: bool = true;

    let mut event_pump = sdl_context.event_pump().unwrap();

    'run: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'run,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => paused = !paused,
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => step += 1,
                //_ => println!("{event:?}"),
                _ => (),
            }
        }
        if step > 0 || !paused {
            canvas.set_draw_color(Color::RGB(127, 127, 127));
            canvas.clear();

            let table = grid.process_life();
            grid.update_cells(table);

            grid.draw(&mut canvas);

            canvas.present();

            debug_log(paused);
            if step > 0 {
                step -= 1;
            }
        }
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000_u32 / 60));
    }
}

fn debug_log(paused: bool) {
    if paused {
        println!("paused");
    } else {
        println!("unpaused");
    }
}

#[derive(Debug)]
struct Cell {
    position: (u32, u32),
    state: bool,
}

impl Cell {
    pub fn new(position: (u32, u32), state: bool) -> Self {
        Self { position, state }
    }
    pub fn switch(&mut self) {
        self.state = !self.state;
    }
}

struct Grid {
    width: u32,
    height: u32,
    x_pad: u32,
    y_pad: u32,
    x_cells: u32,
    y_cells: u32,
    line_weight: u32,
    cells: Vec<Cell>,
}

impl Grid {
    fn new(width: u32, height: u32, x_cells: u32, y_cells: u32, line_weight: u32, num: u32) -> Self {
        Self {
            x_pad: width / x_cells,
            y_pad: height / y_cells,
            width,
            height,
            x_cells,
            y_cells,
            line_weight,
            cells: Self::init_cells_rand(x_cells, y_cells, num),
        }
    }
    fn init_cells(limit_x: u32, limit_y: u32, positions: &[(u32, u32)]) -> Vec<Cell> {
        positions.into_iter().map(|pos| Cell::new(*pos, true)).collect()
    }
    fn init_cells_rand(limit_x: u32, limit_y: u32, num: u32) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::new();
        for j in 0..limit_y {
            for i in 0..limit_x {
                cells.push(Cell::new((i, j), false));
            }
        }

        let limit = limit_x * limit_y;
        for _ in 0..num {
            let pos = rand::thread_rng().gen_range(0..limit);
            cells[pos as usize].switch();
        }
        cells
    }

    pub fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let mut lines: Vec<sdl2::rect::Rect> = Vec::new();
        for j in 0..=self.y_cells {
            lines.push(sdl2::rect::Rect::new(
                0,
                (j * self.y_pad) as i32,
                self.width,
                self.line_weight,
            ));
        }
        for i in 0..=self.x_cells {
            lines.push(sdl2::rect::Rect::new(
                (i * self.x_pad) as i32,
                0,
                self.line_weight,
                self.height,
            ));
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        for line in lines {
            canvas.fill_rect(line).unwrap();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 122));
        let pad = 0;
        for cell in self.cells.iter() {
            if cell.state == true {
                canvas
                    .fill_rect(sdl2::rect::Rect::new(
                        (cell.position.0 * self.x_pad + pad / 2 + self.line_weight)
                            .try_into()
                            .unwrap(),
                        (cell.position.1 * self.y_pad + pad / 2 + self.line_weight)
                            .try_into()
                            .unwrap(),
                        self.x_pad - self.line_weight - pad as u32,
                        self.y_pad - self.line_weight - pad as u32,
                    ))
                    .unwrap();
            }
        }
    }

    fn update_cells(&mut self, table: Vec<bool>) {
        for i in 0..self.cells.len() {
            self.cells[i].state = table[i];
        }
    }

    fn process_life(&mut self) -> Vec<bool> {
        let mut selection_list: Vec<bool> = Vec::new();
        for i in 0..self.cells.len() {
            let (x, y): (u32, u32) = self.cells[i].position;
            let mut live_count = 0;
            let mut positions_to_check: Vec<u32> = Vec::new();

            if x as i32 - 1 >= 0 {
                positions_to_check.push(sub_until_zero(y, 1) * self.x_cells + x - 1);
                if y + 1 < self.x_cells {
                    positions_to_check.push((y) * self.x_cells + x - 1);
                }
            }
            if x + 1 < self.x_cells {
                positions_to_check.push(sub_until_zero(y, 1) * self.x_cells + x + 1);
                if y + 1 < self.x_cells {
                    positions_to_check.push((y) * self.x_cells + x + 1);
                }
            }
            if y as i32 - 1 >= 0 {
                positions_to_check.push(sub_until_zero(y, 2) * self.x_cells + x);
                if x  as i32 - 1 >= 0 {
                    positions_to_check.push(sub_until_zero(y, 2) * self.x_cells + x - 1);
                }
                if x + 1 < self.x_cells {
                    positions_to_check.push(sub_until_zero(y, 2) * self.x_cells + x + 1);
                }
            }
            if y + 1 < self.x_cells {
                positions_to_check.push(y * self.x_cells + x);
            }

            for pos in positions_to_check {
                if self.cells[pos as usize].state == true {
                    live_count += 1;
                }
            }

            if self.cells[i].state {
                match live_count {
                    0 | 1 => selection_list.push(false),
                    2 | 3 => selection_list.push(true),
                    _ => selection_list.push(false),
                }
            } else {
                match live_count {
                    3 => selection_list.push(true),
                    _ => selection_list.push(self.cells[i].state),
                }
            }
        }
        assert_eq!(self.cells.len(), selection_list.len());
        selection_list
    }

}

fn sub_until_zero(n: u32, i: i32) -> u32 {
    let sub = n as i32 - i;
    if sub < 0 {
        0_u32
    } else {
        sub as u32
    }
}
