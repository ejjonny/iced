use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::window;
use crate::core::widget::Tree;
use crate::core::{
    Clipboard, Element, Layout, Length, Rectangle,
    Shell, Widget,
};
use iced_style::animation::{AnimatedValue, Interpolable, Timing, Animatable};

pub struct Animating<'a, Message, T, Renderer = crate::Renderer>
where
    T: Animatable,
{
    child: Element<'a, Message, Renderer>,
    animation: T,
    animation_update: Box<dyn Fn(T) -> Message + 'a>,
}

impl<'a, Message, T, Renderer> Animating<'a, Message, T, Renderer>
where
    T: Animatable + Clone,
{
    pub fn new<F>(
        child: Element<'a, Message, Renderer>,
        animation: T,
        animation_update: F,
    ) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        Animating {
            child,
            animation,
            animation_update: Box::new(animation_update),
        }
    }

    pub fn animation(mut self, mut configuration: impl FnMut(&mut T)) -> Self {
        configuration(&mut self.animation);
        self
    }
}

impl<'a, Message, T, Renderer> Widget<Message, Renderer>
    for Animating<'a, Message, T, Renderer>
where
    T: Animatable + Copy,
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
        self.child
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
        self.child
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
                if self.animation.on_redraw_request_update(now) {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                    shell.publish((self.animation_update)(self.animation))
                }
            }
            _ => {}
        }
        std::iter::once(self)
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((animating, state), layout)| {
                animating.child.as_widget_mut().on_event(
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
        self.child
            .as_widget()
            .operate(state, layout, renderer, operation)
    }
    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.child.as_widget().layout(renderer, limits)
    }
    fn width(&self) -> Length {
        self.child.as_widget().width()
    }
    fn height(&self) -> Length {
        self.child.as_widget().height()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }
}

impl<'a, Message, T, Renderer> From<Animating<'a, Message, T, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    T: Animatable + Copy + 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(animating: Animating<'a, Message, T, Renderer>) -> Self {
        Self::new(animating)
    }
}
