
fn main() {

    let json_str = r#"{"name":"jonas"}"#;

    let parsed = json_parser::parse(json_str);

    println!( "{:?}", parsed.as_ref() );
    
}
