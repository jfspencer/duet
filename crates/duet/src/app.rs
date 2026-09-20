//! The root view of the Duet window.

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::{Context, IntoElement, Render, SharedString, Window, div};

/// Root view: a title and a counter button that proves state round-trips
/// through a `cx.listener` and a `cx.notify()` redraw.
#[derive(Debug, Default)]
pub(crate) struct DuetApp {
    /// How many times the button has been pressed.
    clicks: u32,
}

/// The label shown under the title for a click count.
fn click_label(clicks: u32) -> String {
    match clicks {
        0 => "Not clicked yet".to_owned(),
        1 => "Clicked once".to_owned(),
        n => format!("Clicked {n} times"),
    }
}

impl Render for DuetApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let label: SharedString = click_label(self.clicks).into();
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(div().text_2xl().font_semibold().child("Duet"))
            .child(label)
            .child(
                Button::new("count")
                    .primary()
                    .label("Count")
                    .on_click(cx.listener(|view, _click, _win, view_cx| {
                        view.clicks = view.clicks.saturating_add(1);
                        view_cx.notify();
                    })),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::click_label;

    #[test]
    fn label_reads_naturally() {
        assert_eq!(click_label(0), "Not clicked yet", "zero clicks");
        assert_eq!(click_label(1), "Clicked once", "one click");
        assert_eq!(click_label(7), "Clicked 7 times", "many clicks");
    }
}
