use mermaid::Pie;

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
    println!("{:#?}", chart);
    chart.to_svg_file("output.svg").unwrap();
    chart.to_png_file("output.png").unwrap();
}
