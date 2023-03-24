extern crate glium;

use glium::{glutin, Surface};
use glium::glutin::event::*;
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::glutin::event_loop::*;
use std::time::Instant;
use glium::glutin::dpi::LogicalSize;
use nalgebra::{Point3, Vector3, Matrix, U4, ArrayStorage, Matrix4};
use glium::*;
use num_format::{Locale, ToFormattedString};
use std::io::{stdout, Write};

const VERTEX_SHADER_SRC: &str = r#"
    #version 330

    in vec3 position;
    in vec3 color;
    out vec3 v_color;

    uniform mat4 projection;
    uniform mat4 view;
    uniform mat4 model;

    void main() {
        gl_Position = projection * view * model * vec4(position, 1.0);
        v_color = color;
    }
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 330

    in vec3 v_color;
    out vec4 color;

    void main() {
        color = vec4(v_color, 1.0);
    }
"#;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f64; 3],
    color: [f64; 3],
}

implement_vertex!(Vertex, position, color);

struct Cube {
    vertices: Vec<Vertex>,
    // indices: [u32; 36],
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u32>,
}

impl Cube {
    fn new(display: &Display) -> Self {
        let vertices = vec![
            // Front face
            Vertex { position: [-0.5, -0.5, 0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], color: [1.0, 1.0, 0.0] },
            // Back face
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], color: [0.0, 0.0, 0.0] },
        ];

        let indices: [u32; 36] = [
            0, 1, 2, 2, 3, 0, // Front face
            4, 5, 6, 6, 7, 4, // Back face
            7, 6, 2, 2, 3, 7, // Top face
            4, 5, 1, 1, 0, 4, // Bottom face
            5, 6, 2, 2, 1, 5, // Right face
            7, 4, 0, 0, 3, 7, // Left face
        ];

        let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
        let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();
        
        Self {
            vertices,
            // indices,
            vertex_buffer,
            index_buffer,
        }
    }

    fn draw(&self, target: &mut glium::Frame, program: &glium::Program, model_matrix: &nalgebra::Matrix4<f32>) {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let fov: f32 = 3.141592 / 3.0;
        let zfar = 1024.0;
        let znear = 0.1;

        let projection_matrix: nalgebra::Matrix4<f32> = nalgebra::Perspective3::new(aspect_ratio, fov, znear, zfar).into();
        let view_matrix = nalgebra::Matrix4::look_at_rh(
            &Point3::new(0.0, 0.0, 2.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );

        let uniforms = uniform! {
            projection: Into::<[[f32; 4]; 4]>::into(projection_matrix),
            view: Into::<[[f32; 4]; 4]>::into(view_matrix),
            model: Into::<[[f32; 4]; 4]>::into(*model_matrix),
        };

        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }

    fn get_center(&self) -> [f64; 2] {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
    
        for vertex in self.vertices.iter() {
            if vertex.position[0] < min_x {
                min_x = vertex.position[0];
            }
            if vertex.position[0] > max_x {
                max_x = vertex.position[0];
            }
            if vertex.position[1] < min_y {
                min_y = vertex.position[1];
            }
            if vertex.position[1] > max_y {
                max_y = vertex.position[1];
            }
        }
    
        let center = [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0];

        return center
    }

    // fn get_vertices(&self) -> &Vec<Vertex> {
    //     &self.vertices
    // }
}

#[allow(unused_assignments)]
fn main() {
    let events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rotating Cube")
        .with_inner_size(LogicalSize { width: 800, height: 600 });
    let args = std::env::args().collect::<Vec<String>>();
    
    let mut samples = 0;
    if args.len() > 1 {
        samples = args[1].parse().unwrap_or(0);
    }
    print!("running {}x AA", samples);
    
    let context = glutin::ContextBuilder::new().with_vsync(false).with_multisampling(samples);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let program =
        glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
            .unwrap();

    let cube = Cube::new(&display);
    let model_matrix = nalgebra::Matrix4::identity();
    let mut is_dragging = false;

    let start_time = Instant::now();

    let width: u32 = 800;
    let height: u32 = 600;

    let mut multiplier = 1.0;
    if args.len() > 2 {
        multiplier = args[2].parse::<f32>().unwrap_or(1.0);
    }
    println!(", at {}x speed.\n", multiplier);

    fn new_draw(multiplier: f32, start_time: Instant, mut model_matrix: Matrix<f32, U4, U4, ArrayStorage<f32, U4, U4>>, display: &Display, cube: &Cube, program: &Program) {
        let elapsed_time = start_time.elapsed().as_secs_f32() * multiplier;
        model_matrix = Matrix4::new_rotation(Vector3::new(elapsed_time, elapsed_time, elapsed_time));

        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.clear_depth(1.0);

        cube.draw(&mut target, &program, &model_matrix);

        target.finish().unwrap();
    }

    let mut frames = 0;
    let mut lock = stdout().lock();

    let mut x = 0f64;
    let mut y = 0f64;

    let mut new_vertices = Vec::new();

    events_loop.run(move |event, _, control_flow| {
        new_draw(multiplier, start_time, model_matrix, &display, &cube, &program);
        frames += 1;

        *control_flow = match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => ControlFlow::Exit,
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                    is_dragging = true;
                    ControlFlow::Poll
                }
                WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                    is_dragging = false;
                    ControlFlow::Poll
                }
                WindowEvent::CursorMoved { position, .. } => {
                    (x, y) = (position.x, position.y);

                    if is_dragging {
                        let new_center = [x / width as f64, y / height as f64, 0.0];

                        // Calculate the offset from the old center to the new center
                        let center = cube.get_center();
                    
                        // Modify the vertex positions relative to the new center position
                        let translation = [new_center[0] - center[0], new_center[1] - center[1], new_center[2] - center[2]];
                    
                        let mut new_vertices = Vec::new();
                        for vertex in cube.vertices.iter() {
                            let new_position = [vertex.position[0] + translation[0], vertex.position[1] + translation[1], vertex.position[2] + translation[2]];
                            new_vertices.push(Vertex { position: new_position, color: vertex.color });
                        }
                    
                        // Create the vertex buffer object (VBO) for the modified vertices
                        cube.vertex_buffer.write(&new_vertices);
                    }

                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll
            },
            _ => { 
                let chars_to_write = format!("\rframe: {}, mouse: ({}, {}), dragging? {}, vertices: {:?}", 
                    frames.to_formatted_string(&Locale::en), x.round(), y.round(), is_dragging, modified_vertices).to_string();

                let cols = termsize::get().unwrap().cols as usize;
                
                let max_chars = {
                    if chars_to_write.len() > cols.into() {
                        cols
                    }
                    else {
                        chars_to_write.len()
                    }
                };

                let padding = " ".repeat(cols - max_chars);

                write!(lock, "{}{}", &chars_to_write[..max_chars], padding).unwrap();

                ControlFlow::Poll
            }
        }
    });
}