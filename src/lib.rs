use std::sync::mpsc::Receiver;
use std::error::Error;
use std::cmp::min;
use std::mem::swap;

use glfw::{Context, WindowHint, WindowEvent, Key, Action};
use glw::shader::ShaderType;
use glw::buffers::StructuredBuffer;
use glw::{Color, RenderTarget, Shader, Uniform, Vec2, MemoryBarrier};
use rand::Rng;
use std::borrow::Borrow;

const WINDOW_WIDTH: u32 = 256;
const WINDOW_HEIGHT: u32 = 256;
const FIELD_WIDTH: i32 = 256;
const FIELD_HEIGHT: i32 = 256;

#[derive(Copy, Clone)]
enum CellType {
    Empty = 0,
    Block = 1,
    Water = 2,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Empty
    }
}

// Struct for structured buffers
// #[allow(dead_code)]
#[derive(Default, Copy, Clone)]
struct Data {
    alive: bool,
    lifetime: f32,
    t: f32,
}

#[derive(Default, Copy, Clone)]
struct Cell {
    element_type: i32,
    mass: f32,
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
    // curr_sb: StructuredBuffer<Data>,
    // prev_sb: StructuredBuffer<Data>,

    compute_program: glw::GraphicsPipeline,
    // compute_program_2: glw::GraphicsPipeline,
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

        #[cfg(debug_assertions)]
            ctx.set_debug();

        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);

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

        // let compute_program_2 = {
        //     let mut c_shader_update = Shader::new(ShaderType::Compute);
        //     c_shader_update.load_from_file("shaders/water_update.shader").unwrap();
        //
        //     glw::PipelineBuilder::new()
        //         .with_compute_shader(c_shader_update)
        //         .build()
        // };

        let field_size = Vec2::<i32> {
            x: FIELD_WIDTH,
            y: FIELD_HEIGHT,
        };

        let image_data = Application::generate_map(&field_size);
        // let image_data = Application::generate_field(&field_size);

        let prev_sb = StructuredBuffer::from(image_data);
        let curr_sb = StructuredBuffer::new((field_size.x * field_size.y) as usize);

        Ok(Application {
            glfw,
            window,
            events,
            field_size,
            curr_sb,
            prev_sb,
            compute_program,
            // compute_program_2,
            render_program,
            quad,
            is_paused: false,
            gl_ctx: ctx,
        })
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.glfw.set_swap_interval(glfw::SwapInterval::None);

        // let update_time = 1.0 / 200.0;
        let update_time = 1.0 / 40.0;
        // let update_time = 1.0 / 5.0;

        let mut timer = 0.0;
        let mut time = self.get_time();

        let mut drawing_cell = 0;
        let mut drawing_type = CellType::Water as i32;
        let mut drawing_x = 0;
        let mut drawing_y = 0;

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
                    WindowEvent::Key(Key::Q, _, Action::Press, _) => self.window.set_should_close(true),
                    WindowEvent::Key(Key::P, _, Action::Press, _) => self.is_paused = !self.is_paused,
                    WindowEvent::Key(Key::Num1, _, Action::Press, _) => drawing_type = CellType::Block as i32,
                    WindowEvent::Key(Key::Num2, _, Action::Press, _) => drawing_type = CellType::Water as i32,
                    WindowEvent::Key(Key::R, _, Action::Press, _) => self.prev_sb.map_data(&Application::generate_cave(&self.field_size)),
                    // WindowEvent::Key(Key::R, _, Action::Press, _) => self.prev_sb.map_data(&Application::generate_field(&self.field_size)),
                    WindowEvent::MouseButton(btn, action, mods) => {
                        match action {
                            glfw::Action::Press => drawing_cell = 1,
                            glfw::Action::Release => drawing_cell = 0,
                            _ => {}
                        }

                        println!("Button: {:?}, Action: {:?}, Modifiers: [{:?}]", glfw::DebugAliases(btn), action, mods);
                    }
                    WindowEvent::CursorPos(xpos, ypos) => {
                        if drawing_cell == 1 {
                            drawing_x = xpos as i32;
                            drawing_y = ypos as i32;
                        }
                    }
                    _ => {}
                }
            }

            self.gl_ctx.bind_rt(&RenderTarget::default());
            self.gl_ctx.clear(Some(Color::new(0, 0, 0, 0)));

            if !self.is_paused && timer <= 0.0 {
                timer = update_time;

                self.gl_ctx.bind_pipeline(&self.compute_program);

                self.compute_program.set_uniform("u_field_size", Uniform::Vec2(self.field_size.x as f32, self.field_size.y as f32));
                self.compute_program.set_uniform("u_dt", Uniform::Float(update_time as f32));
                self.compute_program.set_uniform("u_time", Uniform::Float(self.get_time() as f32));
                self.compute_program.set_uniform("u_drawing", Uniform::Int(drawing_cell));
                self.compute_program.set_uniform("u_drawing_type", Uniform::Int(drawing_type));

                self.compute_program.bind_storage_buffer(self.curr_sb.get_id(), 1);
                self.compute_program.bind_storage_buffer(self.prev_sb.get_id(), 0);

                if drawing_cell == 1 {
                    self.compute_program.set_uniform("u_drawing_coords", Uniform::Vec2(drawing_x as f32, drawing_y as f32));
                }

                self.gl_ctx.dispatch_compute(
                    self.field_size.x as u32 / 8,
                    self.field_size.y as u32 / 8,
                    1,
                );

                // FENCE and sync

                self.gl_ctx.memory_barrier(MemoryBarrier::ShaderStorage);

                swap(&mut self.curr_sb, &mut self.prev_sb);
            }

            self.gl_ctx.bind_pipeline(&self.render_program);
            self.render_program.set_uniform("u_field_size", Uniform::Vec2(self.field_size.x as f32, self.field_size.y as f32));
            self.render_program.set_uniform("u_time", Uniform::Float(self.get_time() as f32));
            self.render_program.bind_storage_buffer(self.prev_sb.get_id(), 0);

            self.quad.draw();

            self.window.swap_buffers();

            println!("{}", self.prev_sb.get_id());
        }

        Ok(())
    }

    fn generate_map(field_size: &Vec2<i32>) -> Vec<Cell> {
        let mut image = Vec::new();

        for _ in 0..field_size.x * field_size.y {
            image.push(Cell {
                element_type: CellType::Empty as i32,
                mass: 0.0,
            });
        }

        for i in 0..256 {
            image[i as usize + (128 * field_size.x as usize)] = Cell {
                element_type: CellType::Block as i32,
                mass: 0.0,
            };
        }

        image[128 + 192 * 256] = Cell { element_type: CellType::Water as i32, mass: 0.0 };
        image[129 + 190 * 256] = Cell { element_type: CellType::Water as i32, mass: 0.0 };
        image[130 + 193 * 256] = Cell { element_type: CellType::Water as i32, mass: 0.0 };
        image[131 + 191 * 256] = Cell { element_type: CellType::Water as i32, mass: 0.0 };
        image[132 + 192 * 256] = Cell { element_type: CellType::Water as i32, mass: 0.0 };

        image
    }

    fn generate_cave(field_size: &Vec2<i32>) -> Vec<Cell> {
        let mut rng = rand::thread_rng();
        let mut image = Vec::new();

        for _ in 0..field_size.x * field_size.y {
            if rng.gen::<f32>() < 0.2 {
                image.push(Cell {
                    element_type: CellType::Block as i32,
                    mass: 0.0,
                })
            } else {
                image.push(Cell {
                    element_type: CellType::Empty as i32,
                    mass: 0.0,
                })
            }
        }

        image
    }

    fn generate_field(field_size: &Vec2<i32>) -> Vec<Data> {
        let mut rng = rand::thread_rng();
        let mut image = Vec::new();

        for _ in 0..field_size.x * field_size.y {
            if rng.gen::<f32>() > 0.1 {
                image.push(Data {
                    alive: false,
                    lifetime: 0.0,
                    t: 0.0,
                });
            } else {
                image.push(Data {
                    alive: true,
                    lifetime: 1.0,
                    t: 0.0,
                });
            }
        }

        image
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
