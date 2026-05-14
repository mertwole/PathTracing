use __core::time::Duration;
use clap::Parser;
use image::{Rgb32FImage, RgbaImage};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use std::{sync::Arc, sync::Mutex, time::Instant};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use worker::api::render_task::RenderTaskUninit;

mod scene;
mod worker_connection;

use scene::Scene;

#[derive(Parser)]
pub struct Cli {
    #[clap(long)]
    mongodb_url: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let render_task_path = "./scene_data/render_task.json";
    let render_task_data = std::fs::read(render_task_path).unwrap();
    let render_task_data = String::from_utf8(render_task_data).unwrap();
    let render_task: RenderTaskUninit = serde_json::de::from_str(&render_task_data).unwrap();

    let scene = Scene::load(&render_task.scene);

    let render_task = render_task.init(scene.md5.clone());

    scene.upload_to_mongodb(&args.mongodb_url).await;

    let image = worker_connection::get_image(render_task).await;
    let image = gamma_correction(image);

    let main_window = MainWindow::init(image).await;
    main_window.enter_render_loop();
}

fn gamma_correction(image: Rgb32FImage) -> RgbaImage {
    let mut gamma_corrected_image = RgbaImage::new(image.width(), image.height());
    for x in 0..image.width() {
        for y in 0..image.height() {
            let res_pixel = image.get_pixel(x, y);
            let r = (res_pixel.0[0] / (1.0 + res_pixel.0[0]) * 255.0) as u8;
            let g = (res_pixel.0[1] / (1.0 + res_pixel.0[1]) * 255.0) as u8;
            let b = (res_pixel.0[2] / (1.0 + res_pixel.0[2]) * 255.0) as u8;
            let gc_pixel = gamma_corrected_image.get_pixel_mut(x, y);
            gc_pixel.0[0] = r;
            gc_pixel.0[1] = g;
            gc_pixel.0[2] = b;
            gc_pixel.0[3] = 255;
        }
    }

    gamma_corrected_image
}

struct Framebuffer {
    back: Arc<Mutex<Option<RgbaImage>>>,
    front: Option<TextureId>,
}

impl Framebuffer {
    fn new() -> Framebuffer {
        Framebuffer {
            back: Arc::from(Mutex::from(None)),
            front: None,
        }
    }

    fn set_image(&self, image: RgbaImage) {
        let mut back = self.back.lock().unwrap();
        back.replace(image);
    }

    fn swap_buffers(
        &mut self,
        renderer: &mut Renderer,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Option<TextureId> {
        self.front = match self.back.try_lock() {
            Ok(ref mut guard) => {
                if let Some(back_image) = guard.as_mut() {
                    if let Some(front) = self.front {
                        let texture = renderer.textures.get(front).unwrap();
                        if texture.width() != back_image.width()
                            || texture.height() != back_image.height()
                        {
                            // TODO: delete 'texture'
                            renderer.textures.remove(front);

                            let texture_config = imgui_wgpu::TextureConfig {
                                size: wgpu::Extent3d {
                                    width: back_image.width(),
                                    height: back_image.height(),
                                    ..Default::default()
                                },
                                label: None,
                                format: Some(wgpu::TextureFormat::Rgba8Unorm),
                                ..Default::default()
                            };

                            let new_texture =
                                imgui_wgpu::Texture::new(device, renderer, texture_config);
                            new_texture.write(
                                queue,
                                back_image.as_raw(),
                                back_image.width(),
                                back_image.height(),
                            );
                            Some(renderer.textures.insert(new_texture))
                        } else {
                            texture.write(
                                queue,
                                back_image.as_raw(),
                                back_image.width(),
                                back_image.height(),
                            );
                            self.front
                        }
                    } else {
                        let texture_config = imgui_wgpu::TextureConfig {
                            size: wgpu::Extent3d {
                                width: back_image.width(),
                                height: back_image.height(),
                                ..Default::default()
                            },
                            label: None,
                            format: Some(wgpu::TextureFormat::Rgba8Unorm),
                            ..Default::default()
                        };

                        let new_texture =
                            imgui_wgpu::Texture::new(device, renderer, texture_config);
                        new_texture.write(
                            queue,
                            back_image.as_raw(),
                            back_image.width(),
                            back_image.height(),
                        );
                        Some(renderer.textures.insert(new_texture))
                    }
                } else {
                    self.front
                }
            }
            Err(_) => self.front,
        };

        self.front
    }
}

struct MainWindow {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    event_loop: Option<EventLoop<()>>,
    renderer: Renderer,
    window: Window,
    imgui: imgui::Context,
    winit_platform: WinitPlatform,

    image: RgbaImage,

    framebuffer: Framebuffer,
}

impl MainWindow {
    pub async fn init(image: RgbaImage) -> MainWindow {
        let event_loop = EventLoop::new();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let (window, size, surface) = {
            let window = Window::new(&event_loop).unwrap();
            window.set_inner_size(LogicalSize {
                width: 1280.0,
                height: 720.0,
            });
            window.set_title("imgui-wgpu");
            let size = window.inner_size();

            let surface = unsafe { instance.create_surface(&window) }.unwrap();

            (window, size, surface)
        };

        let hidpi_factor = window.scale_factor();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        };

        surface.configure(&device, &surface_desc);

        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: surface_desc.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        MainWindow {
            device,
            queue,
            surface,
            event_loop: Some(event_loop),
            renderer,
            window,
            imgui,
            winit_platform: platform,

            image,

            framebuffer: Framebuffer::new(),
        }
    }

    pub fn enter_render_loop(mut self) {
        let clear_color = wgpu::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        let mut last_frame = Instant::now();
        let mut last_cursor = None;
        let mut from_last_tick = Duration::ZERO;

        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    let size = self.window.inner_size();

                    let surface_desc = wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Fifo,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
                    };

                    self.surface.configure(&self.device, &surface_desc);
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::RedrawEventsCleared => {
                    let now = Instant::now();
                    from_last_tick += now - last_frame;
                    self.imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;

                    let frame = match self.surface.get_current_texture() {
                        Ok(frame) => frame,
                        Err(e) => {
                            eprintln!("dropped frame: {e:?}");
                            return;
                        }
                    };
                    self.winit_platform
                        .prepare_frame(self.imgui.io_mut(), &self.window)
                        .expect("Failed to prepare frame");
                    let ui = self.imgui.frame();

                    if from_last_tick.as_secs_f32() > 5.0 {
                        from_last_tick = Duration::ZERO;
                        self.framebuffer.set_image(self.image.clone());
                    }

                    let render_texture = self.framebuffer.swap_buffers(
                        &mut self.renderer,
                        &self.queue,
                        &self.device,
                    );

                    {
                        let window = ui.window("render");
                        window
                            .size([512.0, 512.0], Condition::FirstUseEver)
                            .build(|| {
                                if let Some(render_texture) = render_texture {
                                    Image::new(render_texture, [512.0, 512.0]).build(ui);
                                }
                            });
                    }

                    let mut encoder: wgpu::CommandEncoder = self
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    if last_cursor != Some(ui.mouse_cursor()) {
                        last_cursor = Some(ui.mouse_cursor());
                        self.winit_platform.prepare_render(ui, &self.window);
                    }

                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(clear_color),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    self.renderer
                        .render(self.imgui.render(), &self.queue, &self.device, &mut rpass)
                        .expect("Rendering failed");

                    drop(rpass);

                    self.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                _ => (),
            }

            self.winit_platform
                .handle_event(self.imgui.io_mut(), &self.window, &event);
        });
    }
}
