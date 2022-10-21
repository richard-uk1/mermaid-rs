use mermaid::Flowchart;

fn main() {
    let chart = Flowchart::parse(
        r"
        flowchart TB
        A --> C
        A --> D
    ",
    )
    .unwrap();
    println!("{:#?}", chart);
}
