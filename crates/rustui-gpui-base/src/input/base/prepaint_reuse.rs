// RustUI G5-L6. Exact previous-frame line reuse, independent of view retention.
use gpui::{Font, Hsla, Pixels, TextRun, WindowId};
use ropey::RopeSlice;
use smallvec::SmallVec;
use std::{ops::Range, rc::Rc, sync::OnceLock};

use crate::input::display_map::{LineLayout, WrappingIndent};

pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        gpui::fast_g5_profile_enabled()
            || std::env::var_os("RUSTUI_G5_PREPAINT_REUSE").as_deref() == Some("1".as_ref())
    })
}

pub(super) fn verify() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        std::env::var_os("RUSTUI_G5_PREPAINT_VERIFY").as_deref() == Some("1".as_ref())
    })
}

#[derive(PartialEq)]
pub(super) struct ShapeSettings {
    pub(super) window: WindowId,
    pub(super) scale: f32,
    pub(super) font: Font,
    pub(super) size: Pixels,
    pub(super) content_width: Pixels,
    pub(super) wrap_width: Option<Pixels>,
    pub(super) wrapping_indent: WrappingIndent,
    pub(super) tab_size: usize,
    pub(super) whitespace_color: Option<Hsla>,
}

/// Store the reusable form separately so ordinary layouts still allocate exactly once.
#[derive(Clone)]
pub(super) enum LayoutRows {
    Plain(Rc<Vec<LineLayout>>),
    Cached(Rc<CachedRows>),
}

impl LayoutRows {
    pub(super) fn plain(lines: Vec<LineLayout>) -> Self {
        Self::Plain(Rc::new(lines))
    }

    pub(super) fn len(&self) -> usize {
        match self {
            Self::Plain(rows) => rows.len(),
            Self::Cached(rows) => rows.lines.len(),
        }
    }

    pub(super) fn get(&self, index: usize) -> Option<&LineLayout> {
        match self {
            Self::Plain(rows) => rows.get(index),
            Self::Cached(rows) => rows.lines.get(index).map(|row| &row.layout),
        }
    }

    pub(super) fn last(&self) -> Option<&LineLayout> {
        self.len().checked_sub(1).and_then(|last| self.get(last))
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &LineLayout> {
        (0..self.len()).map(|index| &self[index])
    }
}

impl std::ops::Index<usize> for LayoutRows {
    type Output = LineLayout;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("visible line layout index")
    }
}

struct CachedLine {
    text: String,
    wraps: SmallVec<[Range<usize>; 1]>,
    indent: u32,
    runs: Vec<TextRun>,
    layout: LineLayout,
}

pub(super) struct CachedRows {
    settings: ShapeSettings,
    lines: Vec<Rc<CachedLine>>,
    buffer_line_count: usize,
}

pub(super) struct RowReuse<'a> {
    old: Option<(&'a Rc<CachedRows>, &'a [usize])>,
    output: Option<Vec<Rc<CachedLine>>>,
    settings: ShapeSettings,
    visible_count: usize,
    buffer_line_count: usize,
    reused: usize,
}

impl<'a> RowReuse<'a> {
    pub(super) fn new(
        previous: Option<(&'a LayoutRows, &'a [usize])>,
        settings: ShapeSettings,
        visible_count: usize,
        buffer_line_count: usize,
    ) -> Self {
        let old = previous.and_then(|(lines, indices)| match lines {
            LayoutRows::Cached(rows) if rows.settings == settings => Some((rows, indices)),
            _ => None,
        });
        let output = (!old.is_some_and(|(rows, _)| rows.lines.len() == visible_count))
            .then(|| Vec::with_capacity(visible_count));
        Self {
            old,
            output,
            settings,
            visible_count,
            buffer_line_count,
            reused: 0,
        }
    }

    pub(super) fn line<'r>(
        &mut self,
        index: usize,
        row: usize,
        text: RopeSlice<'_>,
        wraps: &SmallVec<[Range<usize>; 1]>,
        indent: u32,
        runs: impl Iterator<Item = (usize, &'r TextRun)> + Clone,
        shape: impl FnOnce(&str, &[TextRun]) -> LineLayout,
    ) {
        let matches = |cached: &&Rc<CachedLine>| {
            cached.indent == indent
                && &cached.wraps == wraps
                && text.len() == cached.text.len()
                && runs_match(runs.clone(), &cached.runs)
                && text.chunks().flat_map(str::bytes).eq(cached.text.bytes())
        };
        let prior = self.old.and_then(|(old, indices)| {
            indices
                .binary_search(&row)
                .ok()
                .and_then(|i| old.lines.get(i))
                .filter(matches)
                .or_else(|| {
                    // A multiline edit may shift a still-visible line to a different row.
                    (old.buffer_line_count != self.buffer_line_count)
                        .then(|| old.lines.iter().find(matches))
                        .flatten()
                })
        });
        if let Some(row) = prior {
            self.reused += 1;
            if self.output.is_none()
                && self
                    .old
                    .is_some_and(|(old, _)| Rc::ptr_eq(row, &old.lines[index]))
            {
                return;
            }
            let row = row.clone();
            self.output(index).push(row);
        } else {
            let text: String = text.into();
            let runs: Vec<_> = runs
                .map(|(len, run)| TextRun { len, ..run.clone() })
                .collect();
            let layout = shape(&text, &runs);
            self.output(index).push(Rc::new(CachedLine {
                text,
                wraps: wraps.clone(),
                indent,
                runs,
                layout,
            }));
        }
    }

    fn output(&mut self, index: usize) -> &mut Vec<Rc<CachedLine>> {
        self.output.get_or_insert_with(|| {
            let (previous, _) = self.old.expect("a reused prefix has a previous frame");
            let mut output = Vec::with_capacity(self.visible_count);
            output.extend(previous.lines[..index].iter().cloned());
            output
        })
    }

    pub(super) fn finish(self) -> (LayoutRows, usize) {
        let rows = self.output.map_or_else(
            || {
                self.old
                    .expect("unchanged viewport has a previous frame")
                    .0
                    .clone()
            },
            |lines| {
                Rc::new(CachedRows {
                    settings: self.settings,
                    lines,
                    buffer_line_count: self.buffer_line_count,
                })
            },
        );
        (LayoutRows::Cached(rows), self.reused)
    }
}

/// Monotonic cursor over the editor's visible run stream. A hit borrows the
/// clipped lengths and styles instead of constructing a per-line run vector.
pub(super) struct RunCursor<'a> {
    runs: &'a [TextRun],
    first: usize,
    offset: usize,
}

impl<'a> RunCursor<'a> {
    pub(super) fn new(runs: &'a [TextRun]) -> Self {
        Self {
            runs,
            first: 0,
            offset: 0,
        }
    }

    pub(super) fn line(
        &mut self,
        start: usize,
        len: usize,
    ) -> impl Iterator<Item = (usize, &'a TextRun)> + Clone + use<'a> {
        while let Some(run) = self.runs.get(self.first) {
            if self.offset + run.len > start {
                break;
            }
            self.offset += run.len;
            self.first += 1;
        }
        let mut offset = self.offset;
        let end = start + len;
        self.runs[self.first..]
            .iter()
            .scan((), move |_, run| {
                if offset >= end {
                    return None;
                }
                let length = end.min(offset + run.len).saturating_sub(start.max(offset));
                offset += run.len;
                Some((length, run))
            })
            .filter(|(length, _)| *length > 0)
    }
}

fn run_key(
    (len, run): (usize, &TextRun),
) -> (
    usize,
    &Font,
    Hsla,
    Option<Hsla>,
    Option<gpui::UnderlineStyle>,
    Option<gpui::StrikethroughStyle>,
) {
    (
        len,
        &run.font,
        run.color,
        run.background_color,
        run.underline,
        run.strikethrough,
    )
}

fn runs_match<'a>(
    mut incoming: impl Iterator<Item = (usize, &'a TextRun)>,
    saved: &[TextRun],
) -> bool {
    saved.iter().all(|run| {
        incoming
            .next()
            .is_some_and(|other| run_key(other) == run_key((run.len, run)))
    }) && incoming.next().is_none()
}
