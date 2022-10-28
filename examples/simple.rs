use mermaid::style::Color;
use std::{fs, io};

fn main() {
    let file = io::BufWriter::new(fs::File::create("output.svg").unwrap());
    let simple =
        mermaid::simple::Circle::new(kurbo::Point::new(20., 20.), 10., Color::rgb(0., 1., 1.));
    simple.to_svg(file).unwrap();
}
