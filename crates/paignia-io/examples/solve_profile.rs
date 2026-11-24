use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Vec::new();
    io::stdin()
        .take((paignia_io::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut request)?;
    io::stdout().write_all(&paignia_io::solve_profile_json(&request)?)?;
    Ok(())
}
