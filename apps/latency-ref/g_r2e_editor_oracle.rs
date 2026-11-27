//! Native Metal oracle for Kit's editor, including its real glyph atlas.
use gpui::{
    ClipboardItem, Entity, Keystroke, TestAppContext, TestDispatcher, Window, div, prelude::*, px,
    size,
};
use gpui_base::input::{EditorState, Position};
use std::time::Duration;

struct Editor(Entity<EditorState>, bool);
impl Render for Editor {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if self.1 {
            div()
                .w(px(1136.))
                .h(px(700.))
                .bg(gpui::rgb(0xffffff))
                .text_color(gpui::rgb(0x161616))
                .child(self.0.clone())
        } else {
            div().size_full().child(self.0.clone())
        }
    }
}

fn main() {
    let operation = std::env::args().nth(1).expect("operation");
    let frames: usize = std::env::args()
        .nth(2)
        .map(|n| n.parse().unwrap())
        .unwrap_or(300);
    let text = gpui_platform::current_platform(true).text_system();
    let mut cx = TestAppContext::build_with_metal_fixture(
        TestDispatcher::new(0),
        text,
        Box::new(gpui_platform::current_headless_renderer),
    );
    cx.update(gpui_base::init);
    let original = "first row\nsecond row\nthird row\nfourth row\nedit here\nlast row";
    let report_text = std::env::var_os("RUSTUI_G_R3_DOCUMENT")
        .map(|path| std::fs::read_to_string(path).expect("read the exported G0 typing document"));
    let original = report_text.as_deref().unwrap_or(original);
    let report_area = report_text.is_some();
    let viewport = if report_area {
        assert_eq!(original.lines().count(), 10_000);
        size(px(1728.), px(1117.))
    } else {
        size(px(480.), px(400.))
    };
    let mut editor = None;
    let handle = cx.open_window(viewport, |w, cx| {
        let entity = cx.new(|cx| {
            EditorState::new(w, cx)
                .soft_wrap(true)
                .default_value(original.to_owned())
        });
        editor = Some(entity.clone());
        Editor(entity, report_area)
    });
    let editor = editor.unwrap();
    cx.update_window(handle.into(), |_, w, cx| {
        w.activate_window();
        w.draw(cx).clear(cx);
        editor.update(cx, |e, cx| {
            // The G0 document starts with an unindented comment. An edit here
            // keeps the inverse newline operation independent of auto-indent.
            let cursor = if report_area {
                Position::new(0, 7)
            } else {
                Position::new(4, 4)
            };
            e.set_cursor_position(cursor, w, cx)
        });
        cx.write_to_clipboard(ClipboardItem::new_string("abc\ndef".into()));
    })
    .unwrap();
    cx.run_until_parked();
    if operation == "caret" {
        cx.executor().advance_clock(Duration::from_millis(1100));
        cx.run_until_parked();
    }
    let image = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
            w.render_to_image()
                .expect("declared frame vs full Metal redraw")
        })
        .unwrap()
    };
    let mut previous = image(&mut cx);
    if report_area {
        // Ignore the gutter, caret, borders and current line. A fontless
        // test platform still paints those while producing no glyph pixels.
        let document_ink = (100..1000)
            .flat_map(|y| (180..700).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let [r, g, b, a] = previous.get_pixel(x, y).0;
                a == 255 && r.max(g).max(b) < 160
            })
            .count();
        assert!(
            document_ink > 1000,
            "missing visible document text ({document_ink} dark pixels); build the area fixture with gpui_platform/font-kit"
        );
    }
    for i in 0..frames * 2 {
        let outward = i % 2 == 0;
        let key = match (operation.as_str(), outward) {
            ("edit", true) => "x",
            ("edit", false) => "backspace",
            ("newline", true) => "enter",
            ("newline", false) => "backspace",
            ("paste", true) => "cmd-v",
            ("paste", false) => "cmd-z",
            ("selection", true) => "shift-right",
            ("selection", false) => "left",
            ("caret", _) => "",
            _ => panic!("unrecognized operation {operation}"),
        };
        if key.is_empty() {
            cx.executor().advance_clock(Duration::from_millis(500));
        } else {
            cx.update_window(handle.into(), |_, w, cx| {
                assert!(
                    w.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx),
                    "unhandled {key}"
                );
            })
            .unwrap();
        }
        cx.run_until_parked();
        let current = image(&mut cx);
        assert!(
            current.as_raw() != previous.as_raw(),
            "no visible change for {operation} frame {i}"
        );
        previous = current;
        cx.update_window(handle.into(), |_, _, cx| {
            let state = editor.read(cx);
            let delta = match (operation.as_str(), outward) {
                ("edit" | "newline", true) => 1,
                ("paste", true) => 7,
                _ => 0,
            };
            assert_eq!(
                state.value().len(),
                original.len() + delta,
                "{operation} edit {i}"
            );
            if delta == 0 {
                assert_eq!(state.value().as_ref(), original);
            }
            if operation == "selection" {
                assert_eq!(state.selected_range().len(), usize::from(outward));
            }
        })
        .unwrap();
    }
    println!(
        "G_R2E_ORACLE {operation}: {} verified Partial frames",
        2 * frames
    );
}
