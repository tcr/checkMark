use pdf_canvas::graphicsstate::{Color, Matrix, CapStyle, JoinStyle};
use pdf_canvas::Pdf;
use crate::*;

const BASE_LINE_WIDTH: f32 = 1.;
const RANGE_LINE_WIDTH: f32 = 2.;

pub fn render(path: &str, pages: &[Page]) {
    // Open our pdf document.
    let mut document = Pdf::create(path).expect("Create PDF file");

    // Only one page to consider.
    let page = &pages[0];

    // Render a page from our pages struct lines.
    document
        .render_page(1404.0, 1872.0, |c| {
            // Inverse Y coordinate system.
            c.concat(Matrix::scale(1., -1.))?;
            c.concat(Matrix::translate(0., -1872.))?;

            c.set_stroke_color(Color::gray(0))?;

            for layer in &page.layers {
                for line in &layer.lines {
                    if line.points.is_empty() {
                        continue;
                    }
                    let first_point = &line.points[0];
                    c.move_to(first_point.x, first_point.y)?;
                    for point in &line.points {
                        c.set_line_width(BASE_LINE_WIDTH + (point.pressure * RANGE_LINE_WIDTH))?;
                        c.set_line_cap_style(CapStyle::Round)?;
                        c.set_line_join_style(JoinStyle::Round)?;
                        c.line_to(point.x, point.y,)?;
                    }
                    c.stroke()?;
                }
            }

            Ok(())
        })
        .unwrap();
    document.finish().unwrap();
}
