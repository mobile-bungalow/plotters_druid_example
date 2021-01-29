use druid::piet::*;
use druid::*;
use plotters_backend::*;
use std::cell::RefMut;

pub struct PlottingCtx<'a, 'b, 'c, 'd>(
    std::cell::RefCell<&'d mut druid::PaintCtx<'a, 'b, 'c>>,
    &'d druid::Env,
);

impl<'a, 'b, 'c, 'd> PlottingCtx<'a, 'b, 'c, 'd> {
    fn paint_ctx(&self) -> RefMut<&'d mut druid::PaintCtx<'a, 'b, 'c>> {
        self.0.borrow_mut()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NoThrow;
impl std::fmt::Display for NoThrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoThrow")
    }
}
impl std::error::Error for NoThrow {}

fn to_color(color: plotters_backend::BackendColor) -> druid::Color {
    let (r, g, b) = color.rgb;
    let a = (255. * (color.alpha / 1.)) as u8;
    druid::Color::from_rgba32_u32(u32::from_be_bytes([r, g, b, a]))
}

impl<'a, 'b, 'c> DrawingBackend for PlottingCtx<'a, 'b, 'c, '_> {
    type ErrorType = NoThrow;

    fn get_size(&self) -> (u32, u32) {
        let sz = self.paint_ctx().size();
        (sz.width as u32, sz.height as u32)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (x, y) = (point.0 as f64, point.1 as f64);
        let color = to_color(color);
        self.paint_ctx()
            .fill(druid::Rect::new(x, y, x + 1., y + 1.), &color);
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let from = (from.0 as f64, from.1 as f64);
        let to = (to.0 as f64, to.1 as f64);
        let color = to_color(style.color());
        let width = style.stroke_width() as f64;
        let line = druid::kurbo::Line::new(from, to);
        self.paint_ctx().stroke(line, &color, width);
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (x, y) = (upper_left.0 as f64, upper_left.1 as f64);
        let (x_1, y_1) = (bottom_right.0 as f64, bottom_right.1 as f64);
        let color = to_color(style.color());
        if fill {
            self.paint_ctx()
                .fill(druid::Rect::new(x, y, x_1, y_1), &color);
        } else {
            let brush = self.paint_ctx().solid_brush(color);
            self.paint_ctx().stroke(
                druid::Rect::new(x, y, x_1, y_1),
                &brush,
                style.stroke_width() as f64,
            );
        }
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let color = to_color(style.color());
        let width = style.stroke_width() as f64;
        let path: Vec<_> = path.into_iter().collect();

        let mut out_path = druid::kurbo::BezPath::new();
        if !path.is_empty() {
            let point = path[0];
            let pt = (point.0 as f64, point.1 as f64);
            out_path.move_to(pt);
        }
        for point in path {
            let pt = (point.0 as f64, point.1 as f64);
            out_path.line_to(pt);
        }
        self.paint_ctx().stroke(out_path, &color, width);

        Ok(())
    }
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let pt = (center.0 as f64, center.1 as f64);
        let color = to_color(style.color());

        if fill {
            self.paint_ctx()
                .fill(druid::kurbo::Circle::new(pt, radius as f64), &color);
        } else {
            let brush = self.paint_ctx().solid_brush(color);
            self.paint_ctx()
                .fill(druid::kurbo::Circle::new(pt, radius as f64), &brush);
        }
        Ok(())
    }

    fn draw_text<TStyle: plotters_backend::BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let pt = (pos.0 as f64, pos.1 as f64);
        let color = to_color(style.color());

        let size = style.size();
        let family = style.family();

        let mut paint_ctx = self.paint_ctx();
        let text_builder = paint_ctx.text();

        let family = text_builder
            .font_family(family.as_str())
            .unwrap_or(druid::FontFamily::SERIF);

        let layout = text_builder
            .new_text_layout(text.to_owned())
            .font(family, size)
            .text_color(color)
            .build()
            .unwrap();

        let metrics = layout.size();

        paint_ctx.draw_text(
            &layout,
            (pt.0 - metrics.width / 2., pt.1 - metrics.height / 2.),
        );

        Ok(())
    }

    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        let mut layout = druid::TextLayout::<String>::from_text(text);
        let size = style.size();

        let family = style.family();

        let mut paint_ctx = self.paint_ctx();
        let text_builder = paint_ctx.text();

        let family = text_builder
            .font_family(family.as_str())
            .unwrap_or(druid::FontFamily::SERIF);

        layout.set_text_size(size);
        layout.set_font(druid::FontDescriptor::new(family));
        layout.rebuild_if_needed(paint_ctx.text(), self.1);
        let metrics = layout.size();

        Ok((metrics.width as u32, metrics.height as u32))
    }
}

pub trait PlottingProgram<T> {
    fn handle_event(
        &mut self,
        _ctx: &mut druid::EventCtx,
        _event: &druid::Event,
        _data: &mut T,
        _env: &druid::Env,
    ) {
    }

    fn handle_lifecycle(
        &mut self,
        _ctx: &mut druid::LifeCycleCtx,
        _event: &druid::LifeCycle,
        _data: &T,
        _env: &druid::Env,
    ) {
    }

    fn layout_plot(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        _bc: &druid::BoxConstraints,
        _data: &T,
        _env: &druid::Env,
    ) -> druid::Size {
        let size = Size::new(500.0, 500.0);
        size
    }

    fn update_self(
        &mut self,
        _ctx: &mut druid::UpdateCtx,
        _old_data: &T,
        _data: &T,
        _env: &druid::Env,
    ) {
    }

    fn draw_plot<'a, 'b, 'c>(
        &mut self,
        _ctx: PlottingCtx<'a, 'b, 'c, '_>,
        _data: &T,
        _env: &druid::Env,
    ) {
    }
}

impl<T> druid::Widget<T> for Box<dyn PlottingProgram<T>> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        self.handle_event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.handle_lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        self.update_self(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        self.layout_plot(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.draw_plot(PlottingCtx(std::cell::RefCell::new(ctx), env), data, env);
    }
}
