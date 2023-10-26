use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::window;
use crate::core::{
    Clipboard, Element, Layout, Length, Rectangle, Shell, Widget,
};
use iced_renderer::core::widget::tree::State;
use iced_style::animation::{Timing, AnimatableValue, Animation};

pub struct Animating<'a, Message, T, Renderer = crate::Renderer>
where
    T: AnimatableValue,
{
    child: Box<dyn Fn(T) -> Element<'a, Message, Renderer>>,
    animated_value: T,
    duration: std::time::Duration,
    timing: Timing,
}

impl<'a, Message, T, Renderer> Animating<'a, Message, T, Renderer>
where
    T: AnimatableValue,
{
    pub fn new<Content>(
        child: Content,
        animated_value: T,
        duration: std::time::Duration,
        timing: Timing,
    ) -> Self where Content: Fn(T) -> Element<'a, Message, Renderer> + 'static {
        Animating {
            child: Box::new(child),
            animated_value,
            duration,
            timing,
        }
    }
}

impl<'a, 'b, Message, T, Renderer> Widget<Message, Renderer>
    for Animating<'a, Message, T, Renderer>
where
    T: AnimatableValue + Clone + 'static,
    Renderer: crate::core::Renderer,
{
    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_renderer::core::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let animation = state
            .state
            .downcast_ref::<Animation<std::time::Instant, T>>()
            .timed_progress();
        (self.child)(animation)
            .as_widget()
            .draw(state, renderer, theme, style, layout, cursor, viewport)
    }
    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        (self.child)(self.animated_value.clone())
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = tree
                    .state
                    .downcast_mut::<Animation<std::time::Instant, T>>();
                match &state.animation_state {
                    Some(animation) => {
                        if animation.destination != self.animated_value {
                            state.transition(self.animated_value.clone(), now);
                        }
                    }
                    _ => {
                        if state.position != self.animated_value {
                            state.transition(self.animated_value.clone(), now)
                        }
                    }
                }
                if state.animating() {
                    let needs_redraw = state.tick(now);
                    if needs_redraw {
                        shell.request_redraw(
                            window::RedrawRequest::NextFrame,
                        );
                    }
                }
            }
            _ => {}
        }
        let animated_value = self.animated_value.clone();
        std::iter::once(self)
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((animating, state), layout)| {
                (animating.child)(animated_value.clone()).as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }
    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_renderer::core::widget::Operation<Message>,
    ) {
        (self.child)(self.animated_value.clone())
            .as_widget()
            .operate(state, layout, renderer, operation)
    }
    fn state(&self) -> State {
        let animation = Animation::<std::time::Instant, T>::new(
            self.animated_value.clone(),
            self.duration.as_millis() as f32,
            self.timing,
        );
        State::new(animation)
    }
    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        (self.child)(self.animated_value.clone()).as_widget().layout(renderer, limits)
    }
    fn width(&self) -> Length {
        (self.child)(self.animated_value.clone()).as_widget().width()
    }
    fn height(&self) -> Length {
        (self.child)(self.animated_value.clone()).as_widget().height()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&(self.child)(self.animated_value.clone()))]
    }
}

impl<'a, Message, T, Renderer> From<Animating<'a, Message, T, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    T: AnimatableValue + Copy + 'static,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(animating: Animating<'a, Message, T, Renderer>) -> Self {
        Self::new(animating)
    }
}
