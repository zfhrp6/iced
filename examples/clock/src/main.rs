#[macro_use]
extern crate serde_derive;
mod twi;
use iced::{
    button, canvas, executor, text_input, Application, Button, Canvas, Color,
    Command, Container, Element, Length, Point, Row, Settings, Subscription,
    Text, TextInput, Vector,
};
use iced_native::Column;

pub fn main() {
    Clock::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct Counter {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
    post_button: button::State,
    text: String,
    text_state: text_input::State,
    config: twi::Conf,
}

impl Counter {
    fn new() -> Counter {
        Counter {
            config: twi::read_conf(),
            ..Counter::default()
        }
    }
    pub fn update(&mut self, message: ButtonClicked) -> Command<Message> {
        match message {
            ButtonClicked::Increment => {
                self.value += 1;
            }
            ButtonClicked::Decrement => {
                self.value -= 1;
            }
            ButtonClicked::Post => twi::hoge(&self.text, &self.config),
        }
        Command::none()
    }

    fn update_text(&mut self, s: String) -> Command<Message> {
        self.text = s;
        Command::none()
    }
}

struct Clock {
    now: LocalTime,
    clock: canvas::layer::Cache<LocalTime>,
    counter: Counter,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(chrono::DateTime<chrono::Local>),
    Clicked(ButtonClicked),
    TextChanged(String),
}

#[derive(Debug, Clone, Copy)]
enum ButtonClicked {
    Increment,
    Decrement,
    Post,
}

impl Application for Clock {
    type Executor = executor::Default;
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Clock {
                now: chrono::Local::now().into(),
                clock: canvas::layer::Cache::new(),
                counter: Counter::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Clock - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(local_time) => {
                let now = local_time.into();

                if now != self.now {
                    self.now = now;
                    self.clock.clear();
                }
            }
            Message::Clicked(button_clicked) => {
                self.counter.update(button_clicked);
            }
            Message::TextChanged(s) => {
                self.counter.update_text(s);
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(100)).map(Message::Tick)
    }

    fn view(&mut self) -> Element<Message> {
        let counter = Row::new()
            .push(
                Button::new(
                    &mut self.counter.increment_button,
                    Text::new("Increment"),
                )
                .on_press(Message::Clicked(ButtonClicked::Increment)),
            )
            .push(Text::new(&self.counter.value.to_string()).size(50))
            .push(
                Button::new(
                    &mut self.counter.decrement_button,
                    Text::new("Decrement"),
                )
                .on_press(Message::Clicked(ButtonClicked::Decrement)),
            )
            .push(TextInput::new(
                &mut self.counter.text_state,
                "hoge",
                &mut self.counter.text,
                Message::TextChanged,
            ))
            .push(
                Button::new(&mut self.counter.post_button, Text::new("Post"))
                    .on_press(Message::Clicked(ButtonClicked::Post)),
            );
        let canvas = Canvas::new()
            .width(Length::Units(400))
            .height(Length::Units(400))
            .push(self.clock.with(&self.now));

        let a: Element<Message> = Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        Column::new().push(counter).push(a).into()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LocalTime {
    hour: u32,
    minute: u32,
    second: u32,
}

impl From<chrono::DateTime<chrono::Local>> for LocalTime {
    fn from(date_time: chrono::DateTime<chrono::Local>) -> LocalTime {
        use chrono::Timelike;

        LocalTime {
            hour: date_time.hour(),
            minute: date_time.minute(),
            second: date_time.second(),
        }
    }
}

impl canvas::Drawable for LocalTime {
    fn draw(&self, frame: &mut canvas::Frame) {
        let center = frame.center();
        let radius = frame.width().min(frame.height()) / 2.0;
        let offset = Vector::new(center.x, center.y);

        let clock = canvas::Path::new(|path| path.circle(center, radius));

        frame.fill(
            &clock,
            canvas::Fill::Color(Color::from_rgb8(0x12, 0x93, 0xD8)),
        );

        fn draw_hand(
            n: u32,
            total: u32,
            length: f32,
            offset: Vector,
            path: &mut canvas::path::Builder,
        ) {
            let turns = n as f32 / total as f32;
            let t = 2.0 * std::f32::consts::PI * (turns - 0.25);

            let x = length * t.cos();
            let y = length * t.sin();

            path.line_to(Point::new(x, y) + offset);
        }

        let hour_and_minute_hands = canvas::Path::new(|path| {
            path.move_to(center);
            draw_hand(self.hour, 12, 0.5 * radius, offset, path);

            path.move_to(center);
            draw_hand(self.minute, 60, 0.8 * radius, offset, path)
        });

        frame.stroke(
            &hour_and_minute_hands,
            canvas::Stroke {
                width: 6.0,
                color: Color::WHITE,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            },
        );

        let second_hand = canvas::Path::new(|path| {
            path.move_to(center);
            draw_hand(self.second, 60, 0.8 * radius, offset, path)
        });

        frame.stroke(
            &second_hand,
            canvas::Stroke {
                width: 3.0,
                color: Color::WHITE,
                line_cap: canvas::LineCap::Round,
                ..canvas::Stroke::default()
            },
        );
    }
}

mod time {
    use iced::futures;

    pub fn every(
        duration: std::time::Duration,
    ) -> iced::Subscription<chrono::DateTime<chrono::Local>> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = chrono::DateTime<chrono::Local>;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| chrono::Local::now())
                .boxed()
        }
    }
}
