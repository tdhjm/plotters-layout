//! # plotters-layout
//!
//! Layout utility library for [plotters](::plotters) crate.
//!
//! ## Creating a chart whose plotting area has specified size
//!
//! ```
//! use plotters::prelude::*;
//! use plotters_layout::ChartLayout;
//! use plotters::backend::{RGBPixel, PixelFormat};
//!
//! let mut layout = ChartLayout::new();
//! layout.caption("Graph Title", ("sans-serif", 40))?
//!     .margin(4)
//!     .x_label_area_size(40)
//!     .y_label_area_size(40);
//! let (w, h): (u32, u32) = layout.desired_image_size((200, 160));
//! let mut buf = vec![0u8; (w * h) as usize * RGBPixel::PIXEL_SIZE];
//! let graph = BitMapBackend::with_buffer(&mut buf, (w, h));
//! let root_area = graph.into_drawing_area();
//! let builder = layout.bind(&root_area)?;
//! let chart = builder.build_cartesian_2d(0f64..20f64, 0f64..16f64)?;
//! assert_eq!(chart.plotting_area().dim_in_pixel(), (200, 160));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Adjusting aspect ratio
//!
//! ```
//! use plotters::prelude::*;
//! use plotters_layout::{centering_ranges, ChartLayout};
//!
//! let min_range = (-200f64..200f64, -100f64..100f64);
//!
//! let mut buf = String::new();
//! let graph = SVGBackend::with_string(&mut buf, (1280, 720));
//! let root_area = graph.into_drawing_area();
//!
//! let mut builder = ChartLayout::new()
//!     .caption("Graph Title", ("sans-serif", 40))?
//!     .margin(4)
//!     .x_label_area_size(40)
//!     .y_label_area_size(40)
//!     .bind(&root_area)?;
//!
//! let (width, height) = builder.estimate_plot_area_size();
//! let (x_range, y_range) = centering_ranges(&min_range, &(width as f64, height as f64));
//!
//! // (x_range, y_range) and (width, height) has same aspect ratio
//! let inner_ratio = (x_range.end - x_range.start) / (y_range.end - y_range.start);
//! let outer_ratio = width as f64 / height as f64;
//! assert!((inner_ratio - outer_ratio).abs() < 1e-8);
//!
//! let chart = builder.build_cartesian_2d(x_range, y_range)?;
//! assert_eq!(chart.plotting_area().dim_in_pixel(), (width, height));
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod chart;

use std::ops::{Add, Div, Mul, Range, Sub};

pub use crate::chart::*;

pub fn centering_ranges<T, S>(
    minimum: &(Range<T>, Range<T>),
    destination: &(S, S),
) -> (Range<T>, Range<T>)
where
    T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    S: Copy + Into<T>,
{
    let sx = minimum.0.end - minimum.0.start;
    let sy = minimum.1.end - minimum.1.start;
    let dx = destination.0.into();
    let dy = destination.1.into();
    let half = dx / (dx + dx); // == 0.5
    if sx * dy < sy * dx {
        // sx -> sy * dx / dy
        let radius = sy * dx / dy * half;
        let center = (minimum.0.start + minimum.0.end) * half;
        let s0 = (center - radius)..(radius + center);
        (s0, minimum.1.clone())
    } else {
        // sy -> sx * dy / dx
        let radius = sx * dy / dx * half;
        let center = (minimum.1.end + minimum.1.start) * half;
        let s1 = (center - radius)..(radius + center);
        (minimum.0.clone(), s1)
    }
}
