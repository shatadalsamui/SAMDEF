use crate::model::DisplayDetection;
use crate::viewer::events::handle_viewer_event;
use crate::viewer::render::render_viewer;
use crate::viewer::state::ViewerState;
use iced::advanced::image;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::event::Event;
use iced::{Element, Length, Rectangle, Size, Vector};

pub struct AnnotatedViewer<'a, Message> {
    handle: image::Handle,
    detections: &'a [DisplayDetection],
    class_visible: [bool; 8],
    confidence_threshold: f32,
    show_labels: bool,
    reset_counter: usize,
    width: Length,
    height: Length,
    min_scale: f32,
    max_scale: f32,
    scale_step: f32,
    on_status: Option<crate::viewer::state::StatusCallback<'a, Message>>,
}

impl<'a, Message> AnnotatedViewer<'a, Message> {
    pub fn new(
        handle: image::Handle,
        detections: &'a [DisplayDetection],
        class_visible: [bool; 8],
        confidence_threshold: f32,
        show_labels: bool,
        reset_counter: usize,
    ) -> Self {
        Self {
            handle,
            detections,
            class_visible,
            confidence_threshold,
            show_labels,
            reset_counter,
            width: Length::Fill,
            height: Length::Fill,
            min_scale: 0.05,
            max_scale: 25.0,
            scale_step: 0.15,
            on_status: None,
        }
    }

    pub fn on_status<F>(mut self, f: F) -> Self
    where
        F: Fn(f32, Option<(f32, f32)>) -> Message + 'a,
    {
        self.on_status = Some(Box::new(f));
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for AnnotatedViewer<'a, Message>
where
    Renderer: image::Renderer<Handle = image::Handle>
        + text::Renderer<Font = iced::Font>
        + renderer::Renderer,
    Message: 'a + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ViewerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ViewerState::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = limits.width(self.width).height(self.height).resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<ViewerState>();

        // Handle external view reset request
        if state.last_reset != self.reset_counter {
            state.scale = 1.0;
            state.current_offset = Vector::default();
            state.starting_offset = Vector::default();
            state.cursor_grabbed_at = None;
            state.last_reset = self.reset_counter;
        }

        handle_viewer_event(
            state,
            event,
            bounds,
            cursor,
            renderer,
            &self.handle,
            self.min_scale,
            self.max_scale,
            self.scale_step,
            &self.on_status,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<ViewerState>();
        let bounds = layout.bounds();

        if state.cursor_grabbed_at.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<ViewerState>();
        let bounds = layout.bounds();

        render_viewer(
            renderer,
            bounds,
            &self.handle,
            state,
            self.detections,
            &self.class_visible,
            self.confidence_threshold,
            self.show_labels,
        );
    }
}

impl<'a, Message, Renderer> From<AnnotatedViewer<'a, Message>> for Element<'a, Message, Renderer>
where
    Renderer: 'a
        + image::Renderer<Handle = image::Handle>
        + text::Renderer<Font = iced::Font>
        + renderer::Renderer,
    Message: 'a + Clone,
{
    fn from(viewer: AnnotatedViewer<'a, Message>) -> Self {
        Element::new(viewer)
    }
}
