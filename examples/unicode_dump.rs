//! Comprobador de mantenimiento: emite únicamente escalares cuya normalización cambia.
use pcge_peru::normalize_for_search;
use std::io::{self, Write};
fn main() -> io::Result<()> {
    let mut out = io::BufWriter::new(io::stdout().lock());
    for cp in 0..=0x10ffffu32 {
        if let Some(c) = char::from_u32(cp) {
            let input = c.to_string();
            let normalized = normalize_for_search(&input);
            if normalized != input {
                write!(out, "{cp:x}\t")?;
                for byte in normalized.as_bytes() { write!(out, "{byte:02x}")?; }
                writeln!(out)?;
            }
        }
    }
    Ok(())
}
