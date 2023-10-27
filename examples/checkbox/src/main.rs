use iced::animation::{self, Animation, Interpolable, Timing};
use iced::executor;
use iced::font::{self, Font};
use iced::theme::Checkbox;
use iced::widget::animated::{AnimatableConvertible, Animator};
use iced::widget::checkbox::Appearance;
use iced::widget::{checkbox, column, container, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    checked: bool,
    default_checkbox: bool,
    hovered: bool,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            checked: false,
            default_checkbox: false,
            hovered: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Checked,
    Hovered(bool),
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
            Message::Checked => {
                self.checked = !self.checked;
            }
            Message::Hovered(value) => {
                self.hovered = value;
            }
            Message::FontLoaded(_) => (),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // let default_checkbox =
        //     checkbox("Default", self.default_checkbox, Message::Checked);
        // let custom_checkbox = checkbox(
        //     "Custom",
        //     self.checked,
        //     Message::Checked,
        //     Message::Hovered,
        // )
        // .icon(checkbox::Icon {
        //     font: ICON_FONT,
        //     code_point: '\u{e901}',
        //     size: None,
        //     line_height: text::LineHeight::Relative(1.0),
        //     shaping: text::Shaping::Basic,
        // })
        // .style(Checkbox::Success);
        // let checkbox = checkbox(
        //     "Custom",
        //     false,
        //     1.0,
        //     1.0,
        //     || Message::Checked,
        //     Message::Hovered,
        // )
        // .icon(checkbox::Icon {
        //     font: ICON_FONT,
        //     code_point: '\u{e901}',
        //     size: None,
        //     line_height: text::LineHeight::Relative(1.0),
        //     shaping: text::Shaping::Basic,
        // });
        let hovered = self.hovered.clone();
        let animating = Animator::new(
            (self.checked.animatable(), self.hovered.animatable()),
            std::time::Duration::from_millis(500),
            Timing::EaseOut,
            move |(checked_amount, hovered_amount)| {
                checkbox(
                    "Custom",
                    hovered,
                    checked_amount,
                    hovered_amount,
                    || Message::Checked,
                    Message::Hovered,
                )
                .icon(checkbox::Icon {
                    font: ICON_FONT,
                    code_point: '\u{e901}',
                    size: None,
                    line_height: text::LineHeight::Relative(1.0),
                    shaping: text::Shaping::Basic,
                })
                .into()
            },
        );
        // let animating = Animator::new( |checked| {

        //     },
        //     self.checked.animatable(),
        //     std::time::Duration::from_millis(500),
        //     Timing::EaseOutQuint,
        // );
        // let animating = Animating::new(
        //     Element::from(custom_checkbox),
        //     if self.default_checkbox { 1.0 } else { 0.0 },
        //     Message::AnimationUpdate,
        // );
        // .animation(|anim| {
        //     anim.checked_amount.duration_ms = 1000.0;
        //     anim.checked_amount.timing = animation::Timing::EaseOutQuint;
        //     anim.hovered_amount.duration_ms = 200.0;
        //     anim.hovered_amount.timing = animation::Timing::EaseOutQuint;
        // });

        let content = column![animating].spacing(22);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
