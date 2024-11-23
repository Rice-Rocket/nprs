fn main() {
    match nprs::run_cli() {
        Ok(_) => (),
        Err(err) => println!("error: {}", err),
    }
}
