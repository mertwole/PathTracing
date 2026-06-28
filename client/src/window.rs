use std::{net::SocketAddr, sync::Arc};

use ::image::RgbaImage;
use futures::StreamExt;
use iced::{
    Alignment, Element, Subscription, Task,
    advanced::image::Handle as ImageHandle,
    alignment::Horizontal,
    application::BootFn,
    widget::{
        self, button, center, column, container as container_widget, container::Style, image, row,
        space, text, text_input,
    },
};
use iced_aw::{TabLabel, Tabs};

use crate::{
    frame::Frame,
    worker_pool::{self},
};

pub fn start(frame: Arc<Frame>, worker_pool: worker_pool::Handle) -> iced::Result {
    iced::application(
        Layout {
            frame,
            worker_pool,
            active_tab: Default::default(),
            render: None,
            worker_addresses: vec![],
            render_task: RenderTaskData {
                scene_path: "./simple_scene.json".to_string(),
                iteration_count: "1".to_string(),
                trace_depth: "8".to_string(),
            },
        },
        Layout::update,
        Layout::view,
    )
    .subscription(Layout::subscription)
    .title(Layout::title)
    .run()
}

struct Layout {
    frame: Arc<Frame>,
    worker_pool: worker_pool::Handle,

    active_tab: TabId,
    render: Option<RgbaImage>,
    worker_addresses: Vec<String>,

    render_task: RenderTaskData,
}

#[derive(Clone)]
struct RenderTaskData {
    scene_path: String,
    iteration_count: String,
    trace_depth: String,
}

#[derive(Debug, Clone)]
enum Message {
    // TODO: Box RgbaImage.
    NewRender(RgbaImage),
    WorkerPoolStatsChanged(Vec<SocketAddr>),
    StartWorkerDiscovery,
    TabSelected(TabId),
    ScenePathChanged(String),
    IterationCountChanged(String),
    TraceDepthChanged(String),
    SubmitRenderTask,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
enum TabId {
    #[default]
    Render,
    Workers,
}

impl BootFn<Layout, Message> for Layout {
    fn boot(&self) -> (Layout, Task<Message>) {
        (
            Layout {
                frame: self.frame.clone(),
                worker_pool: self.worker_pool.clone(),
                active_tab: Default::default(),
                render: None,
                worker_addresses: vec![],
                render_task: self.render_task.clone(),
            },
            Task::none(),
        )
    }
}

impl Layout {
    fn title(&self) -> String {
        "".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewRender(render) => {
                self.render = Some(render);
            }
            Message::WorkerPoolStatsChanged(addresses) => {
                self.worker_addresses = addresses
                    .into_iter()
                    .map(|address| format!("{address}"))
                    .collect();
            }
            Message::StartWorkerDiscovery => {
                self.worker_pool.discover();
            }
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Message::ScenePathChanged(path) => {
                self.render_task.scene_path = path;
            }
            Message::IterationCountChanged(iterations) => {
                self.render_task.iteration_count = iterations;
            }
            Message::TraceDepthChanged(depth) => {
                self.render_task.trace_depth = depth;
            }
            Message::SubmitRenderTask => {
                // TODO
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            Subscription::run_with(self.worker_pool.clone(), |pool| {
                pool.get_worker_discovery_stream()
                    .map(Message::WorkerPoolStatsChanged)
            }),
            Subscription::run_with(self.frame.clone(), |frame| {
                frame.clone().get_image_stream().map(Message::NewRender)
            }),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let render_task = render_task(&self.render_task);
        let worker_list = worker_list(&self.worker_addresses);

        let submit_render_task = button("submit").on_press(Message::SubmitRenderTask);
        let submit_render_task =
            row![space::horizontal(), submit_render_task, space::horizontal()].padding(8);

        let left_panel = column![
            render_task,
            worker_list,
            space::vertical(),
            submit_render_task
        ]
        .width(256);

        let render = render_area(&self.render);

        row![left_panel, render].into()
    }
}

fn render_task<'a>(render_task: &'a RenderTaskData) -> Element<'a, Message> {
    let scene_path = row![
        text("scene path"),
        text_input("", &render_task.scene_path).on_input(Message::ScenePathChanged)
    ];
    let iterations = row![
        text("iterations"),
        text_input("", &render_task.iteration_count).on_input(Message::IterationCountChanged)
    ];
    let trace_depth = row![
        text("trace depth"),
        text_input("", &render_task.trace_depth).on_input(Message::TraceDepthChanged)
    ];

    container_widget(column![scene_path, iterations, trace_depth].spacing(8))
        .padding(8)
        .into()
}

fn worker_list(addresses: &[String]) -> Element<'_, Message> {
    let discover = button("discover workers").on_press(Message::StartWorkerDiscovery);
    let discover = container_widget(discover)
        .padding(8)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start);

    let entries: Vec<_> = addresses
        .iter()
        .map(|address| {
            container_widget(&**address)
                .padding(8)
                .style(|theme| Style {
                    border: iced::Border::default().rounded(8),
                    ..widget::container::rounded_box(theme)
                })
                .into()
        })
        .collect();

    let worker_list: Element<_> = if addresses.is_empty() {
        text("No workers found").into()
    } else {
        column(entries).spacing(8).into()
    };
    let worker_list = container_widget(worker_list).padding(8);

    column![discover, worker_list].into()
}

fn render_area<'a>(render: &'a Option<RgbaImage>) -> Element<'a, Message> {
    let render = match render {
        Some(render) => column![image(ImageHandle::from_rgba(
            render.width(),
            render.height(),
            render.to_vec(),
        ))],
        None => column![],
    };

    center(render).into()
}
