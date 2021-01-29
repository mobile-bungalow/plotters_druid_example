use druid::widget::Flex;
use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use plotters::prelude::*;
use plotters_druid::*;

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use rand_xorshift::XorShiftRng;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder).window_size((750., 750.));
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(())
}

fn ui_builder() -> impl Widget<()> {
    let plot = Box::new(Plot);
    Flex::column().with_child(plot as Box<dyn PlottingProgram<()>>)
}

pub struct Plot;

impl PlottingProgram<()> for Plot {
    fn layout_plot(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        _bc: &druid::BoxConstraints,
        _data: &(),
        _env: &druid::Env,
    ) -> druid::Size {
        let size = druid::Size::new(750.0, 750.0);
        size
    }

    fn draw_plot<'a, 'b, 'c>(
        &mut self,
        ctx: plotters_druid::PlottingCtx<'a, 'b, 'c, '_>,
        _data: &(),
        _env: &druid::Env,
    ) {
        let root_drawing_area = ctx.into_drawing_area();
        root_drawing_area.fill(&WHITE).unwrap();

        let sd = 0.13;

        let random_points: Vec<(f64, f64)> = {
            let norm_dist = Normal::new(0.5, sd).unwrap();
            let mut x_rand = XorShiftRng::from_seed(*b"MyFragileSeed123");
            let mut y_rand = XorShiftRng::from_seed(*b"MyFragileSeed321");
            let x_iter = norm_dist.sample_iter(&mut x_rand);
            let y_iter = norm_dist.sample_iter(&mut y_rand);
            x_iter.zip(y_iter).take(5000).collect()
        };

        let areas = root_drawing_area.split_by_breakpoints([944], [80]);

        let mut x_hist_ctx = ChartBuilder::on(&areas[0])
            .y_label_area_size(40)
            .build_cartesian_2d((0.0..1.0).step(0.01).use_round().into_segmented(), 0..250)
            .unwrap();

        let mut y_hist_ctx = ChartBuilder::on(&areas[3])
            .x_label_area_size(40)
            .build_cartesian_2d(0..250, (0.0..1.0).step(0.01).use_round())
            .unwrap();

        let mut scatter_ctx = ChartBuilder::on(&areas[2])
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0f64..1f64, 0f64..1f64)
            .unwrap();

        scatter_ctx
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()
            .unwrap();

        scatter_ctx
            .draw_series(
                random_points
                    .iter()
                    .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
            )
            .unwrap();
        let x_hist = Histogram::vertical(&x_hist_ctx)
            .style(GREEN.filled())
            .margin(0)
            .data(random_points.iter().map(|(x, _)| (*x, 1)));

        x_hist_ctx.draw_series(x_hist).unwrap();
    }
}
