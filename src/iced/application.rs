use crate::iced::message;
use crate::iced::program::{self, Program};
use crate::iced::theme;
use crate::iced::window;
use crate::iced::{Element, Executor, Font, Never, Preset, Settings, Subscription, Task, Theme};

use iced_debug as debug;

use std::borrow::Cow;

mod timed;
pub use timed::timed;

pub fn application<State, Message, Theme, Renderer>(
    boot: impl BootFn<State, Message>, update: impl UpdateFn<State, Message>,
    view: impl for<'a> ViewFn<'a, State, Message, Theme, Renderer>,
) -> Application<impl Program<State = State, Message = Message, Theme = Theme>>
where
    State: Send + 'static,
    Message: Send + 'static,
    Theme: theme::Base,
    Renderer: program::Renderer,
{
    use std::marker::PhantomData;

    struct Instance<State, Message, Theme, Renderer, Boot, Update, View> {
        boot: Boot,
        update: Update,
        view: View,
        _state: PhantomData<State>,
        _message: PhantomData<Message>,
        _theme: PhantomData<Theme>,
        _renderer: PhantomData<Renderer>,
    }

    impl<State, Message, Theme, Renderer, Boot, Update, View> Program
        for Instance<State, Message, Theme, Renderer, Boot, Update, View>
    where
        Message: Send + 'static,
        Theme: theme::Base,
        Renderer: program::Renderer,
        Boot: self::BootFn<State, Message>,
        Update: self::UpdateFn<State, Message>,
        View: for<'a> self::ViewFn<'a, State, Message, Theme, Renderer>,
    {
        type State = State;
        type Message = Message;
        type Theme = Theme;
        type Renderer = Renderer;
        type Executor = iced_futures::backend::default::Executor;

        fn name() -> &'static str {
            let name = std::any::type_name::<State>();

            name.split("::").next().unwrap_or("a_cool_application")
        }

        fn boot(&self) -> (State, Task<Message>) {
            self.boot.boot()
        }

        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.update.update(state, message)
        }

        fn view<'a>(
            &self, state: &'a Self::State, _window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.view.view(state)
        }

        fn settings(&self) -> Settings {
            Settings::default()
        }

        fn window(&self) -> Option<iced_core::window::Settings> {
            Some(window::Settings::default())
        }
    }

    Application {
        raw: Instance {
            boot,
            update,
            view,
            _state: PhantomData,
            _message: PhantomData,
            _theme: PhantomData,
            _renderer: PhantomData,
        },
        iced_settings: Settings::default(),
        presets: Vec::new(),
    }
}

pub struct Application<P: Program> {
    raw: P,
    iced_settings: Settings,
    presets: Vec<Preset<P::State, P::Message>>,
}

impl<P: Program + Send> Application<P> {
    pub fn run(self) -> impl Program
    where
        Self: 'static,
        P::Message: message::MaybeDebug + message::MaybeClone,
    {
        #[cfg(feature = "debug")]
        iced_debug::init(iced_debug::Metadata {
            name: P::name(),
            theme: None,
            can_time_travel: cfg!(feature = "time-travel"),
        });

        #[cfg(all(feature = "debug", not(target_arch = "wasm32")))]
        let program = iced_devtools::attach(ApplicationInner {
            raw: self.raw,
            iced_settings: self.iced_settings,
            presets: self.presets,
        });

        #[cfg(not(any(all(feature = "debug", not(target_arch = "wasm32")))))]
        let program = ApplicationInner {
            raw: self.raw,
            iced_settings: self.iced_settings,
            presets: self.presets,
        };

        program
    }

    pub fn settings(self, settings: Settings) -> Self {
        Self { iced_settings: settings, ..self }
    }

    pub fn antialiasing(self, antialiasing: bool) -> Self {
        Self { iced_settings: Settings { antialiasing, ..self.iced_settings }, ..self }
    }

    pub fn default_font(self, default_font: Font) -> Self {
        Self { iced_settings: Settings { default_font, ..self.iced_settings }, ..self }
    }

    pub fn font(mut self, font: impl Into<Cow<'static, [u8]>>) -> Self {
        self.iced_settings.fonts.push(font.into());
        self
    }

    pub fn title(
        self, title: impl TitleFn<P::State>,
    ) -> Application<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        Application {
            raw: program::with_title(self.raw, move |state, _window| title.title(state)),
            iced_settings: self.iced_settings,
            presets: self.presets,
        }
    }

    pub fn subscription(
        self, f: impl Fn(&P::State) -> Subscription<P::Message>,
    ) -> Application<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        Application {
            raw: program::with_subscription(self.raw, f),
            iced_settings: self.iced_settings,
            presets: self.presets,
        }
    }

    pub fn theme(
        self, f: impl ThemeFn<P::State, P::Theme>,
    ) -> Application<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        Application {
            raw: program::with_theme(self.raw, move |state, _window| f.theme(state)),
            iced_settings: self.iced_settings,
            presets: self.presets,
        }
    }

    pub fn style(
        self, f: impl Fn(&P::State, &P::Theme) -> theme::Style,
    ) -> Application<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        Application {
            raw: program::with_style(self.raw, f),
            iced_settings: self.iced_settings,
            presets: self.presets,
        }
    }

    pub fn scale_factor(
        self, f: impl Fn(&P::State) -> f32,
    ) -> Application<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>> {
        Application {
            raw: program::with_scale_factor(self.raw, move |state, _window| f(state)),
            iced_settings: self.iced_settings,
            presets: self.presets,
        }
    }

    pub fn executor<E>(
        self,
    ) -> Application<impl Program<State = P::State, Message = P::Message, Theme = P::Theme>>
    where
        E: Executor,
    {
        Application {
            raw: program::with_executor::<P, E>(self.raw),
            iced_settings: self.iced_settings,
            presets: self.presets,
        }
    }

    pub fn presets(self, presets: impl IntoIterator<Item = Preset<P::State, P::Message>>) -> Self {
        Self { presets: presets.into_iter().collect(), ..self }
    }
}

pub(crate) struct ApplicationInner<P: Program> {
    raw: P,
    iced_settings: Settings,
    presets: Vec<Preset<P::State, P::Message>>,
}

impl<P: Program> Program for ApplicationInner<P> {
    type State = P::State;
    type Message = P::Message;
    type Theme = P::Theme;
    type Renderer = P::Renderer;
    type Executor = P::Executor;

    fn name() -> &'static str {
        P::name()
    }

    fn settings(&self) -> Settings {
        self.iced_settings.clone()
    }

    fn window(&self) -> Option<window::Settings> {
        Some(window::Settings::default())
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        self.raw.boot()
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        debug::hot(|| self.raw.update(state, message))
    }

    fn view<'a>(
        &self, state: &'a Self::State, window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        debug::hot(|| self.raw.view(state, window))
    }

    fn title(&self, state: &Self::State, window: window::Id) -> String {
        debug::hot(|| self.raw.title(state, window))
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        debug::hot(|| self.raw.subscription(state))
    }

    fn theme(&self, state: &Self::State, window: iced_core::window::Id) -> Option<Self::Theme> {
        debug::hot(|| self.raw.theme(state, window))
    }

    fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
        debug::hot(|| self.raw.style(state, theme))
    }

    fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
        debug::hot(|| self.raw.scale_factor(state, window))
    }

    fn presets(&self) -> &[Preset<Self::State, Self::Message>] {
        &self.presets
    }
}

pub trait BootFn<State, Message> {
    fn boot(&self) -> (State, Task<Message>);
}

impl<T, C, State, Message> BootFn<State, Message> for T
where
    T: Fn() -> C,
    C: IntoBoot<State, Message>,
{
    fn boot(&self) -> (State, Task<Message>) {
        self().into_boot()
    }
}

pub trait IntoBoot<State, Message> {
    fn into_boot(self) -> (State, Task<Message>);
}

impl<State, Message> IntoBoot<State, Message> for State {
    fn into_boot(self) -> (State, Task<Message>) {
        (self, Task::none())
    }
}

impl<State, Message> IntoBoot<State, Message> for (State, Task<Message>) {
    fn into_boot(self) -> (State, Task<Message>) {
        self
    }
}

pub trait TitleFn<State> {
    fn title(&self, state: &State) -> String;
}

impl<State> TitleFn<State> for &'static str {
    fn title(&self, _state: &State) -> String {
        self.to_string()
    }
}

impl<T, State> TitleFn<State> for T
where
    T: Fn(&State) -> String,
{
    fn title(&self, state: &State) -> String {
        self(state)
    }
}

pub trait UpdateFn<State, Message> {
    fn update(&self, state: &mut State, message: Message) -> Task<Message>;
}

impl<State> UpdateFn<State, Never> for () {
    fn update(&self, _state: &mut State, _message: Never) -> Task<Never> {
        Task::none()
    }
}

impl<T, State, Message, C> UpdateFn<State, Message> for T
where
    T: Fn(&mut State, Message) -> C,
    C: Into<Task<Message>>,
{
    fn update(&self, state: &mut State, message: Message) -> Task<Message> {
        self(state, message).into()
    }
}

pub trait ViewFn<'a, State, Message, Theme, Renderer> {
    fn view(&self, state: &'a State) -> Element<'a, Message, Theme, Renderer>;
}

impl<'a, T, State, Message, Theme, Renderer, Widget> ViewFn<'a, State, Message, Theme, Renderer>
    for T
where
    T: Fn(&'a State) -> Widget,
    State: 'static,
    Widget: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn view(&self, state: &'a State) -> Element<'a, Message, Theme, Renderer> {
        self(state).into()
    }
}

pub trait ThemeFn<State, Theme> {
    fn theme(&self, state: &State) -> Option<Theme>;
}

impl<State> ThemeFn<State, Theme> for Theme {
    fn theme(&self, _state: &State) -> Option<Theme> {
        Some(self.clone())
    }
}

impl<F, T, State, Theme> ThemeFn<State, Theme> for F
where
    F: Fn(&State) -> T,
    T: Into<Option<Theme>>,
{
    fn theme(&self, state: &State) -> Option<Theme> {
        (self)(state).into()
    }
}
