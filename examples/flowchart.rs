use mermaid::Flowchart;

fn main() {
    let chart = Flowchart::parse(
        r#"
        flowchart TB
            A[[ Some "inner quotes" text ]] ----> C & D === B[ "quoted )) text"]
            B <=x C
    "#,
    )
    .unwrap();
    println!("{:#?}", chart);
}
