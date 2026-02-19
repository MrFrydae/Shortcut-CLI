use std::error::Error;

use crate::project;

pub fn run() -> Result<(), Box<dyn Error>> {
    let (_root, canonical) = project::init()?;
    println!("Initialized sc for {}", canonical.display());
    Ok(())
}
