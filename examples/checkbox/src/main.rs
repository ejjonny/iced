use iced::animation::{self, AnimatedValue, Timing};
use iced::executor;
use iced::font::{self, Font};
use iced::theme::Checkbox;
use iced::widget::animated::Animating;
use iced::widget::checkbox::{Appearance, CheckboxState};
use iced::widget::{checkbox, column, container, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    default_checkbox: bool,
    custom_checkbox: CheckboxState,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            default_checkbox: false,
            custom_checkbox: CheckboxState {
                checked: false,
                checked_amount: AnimatedValue::new(0.0),
                hovered: false,
                hovered_amount: AnimatedValue::new(0.0),
            },
        }
    }
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
            anim.checked_amount.duration_ms = 1000.0;
            anim.checked_amount.timing = animation::Timing::EaseOutQuint;
            anim.hovered_amount.duration_ms = 200.0;
            anim.hovered_amount.timing = animation::Timing::EaseOutQuint;
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
