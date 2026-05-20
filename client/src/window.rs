use std::sync::Arc;

use ::image::RgbaImage;
use futures::StreamExt;
use iced::{
    Element, Subscription, Task,
    advanced::image::Handle as ImageHandle,
    application::BootFn,
    widget::{center, column, image, text},
};
use iced_aw::{TabLabel, Tabs};

use crate::frame::Frame;

pub fn start(frame: Arc<Frame>) -> iced::Result {
    iced::application(
        Layout {
            frame,
            render: None,
            active_tab: Default::default(),
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
    render: Option<RgbaImage>,
    active_tab: TabId,
}

#[derive(Debug)]
enum Message {
    NewRender(RgbaImage),
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
                render: None,
                active_tab: Default::default(),
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
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run_with(self.frame.clone(), |frame| {
            frame.clone().get_image_stream().map(Message::NewRender)
        })
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
                workers_tab(),
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

fn workers_tab() -> Element<'static, Message> {
    center(text("Worker list")).into()
}
