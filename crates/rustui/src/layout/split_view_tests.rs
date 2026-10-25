use super::*;
use crate::{localization::activate_localization, runtime::ComponentHost};
use rustui_i18n::{Catalogs, Locale};
use std::{cell::Cell, rc::Rc};

struct SplitProbe {
    axis: Axis,
    unhandled: Rc<Cell<usize>>,
}

impl View for SplitProbe {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let split = match self.axis {
            Axis::Horizontal => SplitView::horizontal("event-path"),
            Axis::Vertical => SplitView::vertical("event-path"),
        }
        .panels(["first", "second"].map(|id| {
            crate::prelude::ResizablePanel::new(id, EmptyView)
                .initial_size(200.0)
                .size_range(100.0..300.0)
        }));
        let unhandled = self.unhandled.clone();
        div()
            .w(px(400.0))
            .h(px(400.0))
            .on_key_down(move |_, _, _| unhandled.set(unhandled.get() + 1))
            .child(split.into_element(context))
            .into_any_element()
    }
}

fn divider_size(native: &Entity<ResizableState>, cx: &mut gpui::VisualTestContext) -> f32 {
    cx.update(|window, app| {
        let _ = window.draw(app);
        native.read(app).sizes()[0].as_f32()
    })
}

#[gpui::test]
fn focused_divider_routes_physical_arrows_and_clamps_through_the_rendered_control(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(gpui_component::init);
    for (axis, locale, decrease, increase, unrelated, panel) in [
        (Axis::Horizontal, "en", "left", "right", "up", "first"),
        (Axis::Horizontal, "ar", "left", "right", "up", "second"),
        (Axis::Vertical, "ar", "up", "down", "left", "first"),
    ] {
        cx.update(|app| {
            activate_localization(
                Catalogs::new(Locale::parse("en").unwrap()),
                Locale::parse(locale).unwrap(),
                app,
            );
        });
        let unhandled = Rc::new(Cell::new(0));
        let (owner, window) = cx.add_window_view(|_, app| {
            ComponentHost::new(
                SplitProbe {
                    axis,
                    unhandled: unhandled.clone(),
                },
                app,
            )
        });
        let native = window.update(|window, app| {
            let _ = window.draw(app);
            let retained = owner
                .read(app)
                .retained_entity_for_test::<RetainedSplit>(
                    vec![RetentionScope::SplitView(Identity::from("event-path"))],
                    Identity::from("split-state"),
                )
                .expect("the production render retained the split state");
            assert_eq!(retained.read(app).ids[0].as_str(), panel);
            retained.read(app).native.clone()
        });
        let initial = divider_size(&native, window);
        assert_eq!(initial, 200.0);
        // Empty panels contain no controls; the rendered divider is the only tab stop.
        window.update(|window, app| {
            window.focus_next(app);
            assert!(window.focused(app).is_some());
        });

        window.simulate_keystrokes(decrease);
        assert_eq!(divider_size(&native, window), initial - 10.0);
        window.simulate_keystrokes(increase);
        assert_eq!(divider_size(&native, window), initial);
        assert_eq!(unhandled.get(), 0, "handled arrow must stop propagation");

        window.simulate_keystrokes(unrelated);
        window.simulate_keystrokes(&format!("shift-{decrease}"));
        assert_eq!(divider_size(&native, window), initial);
        assert_eq!(unhandled.get(), 2, "ignored keys must reach the ancestor");

        for _ in 0..12 {
            window.simulate_keystrokes(decrease);
        }
        assert_eq!(divider_size(&native, window), 100.0);
        for _ in 0..24 {
            window.simulate_keystrokes(increase);
        }
        assert_eq!(divider_size(&native, window), 300.0);
    }
}
