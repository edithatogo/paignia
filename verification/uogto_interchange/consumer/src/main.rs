use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    io::stdin()
        .take((paignia_io::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut input)?;
    let request: serde_json::Value = serde_json::from_slice(&input)?;
    let output = paignia_io::solve_static_json(&input)?;
    let result: serde_json::Value = serde_json::from_slice(&output)?;
    for field in ["source", "version", "specification", "instance", "run"] {
        if result[field].is_null() || result[field] != request[field] {
            return Err(format!("invalid consumer result binding for {field}").into());
        }
    }
    if result["status"] != "completed" || result["method"] != "backward_induction" {
        return Err("expected a completed backward-induction policy".into());
    }
    io::stdout().write_all(&output)?;
    Ok(())
}
