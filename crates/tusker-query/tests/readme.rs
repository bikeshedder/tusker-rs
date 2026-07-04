#[test]
fn readme_does_not_reference_public_query_client_trait() {
    assert!(!include_str!("../README.md").contains("tusker_query::QueryClient"));
}
