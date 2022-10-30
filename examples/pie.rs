use mermaid::{pie, Pie};

fn main() {
    let chart = Pie::parse(
        r#"
pie showData
    title Key elements in Product X
    "Calcium" : 42.96
    "Potassium" : 50.05
    "Magnesium" : 10.01
    "Iron" :  5
    "#,
    )
    .unwrap();
    let mut style = (*pie::DEFAULT_STYLE).clone();
    style.background_color = piet::Color::WHITE;
    println!("{:#?}", chart);
    chart.to_svg_file("output.svg", Some(&style)).unwrap();
    chart.to_png_file("output.png", 4., Some(&style)).unwrap();
}
