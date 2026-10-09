# PCGE Perú para Rust

Biblioteca Rust de consulta del Plan Contable General Empresarial con ediciones 2019 y 2026 explícitas. Incluye los catálogos canónicos completos, jerarquía, búsqueda Unicode, procedencia documental y anomalías conocidas.

- Edición 2019: 1.757 registros y 10 anomalías documentadas
- Edición 2026: 1.636 registros y 1 anomalía documentada
- Datos fijados al commit [`e9e69ed`](https://github.com/mat-l-dev/pcge-peru/tree/e9e69ed9075c09e8d1e5475760e5b691aa73b251) del proyecto de origen
- Unicode 16.0.0 completo, independiente de las tablas del compilador
- Sin dependencias, red, archivos de ejecución, build script ni código unsafe

## Uso

El crate se llama `pcge-peru` y se importa como `pcge_peru`. Este repositorio puede utilizarse mediante una dependencia de ruta o Git; no se presupone una publicación en crates.io.

```toml
[dependencies]
pcge-peru = { path = "../pcge-peru-rust" }
```

```rust
use pcge_peru::{Catalog, Edition};

fn main() -> Result<(), pcge_peru::Error> {
    let catalog = Catalog::new(Edition::V2026);
    let caja = catalog.get("101").expect("código canónico conocido");
    assert_eq!(caja.name, "Caja");
    assert_eq!(catalog.parent("101")?.unwrap().code, "10");

    for entry in catalog.search("depósitos")? {
        println!("{}: {}", entry.code, entry.name);
    }
    Ok(())
}
```

```sh
cargo test --offline
cargo run --offline --example pcge -- 2026 get 70992
cargo run --offline --example pcge -- 2019 search "depósitos"
cargo doc --offline --no-deps
```

Rust 2021, con versión mínima declarada 1.70. La comprobación inicial se ejecutó con Rust 1.99.0 en Linux x86-64; la versión mínima no se comprobó en ese entorno. Se requiere `std`. Python no es necesario para compilar, usar o ejecutar las pruebas Rust.

## API

`Catalog::new(Edition::V2019)` y `Catalog::new(Edition::V2026)` cargan referencias a datos estáticos sin E/S ni asignación dinámica. `Edition` también implementa `FromStr` y acepta únicamente `"2019"` o `"2026"`.

- `entries()`: registros en orden documental
- `get(code)`: consulta exacta, devuelve `Option<&'static Entry>`
- `parent(code)`: padre inmediato, `Ok(None)` para una raíz y error para un código ausente
- `children(code)`: hijos inmediatos en orden documental
- `ancestors(code)`: padre inmediato hasta la raíz
- `descendants(code)`: DFS en preorden, conservando el orden documental de hermanos
- `search(query)`: subcadena de código o nombre normalizado, sin límite ni orden por relevancia
- `metadata()` y `provenance()`: metadatos tipados y procedencia de origen
- `anomalies()` y `anomalies_for(code)`: anomalías conocidas y apariciones impresas, incluso códigos excluidos del catálogo
- `entries_json()`, `metadata_json()`, `provenance_json()` y `anomalies_json()`: bytes JSON originales como texto UTF-8, sin reserialización
- `normalize_for_search(text)`: normalización Unicode fijada, sin recortar espacios

Los códigos son cadenas y no se convierten a enteros. `"0101"`, `" 101"` y `"101"` son claves distintas. Los códigos de seis dígitos devuelven `None` en `Entry::level()`, porque no se inventa un sexto nivel PCGE.

El catálogo es `Copy`, `Send` y `Sync`; no tiene estado mutable global. Las referencias a sus registros y metadatos tienen duración estática y siguen siendo válidas después de descartar el catálogo. Las colecciones de consultas usan `Vec` propiedad del llamante, con referencias estáticas a los elementos. La búsqueda asigna una cadena temporal normalizada. No es una biblioteca `no_std` ni una API de asignación fallible.

## Búsqueda y compatibilidad

La búsqueda mantiene la semántica del origen: recorta espacios como Python; aplica NFKD; elimina caracteres cuya clase combinante canónica no es cero; aplica casefold completo; compara subcadenas en código o nombre.

`depósitos`, `DEPOSITOS`, la forma con acento descompuesto y letras compatibles de ancho completo se normalizan de forma consistente. No se elimina toda la categoría Unicode de marcas: las marcas de clase combinante cero se conservan. Se recortan U+001C–U+001F; BOM y espacio de ancho cero no se recortan.

Una consulta vacía o formada solo por espacios produce `Error::EmptyQuery`. Una consulta formada únicamente por marcas que se eliminan coincide con todos los registros, como en el origen. No se aplica búsqueda aproximada, tokenización ni límite automático. Los nombres originales se devuelven sin cambios.

## Procedencia y límites

Los ocho JSON en `data/2019` y `data/2026` se conservan byte por byte. `data/manifest.json` fija sus SHA-256 y los de las tablas y fixtures. Los hashes PDF de `source.json` son metadatos heredados: el paquete no distribuye esos PDF ni afirma una verificación nueva.

La edición 2026 declara en sus metadatos de origen una fecha obligatoria de 2028-01-01. La edición siempre la elige el consumidor; la biblioteca no determina qué norma corresponde a una operación.

La anomalía `70992` conserva el registro canónico “Contrato de consultoría TI” bajo `7099` y las dos apariciones impresas. No se crea `70902`. Las otras anomalías se conservan sin correcciones inventadas.

El alcance es la consulta de estos dos catálogos fijados. No ofrece constructor de catálogos personalizados, importación JSON en ejecución, edición de registros, actualizaciones automáticas ni motor de asientos. No calcula impuestos ni decide tratamientos contables. La consulta de datos no sustituye la revisión de la norma aplicable.

## Pruebas y regeneración

```sh
cargo test --offline --all-targets
cargo test --offline --doc
cargo build --offline --examples
python3 tests/parity.py target/debug/examples/pcge target/debug/examples/unicode_dump
python3 scripts/generate.py --check
python3 scripts/generate.py
```

Los dos últimos tipos de comprobación requieren Python 3.10+ y solo su biblioteca estándar. Funcionan sin red ni rutas a otros repositorios. El generador valida hashes, recuentos, unicidad y jerarquía antes de producir las tablas Rust versionadas. No usa los datos Unicode de la versión local de Python.

La paridad compara los 3.393 registros, todas sus relaciones y el orden, 44 búsquedas y las consultas de anomalías de referencia. El comprobador Unicode recorre los 1.112.064 valores escalares y compara las 19.026 transformaciones con la tabla fijada. `data/unicode-normalization.json` registra las fuentes oficiales Unicode y sus hashes. Se incluyen pruebas locales, sin workflows de CI.

## Licencia

Software: Apache-2.0, con el `LICENSE` del origen preservado. Tablas Unicode: Unicode-3.0, en `licenses/LICENSE-UNICODE`. Véase `NOTICE`. La licencia del software no se atribuye a los textos oficiales del Estado peruano.
