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

}

struct World {
    water: [i32; WIDTH],
    energy: [i32; WIDTH],
    ground: [i32; WIDTH],
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            energy: [0; WIDTH],
            water: [0; WIDTH],
            ground: [10; WIDTH],
        };

        this
    }

    fn tick(&mut self) {
        let mut dwater = [0; WIDTH];
        let mut denergy = [0; WIDTH];

        let ground = self.ground;
        let mut water = self.water;
        let mut energy = self.energy;

        for x in 1..WIDTH - 1 {
            // left force
            if ground[x] + water[x] - energy[x] > ground[x - 1] + water[x - 1] + energy[x - 1] {
                let flow = water[x].min(ground[x] + water[x] - energy[x] - ground[x - 1] - water[x - 1] - energy[x - 1]) / 4;
                dwater[x - 1] += flow;
                dwater[x] -= flow;
                denergy[x - 1] += -energy[x - 1] / 2 - flow;
            }

            // right force
            if ground[x] + water[x] + energy[x] > ground[x + 1] + water[x + 1] - energy[x + 1] {
                let flow = water[x].min(ground[x] + water[x] + energy[x] - ground[x + 1] - water[x + 1] + energy[x + 1]) / 4;
                dwater[x + 1] += flow;
                dwater[x] -= flow;
                denergy[x + 1] += -energy[x + 1] / 2 + flow;
            }
        }

        for x in 1..WIDTH-1 {
            water[x] += dwater[x];
            energy[x] += denergy[x];
        }
        println!("\n");

        self.water = water;
        self.energy = energy;
    }

    fn spawn_water(&mut self, x: f32) {
        self.water[x as usize] += 5;
    }

    fn spawn_ground(&mut self, x: f32) {
        self.ground[x as usize] += 1;
    }

    fn render_line(&self, buff: &mut [u32], x: usize, mut y0: usize, mut y1: usize, color: Color) {
        let j: usize = 0;
        y0 = HEIGHT - y0;
        y1 = HEIGHT - y1;

        while y0 > y1 {
            y0 -= 1;
            buff[y0 as usize * WIDTH + x as usize] = color.get_hex();
        }
    }

    fn troubleshoor_renderer(&self, buff: &mut [u32]) {
        self.render_line(buff, 10, 0, 10, Color::Red);
        self.render_line(buff, 10, 10, 50, Color::Blue);
    }

    fn render(&self, buff: &mut [u32]) {
        let ground = self.ground.clone();
        let water = self.water.clone();

        for x in 0..WIDTH {
            let ground_y0 = 0;
            let ground_y1 = ground[x] as usize;
            let water_y0 = ground_y1;
            let water_y1 = water_y0 + water[x] as usize;

            self.render_line(buff, x, ground_y0, ground_y1, Color::Red);
            self.render_line(buff, x, water_y0, water_y1, Color::Blue);
        }
    }
}

fn main() {
    let mut world = World::new();

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

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600 * 2)));

    while window.is_open() && !window.is_key_down(Key::Q) {
        if window.get_mouse_down(MouseButton::Left) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, _)| {
                world.spawn_water(x);
            });
        }

        if window.get_mouse_down(MouseButton::Right) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, _)| {
                world.spawn_ground(x);
            });
        }

        world.tick();
        world.render(&mut buff);

        window
            .update_with_buffer(&buff, WIDTH, HEIGHT)
            .unwrap();
    }
}