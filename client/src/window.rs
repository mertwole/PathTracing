use iced::keyboard;
use iced::widget::{
    button, canvas, center, center_y, checkbox, column, container, pick_list, pin, responsive, row,
    rule, scrollable, space, stack, text,
};
use iced::{
    Center, Element, Fill, Font, Length, Point, Rectangle, Renderer, Shrink, Subscription, Theme,
    color,
};

pub fn start() -> iced::Result {
    iced::application(Layout::default, Layout::update, Layout::view)
        .subscription(Layout::subscription)
        .theme(Layout::theme)
        .title(Layout::title)
        .run()
}

#[derive(Debug, Default)]
struct Layout {
    theme: Option<Theme>,
}

#[derive(Debug, Clone)]
enum Message {}

impl Layout {
    fn title(&self) -> String {
        format!("")
    }

    fn update(&mut self, _message: Message) {}

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|_event| None)
    }

    fn view(&self) -> Element<'_, Message> {
        let header = row![text("").size(20).font(Font::MONOSPACE), space::horizontal(),]
            .spacing(20)
            .align_y(Center);

        let example = centered();

        column![header, example].spacing(10).padding(20).into()
    }

    fn theme(&self) -> Option<Theme> {
        self.theme.clone()
    }
}

fn centered<'a>() -> Element<'a, Message> {
    center(text("I am centered!").size(50)).into()
}
