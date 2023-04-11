use __core::time::Duration;
use clap::Parser;
use image::{Rgba32FImage, RgbaImage};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use std::time::Instant;
use wgpu::{util::DeviceExt, TextureViewDescriptor};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use control_panel::api::{GetRenderResponse, PostRenderTaskRequest};
use worker::api::render_task::{RenderTask, RenderTaskUninit};

mod scene;

use scene::Scene;

#[derive(Parser)]
pub struct Cli {
    #[clap(long, env = "CONTROL_PANEL_URL")]
    pub control_panel_url: String,
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
    let render_task_md5 = render_task.md5();

    scene
        .upload_to_control_panel(&args.control_panel_url, &render_task_md5)
        .await;

    send_render_task(&args.control_panel_url, render_task).await;

    let main_window = MainWindow::init(&args.control_panel_url, &render_task_md5).await;
    main_window.enter_render_loop();
}

async fn send_render_task(control_panel_url: &str, render_task: RenderTask) {
    let client = reqwest::Client::new();

    let body = PostRenderTaskRequest { task: render_task };

    client
        .post(format!("{}/render_tasks", control_panel_url))
        .json(&body)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

async fn save_render(control_panel_url: &str, render_task_md5: &str) {
    let res = reqwest::get(format!(
        "{}/render_tasks/{}/render",
        control_panel_url, render_task_md5
    ))
    .await
    .unwrap()
    .error_for_status()
    .unwrap();
    let res: GetRenderResponse = res.json().await.unwrap();

    if res.image_data.len() == 0 {
        return;
    }

    let image = Rgba32FImage::from_raw(res.image_width, res.image_height, res.image_data).unwrap();

    image
        .save_with_format("./renders/0.exr", image::ImageFormat::OpenExr)
        .unwrap();
}

struct MainWindow {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    event_loop: EventLoop<()>,
    renderer: Renderer,
    window: Window,
    imgui: imgui::Context,
    winit_platform: WinitPlatform,

    control_panel_url: String,
    render_task_md5: String,

    render_texture_id: TextureId,
    render_width: u32,
    render_height: u32,
}

impl MainWindow {
    pub async fn init(control_panel_url: &str, render_task_md5: &str) -> MainWindow {
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
            window.set_title(&format!("imgui-wgpu"));
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

        let mut window = MainWindow {
            device,
            queue,
            surface,
            event_loop,
            renderer,
            window,
            imgui,
            winit_platform: platform,

            control_panel_url: control_panel_url.to_string(),
            render_task_md5: render_task_md5.to_string(),

            render_texture_id: TextureId::new(0),
            render_width: 0,
            render_height: 0,
        };

        (
            window.render_texture_id,
            window.render_width,
            window.render_height,
        ) = window.load_image_to_wgpu();

        window
    }

    fn load_image_to_wgpu(&mut self) -> (TextureId, u32, u32) {
        let image = futures::executor::block_on(fetch_render(
            &self.control_panel_url,
            &self.render_task_md5,
        ));
        let (width, height) = image.dimensions();
        let raw_data = image.into_raw();
        let texture_config = imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width,
                height,
                ..Default::default()
            },
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&self.device, &self.renderer, texture_config);
        texture.write(&self.queue, &raw_data, width, height);

        (self.renderer.textures.insert(texture), width, height)
    }

    fn update_image(
        control_panel_url: &str,
        render_task_md5: &str,
        renderer: &mut Renderer,
        render_texture_id: &mut TextureId,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        let image = futures::executor::block_on(fetch_render(control_panel_url, render_task_md5));
        let (width, height) = image.dimensions();
        let raw_data = image.into_raw();

        let texture = renderer.textures.get(*render_texture_id).unwrap();

        if texture.width() != width || texture.height() != height {
            let texture_config = imgui_wgpu::TextureConfig {
                size: wgpu::Extent3d {
                    width,
                    height,
                    ..Default::default()
                },
                label: None,
                format: Some(wgpu::TextureFormat::Rgba8Unorm),
                ..Default::default()
            };

            let new_texture = imgui_wgpu::Texture::new(device, renderer, texture_config);
            new_texture.write(queue, &raw_data, width, height);
            *render_texture_id = renderer.textures.insert(new_texture);
        } else {
            texture.write(queue, &raw_data, width, height);
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

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = if cfg!(feature = "metal-auto-capture") {
                ControlFlow::Exit
            } else {
                ControlFlow::Poll
            };
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

                    if from_last_tick.as_secs_f32() > 1.0 {
                        from_last_tick = Duration::ZERO;
                        println!("Image updated");
                        Self::update_image(
                            &self.control_panel_url,
                            &self.render_task_md5,
                            &mut self.renderer,
                            &mut self.render_texture_id,
                            &self.queue,
                            &self.device,
                        );
                    }

                    {
                        //let size = [texture_width as f32, texture_height as f32];
                        let window = ui.window("render");
                        window
                            .size([512.0, 512.0], Condition::FirstUseEver)
                            .build(|| {
                                Image::new(self.render_texture_id, [512.0, 512.0]).build(ui);
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

async fn fetch_render(control_panel_url: &str, render_task_md5: &str) -> RgbaImage {
    let res = reqwest::get(format!(
        "{}/render_tasks/{}/render",
        control_panel_url, render_task_md5
    ))
    .await
    .unwrap()
    .error_for_status()
    .unwrap();
    let res: GetRenderResponse = res.json().await.unwrap();

    if res.image_data.len() == 0 {
        return RgbaImage::new(100, 100);
    }
    let image = Rgba32FImage::from_raw(res.image_width, res.image_height, res.image_data).unwrap();

    let mut gamma_corrected_image = RgbaImage::new(res.image_width, res.image_height);
    for x in 0..res.image_width {
        for y in 0..res.image_height {
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
