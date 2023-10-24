use iced::animation::{AnimatedValue, Timing, self};
use iced::executor;
use iced::font::{self, Font};
use iced::theme::Checkbox;
use iced::widget::animated::Animating;
use iced::widget::checkbox::{CheckboxState, Appearance};
use iced::widget::{checkbox, column, container, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    default_checkbox: bool,
    custom_checkbox: CheckboxState,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    DefaultChecked(bool),
    CustomChecked(bool),
    FontLoaded(Result<(), font::Error>),
}

impl Application for Example {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self::default(),
            font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                .map(Message::FontLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Checkbox - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DefaultChecked(value) => self.default_checkbox = value,
            Message::CustomChecked(value) => self.custom_checkbox = value,
                self.custom_checkbox.check(value);
                self.custom_checkbox.hover(value);
            Message::FontLoaded(_) => (),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let default_checkbox =
            checkbox("Default", self.default_checkbox, Message::DefaultChecked);
        let custom_checkbox =
            checkbox("Custom", self.custom_checkbox, Message::CustomChecked)
                .icon(checkbox::Icon {
                    font: ICON_FONT,
                    code_point: '\u{e901}',
                    size: None,
                    line_height: text::LineHeight::Relative(1.0),
                    shaping: text::Shaping::Basic,
        })
        .style(Checkbox::Success);
        .animation(|anim| {
            anim.checked_amount.duration =
                std::time::Duration::from_millis(1000);
            anim.checked_amount.timing = animation::Timing::Linear;
            anim.hovered_amount.duration =
                std::time::Duration::from_millis(100);
            anim.hovered_amount.timing = animation::Timing::EaseInOut;
        });

        let content = column![default_checkbox, custom_checkbox].spacing(22);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
