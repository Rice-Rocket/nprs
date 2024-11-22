fn main() {
    match nprs::render() {
        Ok(_) => (),
        Err(err) => println!("error: {}", err),
    }
}
