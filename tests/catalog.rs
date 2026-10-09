use pcge_peru::{normalize_for_search, Catalog, Edition, Error, Level, SOURCE_COMMIT, UNICODE_VERSION};

#[test]
fn all_records_are_findable_and_have_consistent_hierarchy() {
    for (edition, count, anomaly_count) in [(Edition::V2019, 1757, 10), (Edition::V2026, 1636, 1)] {
        let catalog = Catalog::new(edition);
        assert_eq!(catalog.edition(), edition);
        assert_eq!(catalog.len(), count);
        assert!(!catalog.is_empty());
        assert_eq!(catalog.metadata().entry_count, count);
        assert_eq!(catalog.metadata().pcge_version, edition.as_str());
        assert_eq!(catalog.metadata().schema_version, 1);
        assert_eq!(catalog.metadata().dataset_revision, 1);
        assert_eq!(catalog.provenance().dataset_sha256.len(), 64);
        assert_eq!(catalog.anomalies().len(), anomaly_count);
        for entry in catalog.entries() {
            assert_eq!(catalog.get(entry.code), Some(entry));
            assert!(entry.code.bytes().all(|c| c.is_ascii_digit()));
            let parent = catalog.parent(entry.code).unwrap();
            assert_eq!(parent.map(|p| p.code), entry.parent_code);
            if let Some(parent) = parent {
                assert_eq!(parent.code, &entry.code[..entry.code.len() - 1]);
                assert!(catalog.children(parent.code).unwrap().contains(&entry));
            }
            let ancestors = catalog.ancestors(entry.code).unwrap();
            assert_eq!(ancestors.len(), entry.code_length() - 1);
            assert_eq!(entry.level().is_none(), entry.code_length() == 6);
            assert!(catalog.search(entry.name).unwrap().contains(&entry));
        }
    }
}
#[test]
fn editions_are_explicit() {
    assert_eq!("2019".parse::<Edition>().unwrap(), Edition::V2019);
    assert_eq!("2026".parse::<Edition>().unwrap(), Edition::V2026);
    for invalid in ["", " 2026", "2026 ", "2025", "latest"] { assert!(invalid.parse::<Edition>().is_err()); }
    assert_eq!(Edition::V2026.to_string(), "2026");
    assert_eq!(SOURCE_COMMIT, "e9e69ed9075c09e8d1e5475760e5b691aa73b251");
    assert_eq!(UNICODE_VERSION, "16.0.0");
}
#[test]
fn exact_codes_and_missing_codes_are_distinct_from_roots() {
    let catalog = Catalog::new(Edition::V2026);
    assert_eq!(catalog.get("101").unwrap().name, "Caja");
    assert_eq!(catalog.get("101").unwrap().level(), Some(Level::Subaccount));
    assert_eq!(catalog.parent("1"), Ok(None));
    assert_eq!(catalog.get(" 101"), None);
    assert_eq!(catalog.get("０１０１"), None);
    assert_eq!(catalog.get(""), None);
    assert!(matches!(catalog.parent("no"), Err(Error::UnknownCode(_))));
    assert!(catalog.children("no").is_err());
    assert!(catalog.ancestors("no").is_err());
    assert!(catalog.descendants("no").is_err());
    assert!(catalog.children("101").unwrap().is_empty());
    assert!(catalog.descendants("101").unwrap().is_empty());
}
#[test]
fn search_preserves_unicode_and_upstream_edge_cases() {
    let catalog = Catalog::new(Edition::V2026);
    assert_eq!(catalog.search("DEPOSITOS"), catalog.search("depósitos"));
    assert_eq!(catalog.search("depo\u{301}sitos"), catalog.search("depósitos"));
    assert_eq!(catalog.search("\u{1c}caja\u{2003}"), catalog.search("caja"));
    assert_eq!(catalog.search("ＣＡＪＡ"), catalog.search("caja"));
    assert_eq!(catalog.search("\u{301}").unwrap().len(), catalog.len());
    assert_eq!(catalog.search("\u{345}").unwrap().len(), catalog.len());
    assert_eq!(catalog.search(""), Err(Error::EmptyQuery));
    assert_eq!(catalog.search("\u{1c}\u{2003}"), Err(Error::EmptyQuery));
    assert!(catalog.search("caja\0x").unwrap().is_empty());
    assert_eq!(normalize_for_search("Straße ﬃ İ ǰ"), "strasse ffi i j");
    assert_eq!(normalize_for_search("\u{345}"), "");
    assert_eq!(normalize_for_search("\u{1c}X"), "\u{1c}x");
}
#[test]
fn anomalies_remain_visible_without_invented_codes() {
    let catalog = Catalog::new(Edition::V2026);
    let anomaly = catalog.anomalies_for("70992").unwrap()[0];
    assert_eq!(anomaly.id, "ANOMALY-2026-70992");
    assert_eq!(anomaly.occurrences.len(), 2);
    assert_eq!(anomaly.occurrences[0].printed_parent_code, "7090");
    assert_eq!(catalog.get("70992").unwrap().name, "Contrato de consultoría TI");
    assert!(catalog.get("70902").is_none());
    assert!(catalog.anomalies_for("101").unwrap().is_empty());
    for invalid in ["", " ", " 70992", "70992\u{2003}"] {
        assert_eq!(catalog.anomalies_for(invalid).unwrap_err(), Error::InvalidAnomalyCode);
    }
    let old = Catalog::new(Edition::V2019);
    assert_eq!(old.anomalies_for("33404").unwrap().len(), 1);
    assert!(old.get("33404").is_none());
    assert!(old.get("36404").is_none());
}
#[test]
fn static_borrows_and_thread_safety() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Catalog>();
    let entry = { Catalog::new(Edition::V2026).get("101").unwrap() };
    assert_eq!(entry.code, "101");
    let catalog = Catalog::new(Edition::V2019);
    let handles: Vec<_> = (0..4).map(|_| std::thread::spawn(move || catalog.search("caja").unwrap())).collect();
    for handle in handles { assert_eq!(handle.join().unwrap(), catalog.search("caja").unwrap()); }
}
