use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    noctalia_build::generate()
}
