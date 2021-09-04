use anyhow::{anyhow, bail, Result};

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next();

    let file_path = args
        .next()
        .ok_or_else(|| anyhow!("file argument not provided"))?;
    let file = std::fs::read(&file_path)?;

    let mut output_file = Vec::with_capacity(file.len());
    let mut state = State::Normal;

    for byte in file {
        match (&state, byte) {
            (State::Normal, b'\\') => {
                state = State::Escape;
            }
            (State::Normal, _) => {
                output_file.push(byte);
            }
            (State::Escape, _) => {
                let output_byte = match byte {
                    b'0' => 0x0,
                    b'\'' | b'"' | b'_' | b'\\' | b'%' => byte,
                    b'b' => 0x8,
                    b'n' => 0xa,
                    b'r' => 0xd,
                    b't' => 0x9,
                    b'Z' => 0x1a,
                    _ => bail!("Invalid escape character: {}", byte),
                };

                output_file.push(output_byte);
                state = State::Normal;
            }
        }
    }

    std::fs::write(file_path, output_file)?;

    Ok(())
}

enum State {
    Normal,
    Escape,
}
