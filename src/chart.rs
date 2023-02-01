use std::fmt::Debug;

use plotters::coord::ranged1d::AsRangedCoord;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::FontError;

const INDEX_TOP: usize = 0;
const INDEX_BOTTOM: usize = 1;
const INDEX_LEFT: usize = 2;
const INDEX_RIGHT: usize = 3;

type DrawingResult<T, DB> = Result<T, DrawingAreaErrorKind<<DB as DrawingBackend>::ErrorType>>;

type ChartContext2d<'a, DB, X, Y> = ChartContext<
    'a,
    DB,
    Cartesian2d<<X as AsRangedCoord>::CoordDescType, <Y as AsRangedCoord>::CoordDescType>,
>;

/// Specifies layout of chart before creating [`DrawingArea`]
#[derive(Clone)]
pub struct ChartLayout<'a> {
    title_height: u32,
    title_content: Option<(String, TextStyle<'a>, u32)>,
    margin: [u32; 4],
    label_area_size: [u32; 4],
}

impl<'a> Debug for ChartLayout<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChartLayout")
            .field("title_height", &self.title_height)
            .field(
                "title_content",
                &self.title_content.as_ref().map(|(t, _, _)| t),
            )
            .field("margin", &self.margin)
            .field("label_area_size", &self.label_area_size)
            .finish()
    }
}

fn estimate_text_size(text: &str, font: &FontDesc) -> Result<(u32, u32), FontError> {
    let text_layout = font.layout_box(text)?;
    Ok((
        ((text_layout.1).0 - (text_layout.0).0) as u32,
        ((text_layout.1).1 - (text_layout.0).1) as u32,
    ))
}

impl<'a> ChartLayout<'a> {
    pub fn new() -> Self {
        Self {
            label_area_size: [0; 4],
            title_height: 0,
            title_content: None,
            margin: [0; 4],
        }
    }

    pub fn set_all_label_area_size(
        &mut self,
        top: u32,
        bottom: u32,
        left: u32,
        right: u32,
    ) -> &mut Self {
        self.label_area_size = [top, bottom, left, right];
        self
    }

    pub fn x_label_area_size(&mut self, size: u32) -> &mut Self {
        self.label_area_size[INDEX_BOTTOM] = size;
        self
    }

    pub fn y_label_area_size(&mut self, size: u32) -> &mut Self {
        self.label_area_size[INDEX_LEFT] = size;
        self
    }

    pub fn top_x_label_area_size(&mut self, size: u32) -> &mut Self {
        self.label_area_size[INDEX_TOP] = size;
        self
    }

    pub fn right_y_label_area_size(&mut self, size: u32) -> &mut Self {
        self.label_area_size[INDEX_RIGHT] = size;
        self
    }

    pub fn set_all_margin(&mut self, top: u32, bottom: u32, left: u32, right: u32) -> &mut Self {
        self.margin = [top, bottom, left, right];
        self
    }

    pub fn margin(&mut self, size: u32) -> &mut Self {
        self.margin = [size, size, size, size];
        self
    }

    pub fn margin_top(&mut self, size: u32) -> &mut Self {
        self.margin[INDEX_TOP] = size;
        self
    }

    pub fn margin_bottom(&mut self, size: u32) -> &mut Self {
        self.margin[INDEX_BOTTOM] = size;
        self
    }

    pub fn margin_left(&mut self, size: u32) -> &mut Self {
        self.margin[INDEX_LEFT] = size;
        self
    }

    pub fn margin_right(&mut self, size: u32) -> &mut Self {
        self.margin[INDEX_RIGHT] = size;
        self
    }

    // Clears caption text and area information for caption
    pub fn no_caption(&mut self) -> &mut Self {
        self.title_height = 0;
        self.title_content = None;
        self
    }

    /// Sets new caption text and calculates caption area above the chart.
    pub fn caption(
        &mut self,
        text: impl Into<String>,
        font: impl Into<FontDesc<'a>>,
    ) -> Result<&mut Self, FontError> {
        let text: String = text.into();
        let font: FontDesc = font.into();
        let (_, text_h) = estimate_text_size(&text, &font)?;
        let style: TextStyle = font.into();
        let y_padding = (text_h / 2).min(5);
        self.title_height = y_padding * 2 + text_h;
        self.title_content = Some((text, style, y_padding));
        Ok(self)
    }

    /// Replaces caption test, without updating layout.
    ///
    /// It is needed to call [`caption()`](Self::caption) in order to make caption visible.
    pub fn replace_caption(&mut self, text: impl Into<String>) -> &mut Self {
        let text: String = text.into();
        if let Some((_, style, y_padding)) = self.title_content.take() {
            self.title_content = Some((text, style, y_padding));
        }
        self
    }

    fn additional_sizes(&self) -> (u32, u32) {
        let [m_top, m_bottom, m_left, m_right] = self.margin;
        let [l_top, l_bottom, l_left, l_right] = self.label_area_size;
        let width = m_left + m_right + l_left + l_right;
        let height = self.title_height + m_top + m_bottom + l_top + l_bottom;
        (width, height)
    }

    /// Size of root area whose plotting area will be equal to `plot_size`.
    ///
    /// An [`DrawingArea`] with returned size should be given for [`bind()`](Self::bind).
    pub fn desired_image_size(&self, plot_size: (u32, u32)) -> (u32, u32) {
        let additional = self.additional_sizes();
        (plot_size.0 + additional.0, plot_size.1 + additional.1)
    }

    /// Estimates required root-area height from its width and the aspect ratio of the plotting area.
    ///
    /// `aspect_ratio` is the ratio of plotting-area height to its width.
    pub fn desired_image_height_from_width(&self, image_width: u32, aspect_ratio: f64) -> u32 {
        let additional = self.additional_sizes();
        if image_width < additional.0 {
            additional.1
        } else {
            ((image_width - additional.0) as f64 * aspect_ratio) as u32 + additional.1
        }
    }

    /// Bind layout information to an actual root area.
    pub fn bind<'b, DB>(
        &self,
        root_area: &'b DrawingArea<DB, Shift>,
    ) -> DrawingResult<ChartLayoutBuilder<'b, DB>, DB>
    where
        'a: 'b,
        DB: DrawingBackend,
    {
        use plotters::style::text_anchor::{HPos, Pos, VPos};

        let title_area_height = self.title_height;
        let main_area = if title_area_height > 0 {
            let (title_area, main_area) = root_area.split_vertically(title_area_height);
            if let Some((text, style, y_padding)) = &self.title_content {
                let dim = title_area.dim_in_pixel();
                let x_padding = dim.0 / 2;
                let style = &style.pos(Pos::new(HPos::Center, VPos::Top));
                title_area.draw_text(text, style, (x_padding as i32, *y_padding as i32))?;
                main_area
            } else {
                main_area
            }
        } else {
            root_area.clone()
        };
        Ok(ChartLayoutBuilder {
            layout: self.clone(),
            main_area,
        })
    }
}

impl<'a> Default for ChartLayout<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ChartLayoutBuilder<'a, DB: DrawingBackend> {
    layout: ChartLayout<'a>,
    main_area: DrawingArea<DB, Shift>,
}

impl<'a, DB: DrawingBackend> ChartLayoutBuilder<'a, DB> {
    /// Estimates size of the plotting area in pixels.
    ///
    /// Can be used to determine plotting value range to pass to [`build_cartesian_2d`](Self::build_cartesian_2d).
    pub fn estimate_plot_area_size(&self) -> (u32, u32) {
        let [m_top, m_bottom, m_left, m_right] = self.layout.margin;
        let [l_top, l_bottom, l_left, l_right] = self.layout.label_area_size;
        // main_area does not include caption part
        let (image_width, image_height) = self.main_area.dim_in_pixel();
        let plot_width = image_width - (m_left + m_right + l_left + l_right);
        let plot_height = image_height - (m_top + m_bottom + l_top + l_bottom);
        (plot_width, plot_height)
    }

    pub fn build_cartesian_2d<X: AsRangedCoord, Y: AsRangedCoord>(
        &self,
        x_spec: X,
        y_spec: Y,
    ) -> DrawingResult<ChartContext2d<DB, X, Y>, DB> {
        let [m_top, m_bottom, m_left, m_right] = self.layout.margin;
        let [l_top, l_bottom, l_left, l_right] = self.layout.label_area_size;

        let mut builder = ChartBuilder::on(&self.main_area);

        builder
            .margin_top(m_top)
            .margin_bottom(m_bottom)
            .margin_left(m_left)
            .margin_right(m_right)
            .set_label_area_size(LabelAreaPosition::Top, l_top)
            .set_label_area_size(LabelAreaPosition::Bottom, l_bottom)
            .set_label_area_size(LabelAreaPosition::Left, l_left)
            .set_label_area_size(LabelAreaPosition::Right, l_right);

        builder.build_cartesian_2d(x_spec, y_spec)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::ops::Range;

    use plotters::backend::RGBPixel;
    use plotters::prelude::*;

    use super::ChartLayout;

    #[test]
    fn size_estimation() -> Result<(), Box<dyn Error>> {
        let x_spec = 0.0..2.0;
        let y_spec = -1.0..2.0;
        let plot_size = (200, 350);
        let mut layout = ChartLayout::new();

        for i in 0..0x200 {
            layout.set_all_margin(
                if i & 0x1 == 0 { 5 } else { 0 },
                if i & 0x2 == 0 { 10 } else { 0 },
                if i & 0x4 == 0 { 12 } else { 0 },
                if i & 0x8 == 0 { 15 } else { 0 },
            );
            layout.set_all_label_area_size(
                if i & 0x10 == 0 { 20 } else { 0 },
                if i & 0x20 == 0 { 25 } else { 0 },
                if i & 0x40 == 0 { 30 } else { 0 },
                if i & 0x80 == 0 { 32 } else { 0 },
            );
            if i & 0x100 == 0 {
                layout.caption("Test Title", ("sans-serif", 20))?;
            } else {
                layout.no_caption();
            }
            bmp2d_size_estimation(&layout, plot_size, x_spec.clone(), y_spec.clone())?;
        }

        Ok(())
    }

    fn bmp2d_size_estimation(
        layout: &ChartLayout,
        plot_size: (u32, u32),
        x_spec: Range<f64>,
        y_spec: Range<f64>,
    ) -> Result<(), Box<dyn Error>> {
        let image_size = layout.desired_image_size(plot_size);

        let mut buf = vec![0u8; (3 * image_size.0 * image_size.1) as usize];
        let backend: BitMapBackend<RGBPixel> =
            BitMapBackend::with_buffer_and_format(&mut buf, image_size)?;
        let root_area = backend.into_drawing_area();

        let builder = layout.bind(&root_area)?;
        let estimated_plot_size = builder.estimate_plot_area_size();
        assert_eq!(
            plot_size, estimated_plot_size,
            "wrong estimation; layout = {layout:?}, image_size = {image_size:?}"
        );

        let chart = builder.build_cartesian_2d(x_spec, y_spec)?;
        let actual_size = chart.plotting_area().dim_in_pixel();

        assert_eq!(
            plot_size, actual_size,
            "wrong actual size, layout = {layout:?}, image_size = {image_size:?}"
        );
        Ok(())
    }
}
