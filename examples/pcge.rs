//! Ejemplo de consultas con salida JSON, sin dependencias de serialización.
use pcge_peru::{Catalog, Edition, Entry};
use std::{env, error::Error, io::{self, Write}};

fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"),
            '\u{0000}'..='\u{001f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push(c),
        }
    }
    out.push('"'); out
}
fn optional(text: Option<&str>) -> String { text.map(quoted).unwrap_or_else(|| "null".to_owned()) }
fn entry(entry: &Entry) -> String {
    format!("{{\"code\":{},\"name\":{},\"parent_code\":{}}}", quoted(entry.code), quoted(entry.name), optional(entry.parent_code))
}
fn codes(entries: &[&Entry]) -> String {
    format!("[{}]", entries.iter().map(|e| quoted(e.code)).collect::<Vec<_>>().join(","))
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let usage = "Uso: pcge {2019|2026} {dump|nav|metadata|source|anomalies}\n     pcge {2019|2026} {get|parent|children|ancestors|descendants|anomalies-for|search} VALOR";
    if args.len() < 2 || args.len() > 3 { return Err(usage.into()); }
    let catalog = Catalog::new(args[0].parse::<Edition>()?);
    let result = match (args[1].as_str(), args.get(2)) {
        ("dump", None) => format!("[{}]", catalog.entries().iter().map(entry).collect::<Vec<_>>().join(",")),
        ("nav", None) => {
            let mut records = Vec::new();
            for e in catalog.entries() {
                records.push(format!("{{\"code\":{},\"parent\":{},\"children\":{},\"ancestors\":{},\"descendants\":{}}}",
                    quoted(e.code), optional(e.parent_code), codes(&catalog.children(e.code)?),
                    codes(&catalog.ancestors(e.code)?), codes(&catalog.descendants(e.code)?)));
            }
            format!("[{}]", records.join(","))
        }
        ("metadata", None) => catalog.metadata_json().to_owned(),
        ("source", None) => catalog.provenance_json().to_owned(),
        ("anomalies", None) => catalog.anomalies_json().to_owned(),
        ("get", Some(code)) => entry(catalog.get(code).ok_or_else(|| format!("código no encontrado: {code}"))?),
        ("parent", Some(code)) => catalog.parent(code)?.map(entry).unwrap_or_else(|| "null".to_owned()),
        ("children", Some(code)) => codes(&catalog.children(code)?),
        ("ancestors", Some(code)) => codes(&catalog.ancestors(code)?),
        ("descendants", Some(code)) => codes(&catalog.descendants(code)?),
        ("search", Some(query)) => codes(&catalog.search(query)?),
        ("anomalies-for", Some(code)) => format!("[{}]", catalog.anomalies_for(code)?.iter().map(|a| quoted(a.id)).collect::<Vec<_>>().join(",")),
        _ => return Err(usage.into()),
    };
    writeln!(io::stdout().lock(), "{result}")?;
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); }
}
