// Tablas Unicode fijadas; no depende de las tablas Unicode del compilador.
mod generated {
    include!("unicode_data.rs");
}
pub(crate) use generated::VERSION;
pub(crate) fn is_whitespace(c: char) -> bool {
    generated::WHITESPACE.binary_search(&(c as u32)).is_ok()
}
pub(crate) fn normalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match generated::MAPPING.binary_search_by_key(&(c as u32), |(cp, _)| *cp) {
            Ok(index) => result.push_str(generated::MAPPING[index].1),
            Err(_) => result.push(c),
        }
    }
    result
}
