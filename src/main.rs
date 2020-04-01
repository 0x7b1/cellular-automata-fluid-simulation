use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use std::path::Path;

use minifb::{
    Key,
    WindowOptions,
    Window,
    MouseMode,
    MouseButton,
    KeyRepeat,
};
use vek::Vec2;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;
const N_NEIGHBORS: u8 = 8;
const DELTA: f32 = 1.0;

#[derive(Clone)]
enum Elements {
    Brick,
    Water,
    Empty,
}

#[derive(Clone)]
enum LifeMode {
    Cross,
    Circle,
    Fish,
    Flower,
    Galaxy,
}

#[derive(Clone, PartialEq)]
enum Color {
    Black,
    White,
    Red,
    Yellow,
    Green,
    Cyan,
    Purple,
    Blue,
}

impl Color {
    pub fn get_hex(&self) -> u32 {
        use Color::*;

        match self {
            Black => 0x000000,
            White => 0xffffff,
            Red => 0xff0000,
            Yellow => 0xffff00,
            Green => 0x00ff00,
            Cyan => 0x00ffff,
            Blue => 0x0000ff,
            Purple => 0xff00ff,
        }
    }
}

#[derive(Copy, Clone)]
enum CellElement {
    Empty,
    Wall,
    Solid,

    Water,
    Ground,
}

#[derive(Copy, Clone, Debug)]
struct Cell {
    velocity: Vec2<f32>,
    population: f32,
    spread: f32,
    conductivity: f32,
    element: CellElement,

    // living: bool,
    // solid: bool,
}

impl Cell {
    fn empty() -> Self {
        Cell {
            velocity: Vec2::zero(),
            population: 0.0,
            spread: 0.0,
            conductivity: 1.0,
            element: CellElement::Empty,
            // living: false,
            // solid: false,
        }
    }

    // fn new(living: bool) -> Self {
    //     Cell {
    //         living,
    //         velocity: Vec2::zero(),
    //         solid: false,
    //         element: CellElement::Water,
    //     }
    // }

    fn set_element(&mut self, element: CellElement) {
        match element {
            CellElement::Empty => {
                self.velocity = Vec2::zero();
                self.population = 0.0;
            }
            CellElement::Wall => {
                self.velocity = Vec2::zero();
                self.population = 0.0;
                self.conductivity = 0.0;
                self.population = 400.0;
            }
            CellElement::Water => {
                self.velocity = Vec2::new(0.0, -0.9);
                self.spread = 0.5;
                self.population = 250.0;
            }
            _ => {}
        }
    }

    fn get_color(&self) -> Color {
        match self.living {
            true => Color::Black,
            false => Color::White,
        }
    }

    fn tick(&self, &mut this: Self, &mut left: Self, &mut right: Self, &mut up: Self, &mut down: Self) {
        let flow_factors = [

        ];
    }

    fn flow_factor(&self, vec: Vec2<f32>, other: Self) -> f32 {
        // let pop_box = &mut Square
    }
}

struct World {
    // w: usize,
    // h: usize,
    // cells: Vec<Vec<Cell>>,
    // life_mode: [[i32; 3]; 3],
    // life_mode: Vec<Vec<u8>>,
    // cells: Box<[[Cell; HEIGHT]; WIDTH]>,

    water: [Cell; WIDTH],
    energy: [Cell; WIDTH],
    ground: [Cell; WIDTH],
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            // cells: Box::new([[Cell::empty(); HEIGHT]; WIDTH]),
            // cells: Box::new(Vec::new()),
            // life_mode: Vec::new(),
            water: [Cell::empty(); WIDTH],
            energy: [Cell::empty(); WIDTH],
            ground: [Cell::empty(); WIDTH],
        };

        // Top and bottom walls
        for i in 0..WIDTH {
            this.cells[i][0].set_element(CellElement::Wall);
            this.cells[i][HEIGHT - 1].set_element(CellElement::Wall);
        }

        // Left and right walls
        for j in 0..HEIGHT {
            this.cells[0][j].set_element(CellElement::Wall);
            this.cells[WIDTH - 1][j].set_element(CellElement::Wall);
        }

        this
    }

    fn count_neighbors(&self, i: i32, j: i32) -> u32 {
        let mut count: u32 = 0;

        let arr_i: [i32; 8] = [i - 1, i - 1, i - 1, i, i, i + 1, i + 1, i + 1];
        let arr_j = [j - 1, j, j + 1, j - 1, j + 1, j - 1, j, j + 1];


        for it in 0..N_NEIGHBORS {
            let i_loc = arr_i[it as usize];
            let j_loc = arr_j[it as usize];

            let i_final = (i_loc + WIDTH as i32) % WIDTH as i32;
            let j_final = (j_loc + HEIGHT as i32) % HEIGHT as i32;

            if self.cells[j_final as usize][i_final as usize].living {
                count += 1;
            }
        }

        count
    }

    fn tick(&mut self) {
        let mut new_cells = self.cells.clone();

        for i in 1..WIDTH - 1 {
            for j in 1..HEIGHT - 1 {
                let mut this = new_cells[i][j];
                let mut left = new_cells[i - 1][j];
                let mut right = new_cells[i + 1][j];
                let mut up = new_cells[i][j - 1];
                let mut down = new_cells[i][j + 1];

                self.cells[i][j].tick(this, left, right, up, down);
            }
        }
    }

    fn tick_conway(&mut self) {
        let mut new_cells = self.cells.clone();

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let n_neighbors = self.count_neighbors(i as i32, j as i32);

                let new_living_state = match n_neighbors {
                    0 | 1 => false,
                    2 => self.cells[j][i].living,
                    3 => true,
                    _ => false,
                };

                new_cells[j][i].living = new_living_state;
            }
        }

        self.cells = new_cells;
    }

    fn render(&self, buff: &mut [u32]) {
        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                buff[j * WIDTH + i] = self.cells[j][i].get_color().get_hex();
            }
        }
    }

    fn rectangle(&mut self, start: Vec2<u32>) {
        let end = Vec2::new(start.x + 1, start.y + 1);

        for y in start.y..end.y {
            for x in start.x..end.x {
                self.cells[y as usize][x as usize] = Cell::new(true);
            }
        }
    }

    fn place_figure(&mut self, pos: Vec2<u32>) {
        for j in 0..self.life_mode.len() {
            for i in 0..self.life_mode[j].len() {
                if self.life_mode[j][i] == 1 {
                    self.cells[(pos.y + j as u32) as usize][(pos.x + i as u32) as usize] = Cell::new(true); // TODO: improve this vector notation
                }
            }
        }
    }

    fn paint(&mut self, pos: Vec2<u32>) {
        // Center
        self.rectangle(Vec2::new(pos.x, pos.y));

        // Inner cross
        self.rectangle(Vec2::new(pos.x + 1, pos.y));
        self.rectangle(Vec2::new(pos.x - 1, pos.y));
        self.rectangle(Vec2::new(pos.x, pos.y + 1));
        self.rectangle(Vec2::new(pos.x, pos.y - 1));

        // Outer cross
        self.rectangle(Vec2::new(pos.x + 2, pos.y));
        self.rectangle(Vec2::new(pos.x - 2, pos.y));
        self.rectangle(Vec2::new(pos.x, pos.y + 2));
        self.rectangle(Vec2::new(pos.x, pos.y - 2));

        // Borders
        self.rectangle(Vec2::new(pos.x + 1, pos.y + 1));
        self.rectangle(Vec2::new(pos.x - 1, pos.y + 1));
        self.rectangle(Vec2::new(pos.x + 1, pos.y - 1));
        self.rectangle(Vec2::new(pos.x - 1, pos.y - 1));
    }

    // fn set_figure_mode(&mut self, mode: LifeMode) {
    //     self.life_mode = match mode {
    //         LifeMode::Cross => vec![
    //             vec![0, 1, 0],
    //             vec![1, 1, 1],
    //             vec![0, 1, 0],
    //         ],
    //         LifeMode::Circle => vec![
    //             vec![0, 0, 1, 0, 0],
    //             vec![0, 1, 1, 1, 0],
    //             vec![1, 1, 1, 1, 1],
    //             vec![0, 1, 1, 1, 0],
    //             vec![0, 0, 1, 0, 0],
    //         ],
    //         LifeMode::Fish => vec![
    //             vec![1, 0, 0, 0, 0, 1, 0],
    //             vec![0, 0, 0, 0, 0, 0, 1],
    //             vec![1, 0, 0, 0, 0, 0, 1],
    //             vec![0, 1, 1, 1, 1, 1, 0],
    //         ],
    //         LifeMode::Flower => vec![
    //             vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //             vec![0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1],
    //             vec![0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0],
    //             vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0],
    //             vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
    //             vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //             vec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
    //             vec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
    //             vec![0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //             vec![0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0],
    //             vec![0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0],
    //             vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
    //             vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0],
    //         ],
    //         LifeMode::Galaxy => vec![
    //             vec![0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
    //             vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
    //             vec![0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0],
    //             vec![1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0],
    //             vec![1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0],
    //             vec![0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0],
    //             vec![0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1],
    //             vec![0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1],
    //             vec![0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0],
    //             vec![0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    //             vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
    //         ],
    //         _ => Vec::new(),
    //     }
    // }

    // fn toggle_simulation(&mut self) {
    //     self.simulate = match self.simulate {
    //         true => false,
    //         false => true,
    //     };
    //
    //     println!("Toggled simulation to {}", self.simulate);
    // }
}

fn main() {
    let mut world = World::new();
    // let mut world = World::create();

    let mut buff = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "CA",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X4,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // let left: Vec2<f32> = Vec2::left();
    // println!("{:?}", left);

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600 * 5)));

    while window.is_open() && !window.is_key_down(Key::Q) {
        // TODO: implement liquid spawning
        if window.get_mouse_down(MouseButton::Left) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, y)| {
                println!("-> {} {}", x, y);
            });
        }

        // Growing ground
        if window.get_mouse_down(MouseButton::Right) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, y)| {
                println!("this!!");
                world.place_figure(Vec2::new(
                    x.floor() as u32,
                    y.floor() as u32,
                ));
            });
        }

        world.tick();
        world.render(&mut buff);

        window
            .update_with_buffer(&buff, WIDTH, HEIGHT)
            .unwrap();
    }
}