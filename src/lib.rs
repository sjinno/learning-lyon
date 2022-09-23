use std::iter;

use lyon::lyon_tessellation::{BuffersBuilder, FillVertex, StrokeOptions, StrokeVertex};
use lyon::math::{point, Box2D, Point};
use lyon::path::Path;
use lyon::path::{builder::BorderRadii, Winding};
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::{FillOptions, FillTessellator, StrokeTessellator, VertexBuffers};

// use lyon::geom::{CubicBezierSegment, Point};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use log::{debug, info, Level};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const VERTICES: &[Vertex] = &[
    // Vertex {
    //     position: [0.0, 0.5, 0.0],
    //     color: [1.0, 0.0, 0.0],
    // },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(module = "../defined-in-js.js"))]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
extern "C" {
    fn hello_world() -> String;
    fn get_window_width() -> u32;
    fn get_window_height() -> u32;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_log::init_with_level(Level::Debug);
        } else {
            env_logger::init();
            println!("Hello, world!");
        }
    }

    debug!("It works!");
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
fn create_canvas(window: &Window, width: u32, height: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(width / 2, height / 2));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        // let cb_curve = CubicBezierSegment {
        //     from: Point {
        //         x: VERTICES[0].position[0],
        //         y: VERTICES[0].position[1],
        //         ..Default::default()
        //     },
        //     ctrl1: Point {
        //         x: VERTICES[0].position[0] + 0.3,
        //         y: VERTICES[0].position[1] - 0.5,
        //         ..Default::default()
        //     },
        //     ctrl2: Point {
        //         x: VERTICES[0].position[0] + 0.4,
        //         y: VERTICES[0].position[1] + 0.8,
        //         ..Default::default()
        //     },
        //     to: Point {
        //         x: VERTICES[1].position[0],
        //         y: VERTICES[1].position[1],
        //         ..Default::default()
        //     },
        // };

        // let flattened = cb_curve.flattened(0.002);
        // // while let Some(p) = flattened.next() {
        // //     eprintln!("{:?}", p);
        // // }

        // let vertices = flattened
        //     .into_iter()
        //     .map(|p| Vertex {
        //         position: [p.x, p.y, 0.0],
        //         color: [1.0, 0.0, 0.0],
        //     })
        //     .collect::<Vec<Vertex>>();

        // // Example 2
        // let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        // let mut geometry_builder = simple_builder(&mut geometry);
        // let options = FillOptions::tolerance(0.1);
        // let mut tessellator = FillTessellator::new();

        // let mut builder = tessellator.builder(&options, &mut geometry_builder);

        // builder.add_rounded_rectangle(
        //     &Box2D {
        //         min: point(0.0, 0.0),
        //         max: point(100.0, 1.0),
        //     },
        //     &BorderRadii {
        //         top_left: 0.0,
        //         top_right: 0.0,
        //         bottom_left: 0.0,
        //         bottom_right: 0.0,
        //     },
        //     Winding::Positive,
        // );

        // let _ = builder.build();

        // // The tessellated geometry is ready to be uploaded to the GPU.
        // println!(
        //     " -- {} vertices {} indices",
        //     geometry.vertices.len(),
        //     geometry.indices.len()
        // );

        // // Create a simple path.
        // let mut path_builder = Path::builder();
        // path_builder.begin(point(0.0, 0.0));
        // path_builder.line_to(point(1.0, 2.0));
        // path_builder.line_to(point(2.0, 0.0));
        // path_builder.line_to(point(1.0, 1.0));
        // path_builder.end(true);
        // let path = path_builder.build();

        // // Create the destination vertex and index buffers.
        // let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

        // {
        //     // Create the destination vertex and index buffers.
        //     let mut vertex_builder = simple_builder(&mut buffers);

        //     // Create the tessellator.
        //     let mut tessellator = StrokeTessellator::new();

        //     // Compute the tessellation.
        //     let _ = tessellator.tessellate(&path, &StrokeOptions::default(), &mut vertex_builder);
        // }

        // println!("The generated vertices are: {:?}.", &buffers.vertices[..]);
        // println!("The generated indices are: {:?}.", &buffers.indices[..]);

        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut fill_tess = FillTessellator::new();

        // Build a Path for the arrow.
        let mut builder = Path::builder();
        builder.begin(point(-1.0, -0.2));
        builder.line_to(point(0.5, -0.2));
        builder.line_to(point(0.5, -0.7));
        builder.line_to(point(1.5, 0.0));
        builder.line_to(point(0.5, 0.7));
        builder.line_to(point(0.5, 0.2));
        builder.line_to(point(-1.0, 0.2));
        builder.close();
        let path = builder.build();

        // // // Bézierish curve... Nope.
        // let mut builder = Path::builder();
        // builder.begin(point(0.0, 0.0));
        // builder.cubic_bezier_to(point(0.5, -0.8), point(1.0, -1.3), point(2.0, 1.0));
        // builder.line_to(point(2.0, 1.1));
        // builder.cubic_bezier_to(point(1.0, -1.2), point(0.5, -0.7), point(0.0, 0.1));
        // builder.close();
        // let path = builder.build();

        {
            fill_tess
                .tessellate_path(
                    &path,
                    &FillOptions::tolerance(0.01),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                        position: [vertex.position().x / 5.0, vertex.position().y / 5.0, 0.0],
                        color: [1.0, 1.0, 0.0],
                    }),
                )
                .unwrap();
            // let mut stroke_tess = StrokeTessellator::new();
            // let _ = stroke_tess.tessellate(
            //     &path,
            //     &StrokeOptions::default(),
            //     &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| Vertex {
            //         position: [vertex.position().x / 5.0, vertex.position().y / 5.0, 0.0],
            //         color: [1.0, 1.0, 0.0],
            //     }),
            // );
        }

        // let vertices = geometry
        //     .vertices
        //     .into_iter()
        //     .map(|p| Vertex {
        //         position: [p.x / 3.0, p.y / 3.0, 0.0],
        //         color: [1.0, 1.0, 0.0],
        //     })
        //     .collect::<Vec<Vertex>>();
        let vertices = geometry.vertices;
        eprintln!("VERTICES {vertices:#?}");

        let indices = geometry.indices;
        eprintln!("INDICES {indices:?}");

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &&shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip, // 1.
                strip_index_format: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default() // front_face: wgpu::FrontFace::Ccw, // 2.
                                     // cull_mode: Some(wgpu::Face::Back),
                                     // // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                                     // polygon_mode: wgpu::PolygonMode::Fill,
                                     // // Requires Features::DEPTH_CLIP_CONTROL
                                     // unclipped_depth: false,
                                     // // Requires Features::CONSERVATIVE_RASTERIZATION
                                     // conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = vertices.len() as u32;

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // render_pass.set_pipeline(&self.render_pipeline);
            // // render_pass.draw(0..3, 0..1);

            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.draw(0..self.num_vertices, 0..1);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    init_logger();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let hw = unsafe { hello_world() };
            info!("SHOHEI: hw {}", hw);
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let width = unsafe { get_window_width() };
            let height = unsafe { get_window_height() };
            info!("SHOHEI: width; {}", width);
            info!("SHOHEI: height; {}", height);
            create_canvas(&window, width, height);
            info!("Canvas successfully created!");
        }
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            info!("SHOHEI: resizing...");
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    })
}
