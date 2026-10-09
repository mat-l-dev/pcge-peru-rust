//! Catálogos PCGE de Perú, inmutables y sin acceso a red.
//!
//! La edición siempre es explícita. Los códigos son texto y no se corrigen las
//! anomalías de la fuente. Las relaciones conservan el orden documental.
//!
//! ```
//! use pcge_peru::{Catalog, Edition};
//! let catalog = Catalog::new(Edition::V2026);
//! assert_eq!(catalog.len(), 1636);
//! assert_eq!(catalog.get("101").unwrap().name, "Caja");
//! assert_eq!(catalog.parent("101").unwrap().unwrap().code, "10");
//! assert!(!catalog.search("depósitos").unwrap().is_empty());
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{error, fmt, str::FromStr};

mod data;
mod unicode;

/// Versión de la biblioteca; independiente de la edición del catálogo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Versión Unicode fijada para búsqueda, independiente de la versión de Rust.
pub const UNICODE_VERSION: &str = unicode::VERSION;
/// Commit exacto del proyecto de origen.
pub const SOURCE_COMMIT: &str = "e9e69ed9075c09e8d1e5475760e5b691aa73b251";

/// Edición documental. No se elige una edición por fecha ni implícitamente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    /// Catálogo modificado de 2019.
    V2019,
    /// Catálogo de 2026.
    V2026,
}
impl Edition {
    /// Identificador de la edición.
    pub const fn as_str(self) -> &'static str {
        match self { Self::V2019 => "2019", Self::V2026 => "2026" }
    }
}
impl fmt::Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}
impl FromStr for Edition {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "2019" => Ok(Self::V2019), "2026" => Ok(Self::V2026),
            _ => Err(Error::UnsupportedEdition(value.to_owned())),
        }
    }
}

/// Nivel definido por el PCGE para códigos de uno a cinco dígitos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Elemento (un dígito).
    Element,
    /// Cuenta (dos dígitos).
    Account,
    /// Subcuenta (tres dígitos).
    Subaccount,
    /// Divisionaria (cuatro dígitos).
    Divisionary,
    /// Subdivisionaria (cinco dígitos).
    Subdivisionary,
}
/// Registro canónico. Las cadenas UTF-8 pertenecen a la biblioteca.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// Código textual, sin conversión numérica.
    pub code: &'static str,
    /// Nombre de la fuente canónica, sin normalizar.
    pub name: &'static str,
    /// Código padre; `None` para los elementos raíz.
    pub parent_code: Option<&'static str>,
}
impl Entry {
    /// Longitud del código ASCII.
    pub fn code_length(&self) -> usize { self.code.len() }
    /// Los códigos de seis dígitos no tienen un nivel PCGE inventado.
    pub fn level(&self) -> Option<Level> {
        match self.code.len() {
            1 => Some(Level::Element), 2 => Some(Level::Account),
            3 => Some(Level::Subaccount), 4 => Some(Level::Divisionary),
            5 => Some(Level::Subdivisionary), _ => None,
        }
    }
}
/// Metadatos del conjunto canónico.
#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    /// Edición identificada en el conjunto de origen.
    pub pcge_version: &'static str,
    /// Versión del esquema JSON de origen.
    pub schema_version: u32,
    /// Revisión del conjunto, distinta de la versión de biblioteca.
    pub dataset_revision: u32,
    /// Cantidad de registros canónicos.
    pub entry_count: usize,
}
/// Intervalo inclusivo de páginas de la fuente.
#[derive(Debug, Clone, Copy)]
pub struct PageRange {
    /// Primera página.
    pub first: u32,
    /// Última página.
    pub last: u32,
}
/// Procedencia transcrita del proyecto de origen. No implica una verificación nueva del PDF.
#[derive(Debug, Clone, Copy)]
pub struct Provenance {
    /// Título documental.
    pub title: &'static str,
    /// Autoridad de la fuente.
    pub authority: &'static str,
    /// Resolución.
    pub resolution: &'static str,
    /// Fecha de resolución ISO 8601.
    pub resolution_date: &'static str,
    /// Fecha de publicación ISO 8601.
    pub publication_date: &'static str,
    /// Fecha obligatoria indicada por la fuente, sin selección automática de edición.
    pub mandatory_effective_date: &'static str,
    /// Enlace a la resolución conservado del conjunto de origen.
    pub resolution_url: &'static str,
    /// Nombre del PDF de origen; el PDF no se distribuye aquí.
    pub source_filename: &'static str,
    /// SHA-256 del PDF declarado por el proyecto de origen.
    pub source_sha256: &'static str,
    /// SHA-256 de los bytes de entries.json.
    pub dataset_sha256: &'static str,
    /// Capítulo del catálogo.
    pub catalog_chapter: &'static str,
    /// Páginas del PDF.
    pub catalog_pdf_pages: PageRange,
    /// Páginas impresas.
    pub catalog_printed_pages: PageRange,
}
/// Aparición impresa conservada en el registro de anomalías.
#[derive(Debug, Clone, Copy)]
pub struct Occurrence {
    /// Índice de aparición en el registro original.
    pub occurrence_index: u32,
    /// Página PDF.
    pub pdf_page: u32,
    /// Página impresa.
    pub printed_page: u32,
    /// Código impreso.
    pub printed_code: &'static str,
    /// Nombre impreso.
    pub printed_name: &'static str,
    /// Padre impreso.
    pub printed_parent_code: &'static str,
    /// Tratamiento de la aparición en el catálogo canónico.
    pub disposition: &'static str,
}
/// Anomalía conocida de la fuente; no se inventan códigos de sustitución.
#[derive(Debug, Clone, Copy)]
pub struct Anomaly {
    /// Identificador original.
    pub id: &'static str,
    /// Tipo original.
    pub kind: &'static str,
    /// Códigos afectados (unifica los campos JSON `code` y `codes`).
    pub codes: &'static [&'static str],
    /// Estado original.
    pub status: &'static str,
    /// Descripción original.
    pub description: &'static str,
    /// Decisión canónica de origen.
    pub decision: &'static str,
    /// Confirmación de que no se inventaron códigos.
    pub confirmation_no_invented_code: &'static str,
    /// Apariciones impresas.
    pub occurrences: &'static [Occurrence],
}
/// Errores de consulta. Un `get` ausente devuelve `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Edición no disponible.
    UnsupportedEdition(String),
    /// Código desconocido en una consulta de relaciones.
    UnknownCode(String),
    /// Consulta vacía antes de normalizar o formada únicamente por espacios.
    EmptyQuery,
    /// Código vacío o con espacios en los extremos en `anomalies_for`.
    InvalidAnomalyCode,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedEdition(v) => write!(f, "edición no disponible: {v}"),
            Self::UnknownCode(v) => write!(f, "código no encontrado: {v}"),
            Self::EmptyQuery => f.write_str("la consulta no puede estar vacía"),
            Self::InvalidAnomalyCode => f.write_str("código vacío o con espacios en los extremos"),
        }
    }
}
impl error::Error for Error {}

/// Catálogo inmutable con datos embebidos. Copiable, `Send` y `Sync`.
#[derive(Debug, Clone, Copy)]
pub struct Catalog { edition: Edition }
impl Catalog {
    /// Carga una edición explícita sin E/S ni asignaciones dinámicas.
    pub const fn new(edition: Edition) -> Self { Self { edition } }
    fn data(&self) -> &'static data::CatalogData {
        match self.edition { Edition::V2019 => &data::V2019, Edition::V2026 => &data::V2026 }
    }
    /// Edición seleccionada.
    pub const fn edition(&self) -> Edition { self.edition }
    /// Número de registros.
    pub fn len(&self) -> usize { self.entries().len() }
    /// Indica si el catálogo no tiene registros.
    pub fn is_empty(&self) -> bool { self.entries().is_empty() }
    /// Registros en orden documental, sin copiar ni normalizar sus nombres.
    pub fn entries(&self) -> &'static [Entry] { self.data().entries }
    /// Consulta exacta de código. No recorta espacios ni convierte números.
    pub fn get(&self, code: &str) -> Option<&'static Entry> {
        let d = self.data();
        d.by_code.binary_search_by(|&i| d.entries[i].code.cmp(code)).ok().map(|i| &d.entries[d.by_code[i]])
    }
    fn require(&self, code: &str) -> Result<&'static Entry, Error> {
        self.get(code).ok_or_else(|| Error::UnknownCode(code.to_owned()))
    }
    /// Padre inmediato; raíz devuelve `Ok(None)`, código desconocido es error.
    pub fn parent(&self, code: &str) -> Result<Option<&'static Entry>, Error> {
        Ok(self.require(code)?.parent_code.and_then(|parent| self.get(parent)))
    }
    /// Hijos inmediatos en orden documental.
    pub fn children(&self, code: &str) -> Result<Vec<&'static Entry>, Error> {
        self.require(code)?;
        Ok(self.entries().iter().filter(|e| e.parent_code == Some(code)).collect())
    }
    /// Ancestros desde el padre inmediato hasta la raíz.
    pub fn ancestors(&self, code: &str) -> Result<Vec<&'static Entry>, Error> {
        let mut result = Vec::new();
        let mut parent = self.require(code)?.parent_code;
        while let Some(code) = parent {
            let entry = self.require(code)?;
            result.push(entry);
            parent = entry.parent_code;
        }
        Ok(result)
    }
    /// Descendientes en preorden DFS, manteniendo el orden documental de hermanos.
    pub fn descendants(&self, code: &str) -> Result<Vec<&'static Entry>, Error> {
        let mut stack = self.children(code)?;
        stack.reverse();
        let mut result = Vec::new();
        while let Some(entry) = stack.pop() {
            result.push(entry);
            for child in self.entries().iter().rev().filter(|e| e.parent_code == Some(entry.code)) {
                stack.push(child);
            }
        }
        Ok(result)
    }
    /// Busca subcadenas en código o nombre con NFKD, eliminación de caracteres con
    /// clase combinante no nula y casefold completo. Recorta espacios como Python.
    /// Una consulta formada solo por marcas combinantes coincide con todo el catálogo,
    /// igual que la implementación de origen. No aplica tokenización ni límite.
    pub fn search(&self, query: &str) -> Result<Vec<&'static Entry>, Error> {
        let cleaned = query.trim_matches(unicode::is_whitespace);
        if cleaned.is_empty() { return Err(Error::EmptyQuery); }
        let normalized = unicode::normalize(cleaned);
        Ok(self.entries().iter().zip(self.data().search_names.iter())
            .filter(|(e, name)| e.code.contains(&normalized) || name.contains(&normalized))
            .map(|(entry, _)| entry).collect())
    }
    /// Metadatos de la edición.
    pub fn metadata(&self) -> &'static Metadata { self.data().metadata }
    /// Procedencia documental de origen.
    pub fn provenance(&self) -> &'static Provenance { self.data().provenance }
    /// Todas las anomalías conocidas, en orden de origen.
    pub fn anomalies(&self) -> &'static [Anomaly] { self.data().anomalies }
    /// Anomalías por código afectado o impreso, incluso si no está en el catálogo.
    pub fn anomalies_for(&self, code: &str) -> Result<Vec<&'static Anomaly>, Error> {
        if code.is_empty() || code.trim_matches(unicode::is_whitespace) != code {
            return Err(Error::InvalidAnomalyCode);
        }
        Ok(self.anomalies().iter().filter(|a| a.codes.contains(&code)
            || a.occurrences.iter().any(|o| o.printed_code == code)).collect())
    }
    /// Bytes JSON originales de los registros, sin reserialización.
    pub fn entries_json(&self) -> &'static str { self.data().entries_json }
    /// Bytes JSON originales de metadatos.
    pub fn metadata_json(&self) -> &'static str { self.data().metadata_json }
    /// Bytes JSON originales de procedencia.
    pub fn provenance_json(&self) -> &'static str { self.data().provenance_json }
    /// Bytes JSON originales de anomalías, incluyendo la forma `code`/`codes` original.
    pub fn anomalies_json(&self) -> &'static str { self.data().anomalies_json }
}

/// Normalización de búsqueda fijada a Unicode 16.0.0, sin recortar espacios.
/// Útil para comparar claves; no debe sustituir nombres o códigos originales.
/// El orden es NFKD → eliminar clase combinante no nula → casefold completo.
pub fn normalize_for_search(text: &str) -> String { unicode::normalize(text) }
