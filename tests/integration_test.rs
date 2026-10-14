use orcrustrator::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UserProfile {
    id: u64,
    name: String,
    company: Option<CompanyDetails>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CompanyDetails {
    name: String,
    tier: String,
}

#[test]
fn test_structured_value_and_typed_deserialization() {
    let mut val = StructuredValue::empty_object();
    val.set_path("user.id", StructuredValue::Number(42.0));
    val.set_path("user.name", StructuredValue::String("Alice".to_string()));
    val.set_path("user.company.name", StructuredValue::String("Acme Corp".to_string()));
    val.set_path("user.company.tier", StructuredValue::String("enterprise".to_string()));
    val.set_path(
        "user.tags",
        StructuredValue::Array(vec![
            StructuredValue::String("admin".to_string()),
            StructuredValue::String("developer".to_string()),
        ]),
    );

    // Test get_path
    assert_eq!(
        val.get_path("user.name").unwrap().as_str(),
        Some("Alice")
    );
    assert_eq!(
        val.get_path("$.user.company.tier").unwrap().as_str(),
        Some("enterprise")
    );

    // Test typed extraction
    let user_obj = val.get_path("user").unwrap();
    let typed: UserProfile = user_obj.into_typed().expect("failed to deserialize UserProfile");
    assert_eq!(typed.id, 42);
    assert_eq!(typed.name, "Alice");
    assert_eq!(
        typed.company,
        Some(CompanyDetails {
            name: "Acme Corp".to_string(),
            tier: "enterprise".to_string(),
        })
    );
    assert_eq!(
        typed.tags,
        Some(vec!["admin".to_string(), "developer".to_string()])
    );
}

#[test]
fn test_schema_inspector_inference() {
    let mut root = StructuredValue::empty_object();
    root.set_path("id", StructuredValue::Number(101.0));
    root.set_path("active", StructuredValue::Bool(true));
    root.set_path("title", StructuredValue::String("API Engineer".to_string()));
    root.set_path(
        "skills",
        StructuredValue::Array(vec![
            StructuredValue::String("Rust".to_string()),
            StructuredValue::String("Distributed Systems".to_string()),
        ]),
    );

    let schema = SchemaInspector::infer(&root);
    let pretty = SchemaInspector::pretty_print(&schema, 0);

    assert!(pretty.contains("id : number"));
    assert!(pretty.contains("active : boolean"));
    assert!(pretty.contains("title : string"));
    assert!(pretty.contains("skills : [ string ]"));
}

#[cfg(feature = "xml")]
#[test]
fn test_xml_parsing_to_structured_value() {
    let xml_payload = r#"
        <response status="success">
            <user>
                <id>1001</id>
                <name>Bob</name>
                <role>Lead</role>
            </user>
        </response>
    "#;

    let format = DataFormat::detect(Some("application/xml"), xml_payload);
    assert_eq!(format, DataFormat::Xml);

    let parsed = format.parse(xml_payload).expect("XML parse should succeed");
    assert!(parsed.get_path("response.user.id").is_some());
    assert_eq!(
        parsed.get_path("response.@status").unwrap().as_str(),
        Some("success")
    );
}

#[test]
fn test_relational_stitcher_has_one_and_has_many() {
    let mut user = StructuredValue::empty_object();
    user.set_path("id", StructuredValue::Number(1.0));
    user.set_path("name", StructuredValue::String("Charlie".to_string()));

    // 1:1 HasOne Stitching
    let mut company = StructuredValue::empty_object();
    company.set_path("name", StructuredValue::String("Cyberdyne".to_string()));
    company.set_path("hq", StructuredValue::String("Los Angeles".to_string()));

    RelationalStitcher::embed_has_one(&mut user, "company", company);
    assert_eq!(
        user.get_path("company.name").unwrap().as_str(),
        Some("Cyberdyne")
    );

    // 1:N HasMany Stitching
    let mut order1 = StructuredValue::empty_object();
    order1.set_path("order_id", StructuredValue::String("ORD-1".to_string()));
    order1.set_path("total", StructuredValue::Number(99.95));

    let mut order2 = StructuredValue::empty_object();
    order2.set_path("order_id", StructuredValue::String("ORD-2".to_string()));
    order2.set_path("total", StructuredValue::Number(150.00));

    RelationalStitcher::embed_has_many(&mut user, "orders", vec![order1, order2]);

    let orders_arr = user.get_path("orders").unwrap().as_array().unwrap();
    assert_eq!(orders_arr.len(), 2);
    assert_eq!(
        orders_arr[0].get_path("order_id").unwrap().as_str(),
        Some("ORD-1")
    );
}

#[test]
fn test_service_registry_route_matching() {
    let mut registry = ServiceRegistry::new();
    let root_ep = Endpoint::get("/users/{id}");
    let rel_ep = RelationalEndpoint::new("/api/v1/users/{id}", "user", root_ep);
    registry.register_read(rel_ep);

    let post_ep = MutationEndpoint::new(HttpMethod::Post, "/api/v1/users");
    registry.register_mutation(post_ep);

    // Match GET /api/v1/users/999
    let match_res = registry.match_route(HttpMethod::Get, "/api/v1/users/999");
    assert!(match_res.is_some());
    if let Some(MatchedRoute::Read(ep, params)) = match_res {
        assert_eq!(ep.path_pattern, "/api/v1/users/{id}");
        assert_eq!(params.get("id"), Some(&"999".to_string()));
    } else {
        panic!("Expected MatchedRoute::Read");
    }

    // Match POST /api/v1/users
    let post_match = registry.match_route(HttpMethod::Post, "/api/v1/users");
    assert!(post_match.is_some());

    // Non-matching route
    assert!(registry.match_route(HttpMethod::Delete, "/api/v1/users").is_none());
}

#[tokio::test]
async fn test_data_sink_persistence_hook() {
    let sink = MemoryDataSink::new();
    let mut test_data = StructuredValue::empty_object();
    test_data.set_path("status", StructuredValue::String("ok".to_string()));

    let mut params = std::collections::HashMap::new();
    params.insert("tenant".to_string(), "t-42".to_string());

    sink.on_data("/test-endpoint", HttpMethod::Get, &params, &test_data)
        .await
        .expect("sink call should succeed");

    let records = sink.get_records().await;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].endpoint_path, "/test-endpoint");
    assert_eq!(records[0].method, HttpMethod::Get);
    assert_eq!(records[0].params.get("tenant"), Some(&"t-42".to_string()));
    assert_eq!(records[0].data.get_path("status").unwrap().as_str(), Some("ok"));
}

#[test]
fn test_transformer_projection() {
    let mut raw = StructuredValue::empty_object();
    raw.set_path("deep.legacy.user_first_name", StructuredValue::String("Diana".to_string()));
    raw.set_path("deep.legacy.user_age", StructuredValue::Number(30.0));
    raw.set_path("irrelevant_data", StructuredValue::String("junk".to_string()));

    let mut mappings = BTreeMap::new();
    mappings.insert("name".to_string(), "deep.legacy.user_first_name".to_string());
    mappings.insert("age".to_string(), "deep.legacy.user_age".to_string());

    let projected = Transformer::project(&raw, &mappings);
    assert_eq!(projected.get_path("name").unwrap().as_str(), Some("Diana"));
    assert_eq!(projected.get_path("age").unwrap(), &StructuredValue::Number(30.0));
    assert!(projected.get_path("irrelevant_data").is_none());
}
