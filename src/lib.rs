use std::sync::mpsc::{Receiver, channel};
use std::error::Error;
use std::cmp::min;
use std::mem::swap;

use glfw::{Context, WindowHint, WindowEvent, Key, Action, CursorMode};
use glw::shader::ShaderType;
use glw::buffers::StructuredBuffer;
use glw::{Color, RenderTarget, Shader, Uniform, Vec2, MemoryBarrier};
use rand::Rng;
use std::borrow::Borrow;

const WINDOW_WIDTH: u32 = 512;
const WINDOW_HEIGHT: u32 = 512;
const FIELD_WIDTH: i32 = 512;
const FIELD_HEIGHT: i32 = 512;
const WIDTH: usize = FIELD_WIDTH as usize;
const HEIGHT: usize = FIELD_HEIGHT as usize;

#[derive(Copy, Clone)]
#[repr(i32)]
enum CellType {
    Empty = 0,
    Block = 1,
    Water = 2,
    Acid = 3,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Empty
    }
}

#[derive(Copy, Clone)]
struct Cell {
    element_type: i32,
    mass: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            element_type: CellType::Empty as i32,
            mass: 0.0,
        }
    }
}

fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else {
        if val > max {
            max
        } else {
            val
        }
    }
}

struct Application {
    // GLFW Setup
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,

    // Configurations
    field_size: Vec2<i32>,

    // 2 Structured buffers needed to store the data for the computed shaders
    curr_sb: StructuredBuffer<Cell>,
    prev_sb: StructuredBuffer<Cell>,
    tmp_sb: StructuredBuffer<f32>,

    compute_program: glw::GraphicsPipeline,
    render_program: glw::GraphicsPipeline,

    // Quad mesh
    quad: glw::Mesh,

    // Program states
    is_paused: bool,
    gl_ctx: glw::GLContext,
}

impl Application {
    fn new() -> Result<Application, Box<dyn Error>> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        glfw.window_hint(WindowHint::Resizable(false));
        glfw.window_hint(WindowHint::ContextVersion(4, 5));
        glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        let (mut window, events) = glfw.create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "Simulation",
            glfw::WindowMode::Windowed,
        ).unwrap();

        // Settup up the OpenGL context
        let mut ctx = glw::GLContext::new(&mut window);

        window.set_cursor_mode(CursorMode::Hidden);
        window.make_current();

        // window.set_all_polling(true);
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_scroll_polling(true);

        window.show();

        let vertices: [f32; 32] = [
            -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
            1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0,
            1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
            -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        ];

        let indices: [i32; 6] = [
            0, 1, 2,
            0, 2, 3,
        ];

        let quad = glw::MeshBuilder::new()
            .with_vertex_data(&vertices)
            .with_index_data(&indices)
            .build();

        let render_program = {
            let mut v_shader = Shader::new(ShaderType::Vertex);
            let mut f_shader = Shader::new(ShaderType::Fragment);

            v_shader.load_from_file("shaders/passthrough.vert").unwrap();
            f_shader.load_from_file("shaders/composition.frag").unwrap();

            glw::PipelineBuilder::new()
                .with_vertex_shader(v_shader)
                .with_fragment_shader(f_shader)
                .build()
        };

        let compute_program = {
            let mut c_shader = Shader::new(ShaderType::Compute);

            c_shader.load_from_file("shaders/compute.shader").unwrap();

            glw::PipelineBuilder::new()
                .with_compute_shader(c_shader)
                .build()
        };

        let field_size = Vec2::<i32> {
            x: FIELD_WIDTH,
            y: FIELD_HEIGHT,
        };

        let image_data = Application::generate_map(&field_size);

        let prev_sb = StructuredBuffer::from(*image_data);
        let curr_sb = StructuredBuffer::new((field_size.x * field_size.y) as usize);

        let mut tmp_vec = Box::new(vec![0.0f32; (field_size.x * field_size.y) as usize]);

        tmp_vec[129 + 190 * 256] = 1.0;
        tmp_vec[129 + 200 * 256] = 1.0;
        tmp_vec[129 + 210 * 256] = 1.0;
        tmp_vec[129 + 180 * 256] = 1.0;
        tmp_vec[129 + 170 * 256] = 1.0;
        tmp_vec[129 + 160 * 256] = 1.0;
        tmp_vec[129 + 120 * 256] = 1.0;
        tmp_vec[132 + 190 * 256] = 1.0;
        tmp_vec[126 + 196 * 256] = 1.0;

        let tmp_sb = StructuredBuffer::from(*tmp_vec);

        Ok(Application {
            glfw,
            window,
            events,
            field_size,
            curr_sb,
            prev_sb,
            tmp_sb,
            compute_program,
            render_program,
            quad,
            is_paused: false,
            gl_ctx: ctx,
        })
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.glfw.set_swap_interval(glfw::SwapInterval::None);

        // let update_time = 1.0 / 200.0;
        let update_time = 0.0;

        let mut timer = 0.0;
        let mut time = self.get_time();

        let mut drawing_cell = 0;
        let mut drawing_type = CellType::Water as i32;
        let mut mouse_x = 0.0;
        let mut mouse_y = 0.0;
        let mut brush_size = 1.0;
        let mut rotation_signal = 0;

        while !self.window.should_close() {
            let (width, height) = self.window.get_size();
            let middle = width / 2;
            let width = width.min(height);

            self.gl_ctx.set_viewport(middle - width / 2, 0, width, width);

            let prev_time = time;
            time = self.get_time();

            let dt = time - prev_time;
            timer -= dt;

            self.glfw.poll_events();


            for (_, event) in glfw::flush_messages(&self.events) {
                match event {
                    WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.window.set_should_close(true),
                    WindowEvent::Key(Key::P, _, Action::Press, _) => self.is_paused = !self.is_paused,
                    WindowEvent::Key(Key::C, _, Action::Press, _) => {
                        self.prev_sb.map_data(&Application::get_empty_field(&self.field_size));
                        self.tmp_sb.map_data(&vec![0.0f32; (self.field_size.x * self.field_size.y) as usize]);
                    }
                    WindowEvent::Key(Key::N, _, Action::Press, _) => {
                        self.prev_sb.map_data(&Application::generate_cave());
                        self.tmp_sb.map_data(&vec![0.0f32; (self.field_size.x * self.field_size.y) as usize]);
                    }
                    WindowEvent::Key(Key::R, _, Action::Press, _) => {
                        rotation_signal = 1;
                    }
                    WindowEvent::Key(Key::Num1, _, Action::Press, _) => drawing_type = CellType::Block as i32,
                    WindowEvent::Key(Key::Num2, _, Action::Press, _) => drawing_type = CellType::Water as i32,
                    WindowEvent::Key(Key::Num3, _, Action::Press, _) => drawing_type = CellType::Acid as i32,
                    WindowEvent::MouseButton(btn, action, mods) => {
                        match action {
                            glfw::Action::Press => drawing_cell = 1,
                            glfw::Action::Release => drawing_cell = 0,
                            _ => {}
                        }

                        // println!("Button: {:?}, Action: {:?}, Modifiers: [{:?}]", glfw::DebugAliases(btn), action, mods);
                    }
                    WindowEvent::Scroll(x, y) => {
                        if y != 0.0 {
                            brush_size = clamp(brush_size - y as f32, 1.0, 20.0);
                        }
                    }
                    WindowEvent::CursorPos(xpos, ypos) => {
                        mouse_x = xpos as f32;
                        mouse_y = ypos as f32;
                    }
                    WindowEvent::Key(Key::Space, _, Action::Press, _) => match self.window.get_cursor_mode() {
                        CursorMode::Disabled => self.window.set_cursor_mode(CursorMode::Normal),
                        CursorMode::Normal => self.window.set_cursor_mode(CursorMode::Disabled),
                        _ => {}
                    },
                    _ => {}
                }
            }

            self.gl_ctx.bind_rt(&RenderTarget::default());
            self.gl_ctx.clear(Some(Color::new(0, 0, 0, 0)));

            if !self.is_paused && timer <= 0.0 {
                timer = update_time;

                self.gl_ctx.bind_pipeline(&self.compute_program);

                self.compute_program.set_uniform("u_resolution", Uniform::Vec2(self.field_size.x as f32, self.field_size.y as f32));
                self.compute_program.set_uniform("u_dt", Uniform::Float(update_time as f32));
                self.compute_program.set_uniform("u_time", Uniform::Float(self.get_time() as f32));
                self.compute_program.set_uniform("u_drawing", Uniform::Int(drawing_cell));
                self.compute_program.set_uniform("u_drawing_type", Uniform::Int(drawing_type));
                self.compute_program.set_uniform("u_mouse", Uniform::Vec2(mouse_x, mouse_y));
                self.compute_program.set_uniform("u_brush_size", Uniform::Float(brush_size));
                self.compute_program.set_uniform("u_rotation_signal", Uniform::Int(rotation_signal));

                self.compute_program.bind_storage_buffer(self.prev_sb.get_id(), 0);
                self.compute_program.bind_storage_buffer(self.curr_sb.get_id(), 1);
                self.compute_program.bind_storage_buffer(self.tmp_sb.get_id(), 2);

                self.gl_ctx.dispatch_compute(
                    self.field_size.x as u32 / 8,
                    self.field_size.y as u32 / 8,
                    1,
                );

                // FENCE and sync

                self.gl_ctx.memory_barrier(MemoryBarrier::ShaderStorage);

                swap(&mut self.curr_sb, &mut self.prev_sb);

                rotation_signal = 0;
            }

            self.gl_ctx.bind_pipeline(&self.render_program);
            self.render_program.set_uniform("u_resolution", Uniform::Vec2(self.field_size.x as f32, self.field_size.y as f32));
            self.render_program.set_uniform("u_time", Uniform::Float(self.get_time() as f32));
            self.render_program.set_uniform("u_mouse", Uniform::Vec2(mouse_x, mouse_y));
            self.render_program.set_uniform("u_brush_size", Uniform::Float(brush_size));
            self.render_program.bind_storage_buffer(self.prev_sb.get_id(), 0);
            self.render_program.bind_storage_buffer(self.tmp_sb.get_id(), 2);

            self.quad.draw();

            self.window.swap_buffers();
        }

        Ok(())
    }

    fn generate_map(field_size: &Vec2<i32>) -> Box<Vec<Cell>> {
        let mut grid = Box::new(Vec::new());

        // New empty grid
        for _ in 0..field_size.x * field_size.y {
            grid.push(Cell {
                element_type: CellType::Empty as i32,
                mass: 0.0,
            });
        }

        // Initial Test Case
        for i in 100..190 {
            grid[i as usize + (128 * field_size.x as usize)] = Cell { element_type: CellType::Block as i32, mass: 0.0 };
            grid[i as usize + (129 * field_size.x as usize)] = Cell { element_type: CellType::Block as i32, mass: 0.0 };
            grid[i as usize + (130 * field_size.x as usize)] = Cell { element_type: CellType::Block as i32, mass: 0.0 };
        }

        for i in 0..5 {
            for j in 0..5 {
                if i == j {
                    grid[(128 + i) + (190 + j) * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
                }
            }
        }

        grid[129 + 190 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[129 + 200 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[129 + 210 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[129 + 180 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[129 + 170 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[129 + 160 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[129 + 120 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[132 + 190 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };
        grid[126 + 196 * 256] = Cell { element_type: CellType::Water as i32, mass: 1.0 };

        grid
    }

    fn initialize_cave() -> [[bool; WIDTH]; HEIGHT] {
        let mut cave_map = [[false; WIDTH]; HEIGHT];
        let chance_to_start_alive = 0.38;

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let chance: f64 = rand::thread_rng().gen();
                if chance < chance_to_start_alive {
                    cave_map[i][j] = true;
                }
            }
        }

        cave_map
    }

    fn count_neighbours(map: [[bool; WIDTH]; HEIGHT], x: usize, y: usize) -> i32 {
        let mut count_n = 0;

        for ii in 0..3 {
            for jj in 0..3 {
                let i = ii as i32 - 1;
                let j = jj as i32 - 1;

                if i == 0 && j == 0 {
                    continue;
                }

                let n_x = x as i32 + i;
                let n_y = y as i32 + j;

                if n_x < 0 || n_y < 0 || n_x >= WIDTH as i32 || n_y >= HEIGHT as i32 {
                    count_n += 1;
                } else if map[n_x as usize][n_y as usize] {
                    count_n += 1;
                }
            }
        }

        count_n
    }

    fn do_cave_generation_step(old_map: [[bool; WIDTH]; HEIGHT]) -> [[bool; WIDTH]; HEIGHT] {
        let mut new_map = [[false; WIDTH]; HEIGHT];
        let death_limit = 3;
        let birth_limit = 3;

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let nbs = Application::count_neighbours(old_map, i, j);
                if old_map[i][j] {
                    if nbs < death_limit {
                        new_map[i][j] = false;
                    } else {
                        new_map[i][j] = true;
                    }
                } else {
                    if nbs > birth_limit {
                        new_map[i][j] = true;
                    } else {
                        new_map[i][j] = false;
                    }
                }
            }
        }

        new_map
    }

    fn generate_cave() -> Vec<Cell> {
        let mut grid = vec![Cell::default(); WIDTH * HEIGHT];
        let mut cave_map = Application::initialize_cave();

        for _ in 0..4 {
            cave_map = Application::do_cave_generation_step(cave_map);
        }

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let idx = (i + j * WIDTH as usize);
                if cave_map[i][j] {
                    grid[idx] = Cell {
                        element_type: CellType::Empty as i32,
                        mass: 0.0,
                    };
                } else {
                    grid[idx] = Cell {
                        element_type: CellType::Block as i32,
                        mass: 0.0,
                    };
                }
            }
        }

        grid
    }

    fn get_empty_field(field_size: &Vec2<i32>) -> Vec<Cell> {
        vec![Cell::default(); (field_size.x * field_size.y) as usize]
    }

    fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }
}

pub fn run_simulation() -> Result<(), Box<dyn Error + 'static>> {
    let mut app = Application::new()?;
    app.run()?;

    Ok(())
}
