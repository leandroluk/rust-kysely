use rust_kysely::VERSION;

#[test]
fn test_version_present() {
    assert!(!VERSION.is_empty());
}
