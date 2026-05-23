use std::{net::SocketAddr, sync::Arc};

use ::image::RgbaImage;
use futures::StreamExt;
use iced::{
    Element, Subscription, Task,
    advanced::image::Handle as ImageHandle,
    application::BootFn,
    widget::{center, column, container, container::Style, image, text},
};
use iced_aw::{TabLabel, Tabs};

use crate::{frame::Frame, worker_pool::Stats as WorkerPoolStats};

pub fn start(frame: Arc<Frame>, worker_pool_stats: WorkerPoolStats) -> iced::Result {
    iced::application(
        Layout {
            frame,
            worker_pool_stats,
            active_tab: Default::default(),
            render: None,
            worker_addresses: vec![],
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
    worker_pool_stats: WorkerPoolStats,

    active_tab: TabId,
    render: Option<RgbaImage>,
    worker_addresses: Vec<String>,
}

#[derive(Debug)]
enum Message {
    NewRender(RgbaImage),
    WorkerPoolStatsChanged(Vec<SocketAddr>),
    TabSelected(TabId),
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
                worker_pool_stats: self.worker_pool_stats.clone(),
                active_tab: Default::default(),
                render: None,
                worker_addresses: vec![],
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
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            Subscription::run_with(self.worker_pool_stats.clone(), |stats| {
                stats
                    .clone()
                    .get_worker_addresses_stream()
                    .map(Message::WorkerPoolStatsChanged)
            }),
            Subscription::run_with(self.frame.clone(), |frame| {
                frame.clone().get_image_stream().map(Message::NewRender)
            }),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        Tabs::new(Message::TabSelected)
            .push(
                TabId::Render,
                TabLabel::Text("render".to_string()),
                render_tab(&self.render),
            )
            .push(
                TabId::Workers,
                TabLabel::Text("workers".to_string()),
                workers_tab(&self.worker_addresses),
            )
            .set_active_tab(&self.active_tab)
            .into()
    }
}

fn render_tab(render: &Option<RgbaImage>) -> Element<'_, Message> {
    let render = match render {
        Some(render) => column![image(ImageHandle::from_rgba(
            render.width(),
            render.height(),
            render.to_vec(),
        ))],
        None => column![],
    };

    center(render).padding(10).into()
}

fn workers_tab(addresses: &[String]) -> Element<'_, Message> {
    let entries: Vec<_> = addresses
        .iter()
        .map(|address| {
            container(&**address)
                .padding(8)
                .style(|theme| Style {
                    border: iced::Border::default().rounded(8),
                    ..container::rounded_box(theme)
                })
                .into()
        })
        .collect();

    if addresses.is_empty() {
        center(text("No workers found")).into()
    } else {
        center(column(entries).spacing(8)).into()
    }
}
