use std::sync::Arc;

use ::image::RgbaImage;
use futures::StreamExt;
use iced::{
    Element, Subscription, Task,
    advanced::image::Handle as ImageHandle,
    application::BootFn,
    widget::{center, column, image},
};

use crate::frame::Frame;

pub fn start(frame: Arc<Frame>) -> iced::Result {
    iced::application(
        Layout {
            frame,
            render: None,
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
}

#[derive(Debug)]
enum Message {
    NewRender(RgbaImage),
}

impl BootFn<Layout, Message> for Layout {
    fn boot(&self) -> (Layout, Task<Message>) {
        (
            Layout {
                frame: self.frame.clone(),
                render: None,
            },
            Task::none(),
        )
    }
}

impl Layout {
    fn title(&self) -> String {
        format!("")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewRender(render) => {
                self.render = Some(render);
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run_with(self.frame.clone(), |frame| {
            frame
                .clone()
                .get_image_stream()
                .map(|image| Message::NewRender(image))
        })
    }

    fn view(&self) -> Element<'_, Message> {
        let render = match &self.render {
            Some(render) => column![image(ImageHandle::from_rgba(
                render.width(),
                render.height(),
                render.to_vec(),
            ))],
            None => column![],
        };

        center(render).padding(10).into()
    }
}
