//! Bounded local vector drawing through GPUI's native canvas element.

use std::{cell::Cell, error::Error, fmt, rc::Rc};

use gpui::{
    AnyElement, Bounds, ContentMask, Hsla, IntoElement as _, PaintQuad, Path, Pixels, Styled as _,
    canvas, point, px, quad, size,
};

use crate::view::{EmptyView, RenderContext, View};

/// A finite canvas extent in logical pixels. Zero is valid for a collapsed parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasSize {
    width: f32,
    height: f32,
}

impl CanvasSize {
    pub fn new(width: f32, height: f32) -> Result<Self, CanvasError> {
        if [width, height]
            .into_iter()
            .all(|value| value.is_finite() && value >= 0.0)
        {
            Ok(Self { width, height })
        } else {
            Err(CanvasError::InvalidSize)
        }
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn height(self) -> f32 {
        self.height
    }
}

/// A local point in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}

impl CanvasPoint {
    pub fn new(x: f32, y: f32) -> Result<Self, CanvasError> {
        if valid_coordinate(x) && valid_coordinate(y) {
            Ok(Self { x, y })
        } else {
            Err(CanvasError::InvalidGeometry)
        }
    }
}

/// A local rectangle in logical pixels. Its width and height must be positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CanvasRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Result<Self, CanvasError> {
        if !valid_coordinate(x)
            || !valid_coordinate(y)
            || !valid_extent(width)
            || !valid_extent(height)
            || !valid_coordinate_sum(x, width)
            || !valid_coordinate_sum(y, height)
        {
            return Err(CanvasError::InvalidGeometry);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasError {
    InvalidSize,
    InvalidGeometry,
    InvalidColor,
    NativeBounds,
}

impl fmt::Display for CanvasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize => formatter.write_str("canvas size must be finite and nonnegative"),
            Self::InvalidGeometry => {
                formatter.write_str("canvas geometry is invalid or out of range")
            }
            Self::InvalidColor => {
                formatter.write_str("canvas HSLA channels must be finite in 0..=1")
            }
            Self::NativeBounds => {
                formatter.write_str("native canvas bounds cannot represent drawing coordinates")
            }
        }
    }
}

impl Error for CanvasError {}

#[derive(Clone)]
enum CanvasCommand {
    FillRect(CanvasRect, Hsla),
    StrokeRect(CanvasRect, f32, Hsla),
    Line([CanvasPoint; 4], Hsla),
}

/// Immutable drawing commands, cheaply cloned across component renders.
/// Commands use local logical coordinates and preserve insertion paint order.
#[derive(Clone)]
pub struct CanvasScene {
    commands: Rc<[CanvasCommand]>,
    reported_native_error: Rc<Cell<bool>>,
}

impl CanvasScene {
    pub fn builder() -> CanvasSceneBuilder {
        CanvasSceneBuilder::default()
    }
}

#[derive(Default)]
pub struct CanvasSceneBuilder {
    commands: Vec<CanvasCommand>,
}

impl CanvasSceneBuilder {
    pub fn fill_rect(&mut self, rect: CanvasRect, color: Hsla) -> Result<&mut Self, CanvasError> {
        validate_rect(rect)?;
        validate_color(color)?;
        Ok(self.push(CanvasCommand::FillRect(rect, color)))
    }

    pub fn stroke_rect(
        &mut self,
        rect: CanvasRect,
        width: f32,
        color: Hsla,
    ) -> Result<&mut Self, CanvasError> {
        validate_rect(rect)?;
        if !valid_extent(width) || width > rect.width.min(rect.height) / 2.0 {
            return Err(CanvasError::InvalidGeometry);
        }
        validate_color(color)?;
        Ok(self.push(CanvasCommand::StrokeRect(rect, width, color)))
    }

    pub fn stroke_line(
        &mut self,
        from: CanvasPoint,
        to: CanvasPoint,
        width: f32,
        color: Hsla,
    ) -> Result<&mut Self, CanvasError> {
        if !valid_extent(width) {
            return Err(CanvasError::InvalidGeometry);
        }
        validate_color(color)?;
        Ok(self.push(CanvasCommand::Line(line_corners(from, to, width)?, color)))
    }

    pub fn build(self) -> CanvasScene {
        CanvasScene {
            commands: self.commands.into(),
            reported_native_error: Rc::new(Cell::new(false)),
        }
    }

    fn push(&mut self, command: CanvasCommand) -> &mut Self {
        self.commands.push(command);
        self
    }
}

/// A bounded, noninteractive native drawing leaf. Drawings outside its bounds
/// are clipped to the canvas rectangle; layout and display scaling remain GPUI's.
pub struct CanvasView {
    size: CanvasSize,
    scene: CanvasScene,
}

#[allow(non_snake_case)]
pub fn Canvas(size: CanvasSize, scene: CanvasScene) -> CanvasView {
    CanvasView::new(size, scene)
}

impl CanvasView {
    pub fn new(size: CanvasSize, scene: CanvasScene) -> Self {
        Self { size, scene }
    }
}

impl View for CanvasView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _context: &mut RenderContext<'_>) -> AnyElement {
        let scene = self.scene.clone();
        let reported_native_error = scene.reported_native_error.clone();
        canvas(
            move |bounds, _, _| prepare(&scene, bounds),
            move |bounds, prepared, window, _| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| match prepared {
                    Ok(commands) => {
                        for command in commands {
                            match command {
                                PreparedCommand::Quad(quad) => window.paint_quad(quad),
                                PreparedCommand::Path(path, color) => {
                                    window.paint_path(path, color)
                                }
                            }
                        }
                    }
                    Err(error) => {
                        if !reported_native_error.replace(true) {
                            eprintln!("RustUI canvas could not paint: {error}");
                        }
                    }
                });
            },
        )
        .w(px(self.size.width))
        .h(px(self.size.height))
        .flex_shrink_0()
        .into_any_element()
    }
}

enum PreparedCommand {
    Quad(PaintQuad),
    Path(Path<Pixels>, Hsla),
}

fn prepare(
    scene: &CanvasScene,
    bounds: Bounds<Pixels>,
) -> Result<Vec<PreparedCommand>, CanvasError> {
    let mut prepared = Vec::with_capacity(scene.commands.len());
    for command in scene.commands.iter() {
        match command {
            CanvasCommand::FillRect(rect, color) => {
                prepared.push(PreparedCommand::Quad(gpui::fill(
                    absolute_rect(*rect, bounds)?,
                    *color,
                )));
            }
            CanvasCommand::StrokeRect(rect, width, color) => {
                prepared.push(PreparedCommand::Quad(quad(
                    absolute_rect(*rect, bounds)?,
                    px(0.0),
                    gpui::transparent_black(),
                    px(*width),
                    *color,
                    Default::default(),
                )));
            }
            CanvasCommand::Line(corners, color) => {
                let a = absolute_point(corners[0], bounds.origin)?;
                let b = absolute_point(corners[1], bounds.origin)?;
                let c = absolute_point(corners[2], bounds.origin)?;
                let d = absolute_point(corners[3], bounds.origin)?;
                if !line_span_representable(&[
                    CanvasPoint {
                        x: f32::from(a.x),
                        y: f32::from(a.y),
                    },
                    CanvasPoint {
                        x: f32::from(b.x),
                        y: f32::from(b.y),
                    },
                    CanvasPoint {
                        x: f32::from(c.x),
                        y: f32::from(c.y),
                    },
                    CanvasPoint {
                        x: f32::from(d.x),
                        y: f32::from(d.y),
                    },
                ]) {
                    return Err(CanvasError::NativeBounds);
                }
                let mut path = Path::new(a);
                path.push_triangle(
                    (a, b, c),
                    (point(0.0, 1.0), point(0.0, 1.0), point(0.0, 1.0)),
                );
                path.push_triangle(
                    (c, b, d),
                    (point(0.0, 1.0), point(0.0, 1.0), point(0.0, 1.0)),
                );
                prepared.push(PreparedCommand::Path(path, *color));
            }
        }
    }
    Ok(prepared)
}

fn absolute_rect(rect: CanvasRect, bounds: Bounds<Pixels>) -> Result<Bounds<Pixels>, CanvasError> {
    let origin = absolute_point(
        CanvasPoint {
            x: rect.x,
            y: rect.y,
        },
        bounds.origin,
    )?;
    let far = absolute_point(
        CanvasPoint {
            x: (f64::from(rect.x) + f64::from(rect.width)) as f32,
            y: (f64::from(rect.y) + f64::from(rect.height)) as f32,
        },
        bounds.origin,
    )?;
    let width = f32::from(far.x) - f32::from(origin.x);
    let height = f32::from(far.y) - f32::from(origin.y);
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(CanvasError::NativeBounds);
    }
    Ok(Bounds {
        origin,
        size: size(px(width), px(height)),
    })
}

fn absolute_point(
    local: CanvasPoint,
    origin: gpui::Point<Pixels>,
) -> Result<gpui::Point<Pixels>, CanvasError> {
    let x = (f64::from(local.x) + f64::from(origin.x)) as f32;
    let y = (f64::from(local.y) + f64::from(origin.y)) as f32;
    if x.is_finite() && y.is_finite() {
        Ok(point(px(x), px(y)))
    } else {
        Err(CanvasError::NativeBounds)
    }
}

fn line_corners(
    from: CanvasPoint,
    to: CanvasPoint,
    width: f32,
) -> Result<[CanvasPoint; 4], CanvasError> {
    for p in [from, to] {
        CanvasPoint::new(p.x, p.y)?;
    }
    let dx = f64::from(to.x) - f64::from(from.x);
    let dy = f64::from(to.y) - f64::from(from.y);
    let length = dx.hypot(dy);
    if length == 0.0 {
        return Err(CanvasError::InvalidGeometry);
    }
    let offset_x = -dy / length * f64::from(width) / 2.0;
    let offset_y = dx / length * f64::from(width) / 2.0;
    let corner = |point: CanvasPoint, sign: f64| {
        let x = (f64::from(point.x) + sign * offset_x) as f32;
        let y = (f64::from(point.y) + sign * offset_y) as f32;
        CanvasPoint::new(x, y)
    };
    let corners = [
        corner(from, 1.0)?,
        corner(from, -1.0)?,
        corner(to, 1.0)?,
        corner(to, -1.0)?,
    ];
    if corners[0] == corners[1]
        || corners[2] == corners[3]
        || corners[0] == corners[2]
        || !line_span_representable(&corners)
    {
        return Err(CanvasError::InvalidGeometry);
    }
    Ok(corners)
}

fn line_span_representable(corners: &[CanvasPoint; 4]) -> bool {
    for values in [corners.map(|point| point.x), corners.map(|point| point.y)] {
        let min = values.into_iter().fold(f32::INFINITY, f32::min);
        let max = values.into_iter().fold(f32::NEG_INFINITY, f32::max);
        let span = (f64::from(max) - f64::from(min)) as f32;
        if !span.is_finite() || span <= 0.0 {
            return false;
        }
    }
    true
}

fn valid_coordinate(value: f32) -> bool {
    value.is_finite()
}

fn valid_extent(value: f32) -> bool {
    value.is_finite() && value > 0.0
}

fn valid_coordinate_sum(start: f32, extent: f32) -> bool {
    let end = (f64::from(start) + f64::from(extent)) as f32;
    end.is_finite() && end > start
}

fn validate_rect(rect: CanvasRect) -> Result<(), CanvasError> {
    CanvasRect::new(rect.x, rect.y, rect.width, rect.height).map(|_| ())
}

fn validate_color(color: Hsla) -> Result<(), CanvasError> {
    if [color.h, color.s, color.l, color.a]
        .into_iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
    {
        Ok(())
    } else {
        Err(CanvasError::InvalidColor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color() -> Hsla {
        Hsla {
            h: 0.5,
            s: 0.5,
            l: 0.5,
            a: 1.0,
        }
    }

    #[test]
    fn rejects_nonfinite_and_overflowing_geometry() {
        assert!(CanvasSize::new(f32::NAN, 1.0).is_err());
        assert!(CanvasSize::new(-1.0, 1.0).is_err());
        assert_eq!(CanvasSize::new(f32::MAX, 0.0).unwrap().width(), f32::MAX);
        assert!(CanvasRect::new(f32::MAX, 0.0, f32::MAX, 1.0).is_err());
        assert!(CanvasPoint::new(f32::INFINITY, 0.0).is_err());
        assert!(
            line_corners(
                CanvasPoint::new(-f32::MAX, 0.0).unwrap(),
                CanvasPoint::new(f32::MAX, 0.0).unwrap(),
                1.0,
            )
            .is_err()
        );
        assert!(
            line_corners(
                CanvasPoint::new(0.0, 0.0).unwrap(),
                CanvasPoint::new(0.0, 0.0).unwrap(),
                1.0,
            )
            .is_err()
        );
    }

    #[test]
    fn scene_clones_share_commands_and_validate_colors() {
        let rect = CanvasRect::new(-10.0, 0.0, 20.0, 30.0).unwrap();
        let mut builder = CanvasScene::builder();
        builder.fill_rect(rect, color()).unwrap();
        let scene = builder.build();
        assert!(Rc::ptr_eq(&scene.commands, &scene.clone().commands));
        let invalid = Hsla {
            a: f32::NAN,
            ..color()
        };
        assert!(CanvasScene::builder().fill_rect(rect, invalid).is_err());
    }

    #[test]
    fn prepares_diagonal_native_path_and_clipped_rect() {
        let mut builder = CanvasScene::builder();
        builder
            .fill_rect(CanvasRect::new(-5.0, -5.0, 20.0, 20.0).unwrap(), color())
            .unwrap();
        builder
            .stroke_line(
                CanvasPoint::new(0.0, 0.0).unwrap(),
                CanvasPoint::new(10.0, 10.0).unwrap(),
                2.0,
                color(),
            )
            .unwrap();
        let scene = builder.build();
        let bounds = Bounds {
            origin: point(px(100.0), px(200.0)),
            size: size(px(10.0), px(10.0)),
        };
        let prepared = prepare(&scene, bounds).unwrap();
        assert_eq!(prepared.len(), 2);
        assert!(matches!(&prepared[0], PreparedCommand::Quad(_)));
        assert!(matches!(&prepared[1], PreparedCommand::Path(path, _) if path.vertices.len() == 6));
        let invalid_origin = Bounds {
            origin: point(px(f32::MAX), px(0.0)),
            size: size(px(10.0), px(10.0)),
        };
        assert!(matches!(
            prepare(&scene, invalid_origin),
            Err(CanvasError::NativeBounds)
        ));
    }
}
