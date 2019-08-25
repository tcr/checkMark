use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::io::Read;
use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum FormatVersion {
    V3,
    V5,
}

impl FormatVersion {
    fn line_header_5th_attribute(&self) -> bool {
        match self {
            FormatVersion::V3 => false,
            FormatVersion::V5 => true,
        }
    }
}

#[test]
fn test_read_number_i32() {
    let num = read_number_i32(&[42, 0, 0, 0]);
    assert_eq!(42, num);
}

pub fn read_number_i32(bytes: &[u8]) -> i32 {
    let mut rdr = Cursor::new(&bytes[0..4]);
    // TODO implement if let Some(...)
    let number = rdr.read_i32::<LittleEndian>().unwrap();
    number
}

pub fn read_number_f32(bytes: &[u8]) -> f32 {
    let mut rdr = Cursor::new(&bytes[0..4]);
    // TODO implement if let Some(...)
    let number = rdr.read_f32::<LittleEndian>().unwrap();
    number
}

pub fn parse_line_header(version: FormatVersion, cursor: &mut Cursor<&[u8]>) -> Option<(i32, i32, i32, f32)> {
    let mut brush_type = [0u8; 4];
    let mut color = [0u8; 4];
    let mut unknown_line_attribute = [0u8; 4];
    let mut brush_base_size = [0u8; 4];
    let mut unknown_line_attribute_2 = [0u8; 4];

    cursor.read_exact(&mut brush_type).ok()?;
    cursor.read_exact(&mut color).ok()?;
    cursor.read_exact(&mut unknown_line_attribute).ok()?;
    cursor.read_exact(&mut brush_base_size).ok()?;
    if version.line_header_5th_attribute() {
        cursor.read_exact(&mut unknown_line_attribute_2).ok()?;
    }

    // TODO verify range of values
    return Some((
        read_number_i32(&brush_type),
        read_number_i32(&color),
        read_number_i32(&unknown_line_attribute),
        read_number_f32(&brush_base_size),
    ));
}

pub fn parse_point_header(cursor: &mut Cursor<&[u8]>) -> Option<(f32, f32, f32, f32, f32, f32)> {
    let mut x = [0u8; 4];
    let mut y = [0u8; 4];
    let mut speed = [0u8; 4];
    let mut direction = [0u8; 4];
    let mut width = [0u8; 4];
    let mut pressure = [0u8; 4];

    cursor.read_exact(&mut x).ok()?;
    cursor.read_exact(&mut y).ok()?;
    cursor.read_exact(&mut speed).ok()?;
    cursor.read_exact(&mut direction).ok()?;
    cursor.read_exact(&mut width).ok()?;
    cursor.read_exact(&mut pressure).ok()?;

    return Some((
        read_number_f32(&x),
        read_number_f32(&y),
        read_number_f32(&speed),
        read_number_f32(&direction),
        read_number_f32(&width),
        read_number_f32(&pressure),
    ));
}

pub fn read_points(cursor: &mut Cursor<&[u8]>, _max_size_file: usize) -> Vec<Point> {
    let mut points = Vec::<Point>::default();
    let mut num_points = [0u8; 4];
    if let Ok(()) = cursor.read_exact(&mut num_points) {
        // eprintln!("points: {}", read_number_i32(&num_points));
        for _pt in 0..read_number_i32(&num_points) {
            if let Some(tuple) = parse_point_header(cursor) {
                let new_point = Point::new(tuple);
                points.push(new_point);
            } else {
                break;
            }
        }
    }
    points
}

pub fn read_lines(version: FormatVersion, cursor: &mut Cursor<&[u8]>, _max_size_file: usize) -> Vec<Line> {
    let mut lines = vec![];
    let mut num_lines = [0u8; 4];
    if let Ok(()) = cursor.read_exact(&mut num_lines) {
        let line_count = read_number_i32(&num_lines);
        for _li in 0..line_count {
            // eprintln!("li: {} / {}", _li, line_count);
            if let Some(tuple) = parse_line_header(version, cursor) {
                let new_line = Line::new(tuple, read_points(cursor, _max_size_file));
                lines.push(new_line);
            } else {
                break;
            }
        }
    }
    lines
}

pub fn read_layers(version: FormatVersion, cursor: &mut Cursor<&[u8]>, _max_size_file: usize) -> Vec<Layer> {
    let mut layers = vec![];
    let mut num_layers = [0u8; 4];
    if let Ok(()) = cursor.read_exact(&mut num_layers) {
        let layer_count = read_number_i32(&num_layers);
        for _l in 0..layer_count {
            // eprintln!("l: {} / {}", _l, layer_count);
            let new_layer = Layer {
                lines: read_lines(version, cursor, _max_size_file),
            };
            layers.push(new_layer);
        }
    }
    layers
}

pub fn read_pages(version: FormatVersion, content: &[u8], _max_size_file: usize) -> Vec<Page> {
    let mut cursor = Cursor::new(content);

    let mut pages = vec![];
    let num_pages = 1;
    // eprintln!("page: 0 / {}", num_pages);
    let new_page = Page {
        layers: read_layers(version, &mut cursor, _max_size_file),
    };
    pages.push(new_page);
    pages
}

pub fn read_document(line_file: &[u8], _max_size_file: usize) -> Vec<Page> {
    assert_eq!(&line_file[0..32], "reMarkable .lines file, version=".as_bytes());
    let version = match line_file[32] as char {
        '3' => FormatVersion::V3,
        '4' => FormatVersion::V3,
        '5' => FormatVersion::V5,
        v => {
            assert!(false, "Unexpected version number: {:?}", v);
            FormatVersion::V3
        }
    };

    // Read document content.
    let content = &line_file[43..];
    return read_pages(version, content, _max_size_file);
}
