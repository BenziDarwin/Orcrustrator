#[cfg(feature = "compiler")]
mod compile_tests {
    use orcrustrator::compiler::*;
    use orcrustrator::oop::RelationshipDeclaration;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compiler_scan_and_parse_declarations() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("patient_service.rs");

        let code = r#"
            // Sample source code with relation declarations
            relation! {
                id: "patient_concept_ref",
                source: "patient_registry",
                target: "terminology_service",
                relation_type: "references",
                foreign_key: "concept_id",
                embed_as: "concept",
            }

            relation! {
                id: "encounter_diagnosis_ref",
                source: "encounter_service",
                target: "terminology_service",
                relation_type: "has_one",
                foreign_key: "diagnosis_code",
                embed_as: "diagnosis",
            }
        "#;

        fs::write(&src_file, code).unwrap();

        let scanned = scan_source_directory(dir.path()).unwrap();
        assert_eq!(scanned.len(), 2);

        let p_rel = scanned.iter().find(|r| r.id == "patient_concept_ref").unwrap();
        assert_eq!(p_rel.source, "patient_registry");
        assert_eq!(p_rel.target, "terminology_service");
        assert_eq!(p_rel.relation_type, "references");
        assert_eq!(p_rel.foreign_key, "concept_id");
        assert_eq!(p_rel.embed_as, "concept");

        let e_rel = scanned.iter().find(|r| r.id == "encounter_diagnosis_ref").unwrap();
        assert_eq!(e_rel.source, "encounter_service");
        assert_eq!(e_rel.target, "terminology_service");
    }

    #[test]
    fn test_manifest_merge_preserves_gui_overrides() {
        let code_decls = vec![
            RelationshipDeclaration::new(
                "patient_terminology",
                "patient_registry",
                "terminology_service_v1",
                "references",
                "concept_id",
                "concept",
            ),
            RelationshipDeclaration::new(
                "encounter_facility",
                "encounter_service",
                "facility_registry",
                "has_one",
                "facility_id",
                "facility",
            ),
        ];

        let def_manifest = DefinitionsManifest::new(code_decls);

        // Operator in GUI rerouted patient_terminology to terminology_v2
        let mut overrides = OverridesManifest::default();
        overrides.overrides.insert(
            "patient_terminology".to_string(),
            RelationOverride {
                target: Some("terminology_service_v2".to_string()),
                ..Default::default()
            },
        );

        let effective = merge_manifests(&def_manifest, &overrides);

        let patient_rel = effective.relations.get("patient_terminology").unwrap();
        assert_eq!(patient_rel.target, "terminology_service_v2");
        assert_eq!(patient_rel.foreign_key, "concept_id");

        let facility_rel = effective.relations.get("encounter_facility").unwrap();
        assert_eq!(facility_rel.target, "facility_registry");
    }

    #[test]
    fn test_cycle_detection_rejects_cycles() {
        let cyclic_decls = vec![
            RelationshipDeclaration::new("rel_1", "service_a", "service_b", "references", "id", "b"),
            RelationshipDeclaration::new("rel_2", "service_b", "service_c", "references", "id", "c"),
            RelationshipDeclaration::new("rel_3", "service_c", "service_a", "references", "id", "a"),
        ];

        let result = validate_dag(&cyclic_decls);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Circular relation cycle detected"));
    }

    #[test]
    fn test_atomic_file_write_and_diff() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("definitions.toml");

        let decls_v1 = vec![
            RelationshipDeclaration::new("rel_a", "srv_1", "srv_2", "has_one", "fk_a", "emb_a"),
        ];

        let manifest_v1 = DefinitionsManifest::new(decls_v1);
        manifest_v1.save_atomic(&manifest_path).unwrap();

        let loaded_v1 = DefinitionsManifest::load_from_file(&manifest_path).unwrap();
        assert_eq!(loaded_v1.relations.len(), 1);

        // New declarations with an addition and an update
        let decls_v2 = vec![
            RelationshipDeclaration::new("rel_a", "srv_1", "srv_2_updated", "has_one", "fk_a", "emb_a"),
            RelationshipDeclaration::new("rel_b", "srv_3", "srv_4", "references", "fk_b", "emb_b"),
        ];

        let manifest_v2 = DefinitionsManifest::new(decls_v2);
        let diff = calculate_diff(&loaded_v1, &manifest_v2);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.removed.len(), 0);
        assert!(!diff.is_empty());
    }

    #[test]
    fn test_infer_relations_from_struct_models() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("models.rs");

        let code = r#"
            pub struct TradeItem {
                pub id: String,
                pub name: String,
            }

            pub struct Product {
                pub product_code: String,
                pub category_id: String,
                pub trade_items: Vec<TradeItem>,
            }
        "#;

        fs::write(&src_file, code).unwrap();

        let scanned = scan_source_directory(dir.path()).unwrap();
        assert!(!scanned.is_empty(), "Should automatically infer relations from struct models");

        // Verify HasMany inference
        let items_rel = scanned.iter().find(|r| r.id == "product_trade_items").expect("Should infer product_trade_items");
        assert_eq!(items_rel.relation_type, "HasMany");
        assert_eq!(items_rel.embed_as, "trade_items");

        // Verify HasOne foreign key inference
        let cat_rel = scanned.iter().find(|r| r.id == "product_category").expect("Should infer product_category");
        assert_eq!(cat_rel.relation_type, "HasOne");
        assert_eq!(cat_rel.foreign_key, "category_id");
        assert_eq!(cat_rel.embed_as, "category");
    }

    #[test]
    fn test_infer_relations_from_route_hierarchies() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("routes.rs");

        let code = r#"
            pub fn setup_routes() {
                let base = "/api/v1/facilities";
                let wards = "/api/v1/facilities/{facility_code}/wards";
                println!("Routes: {} and {}", base, wards);
            }
        "#;

        fs::write(&src_file, code).unwrap();

        let scanned = scan_source_directory(dir.path()).unwrap();
        assert!(!scanned.is_empty(), "Should automatically infer relations from route strings");

        // Verify child relation inference from path prefix
        let wards_rel = scanned.iter().find(|r| r.id == "facilities_wards").expect("Should infer facilities_wards relation");
        assert_eq!(wards_rel.source, "/api/v1/facilities");
        assert_eq!(wards_rel.target, "/api/v1/facilities/{facility_code}/wards");
        assert_eq!(wards_rel.relation_type, "HasMany");
        assert_eq!(wards_rel.foreign_key, "facility_code");
        assert_eq!(wards_rel.embed_as, "wards");

        // Verify Root relation was generated for the parent
        let root_rel = scanned.iter().find(|r| r.id == "facilities_root").expect("Should infer facilities_root");
        assert_eq!(root_rel.relation_type, "Root");
        assert_eq!(root_rel.source, "/api/v1/facilities");
    }

    #[test]
    fn test_infer_relations_from_struct_field_attributes() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("annotated_models.rs");

        let code = r#"
            pub struct MedicalProduct {
                pub id: String,
                #[relation(has_one, target = "/api/v1/categories/{category_id}", fk = "category_id")]
                pub category_id: String,
                #[relation(has_many, target = "/api/v1/products/{product_code}/trade_items", embed_as = "trade_items")]
                pub trade_items: Vec<String>,
            }
        "#;

        fs::write(&src_file, code).unwrap();

        let scanned = scan_source_directory(dir.path()).unwrap();
        assert_eq!(scanned.len(), 3, "Expected 1 root and 2 field relations");

        let cat_rel = scanned.iter().find(|r| r.id == "medical_product_category_id").unwrap();
        assert_eq!(cat_rel.relation_type, "HasOne");
        assert_eq!(cat_rel.target, "/api/v1/categories/{category_id}");
        assert_eq!(cat_rel.foreign_key, "category_id");

        let items_rel = scanned.iter().find(|r| r.id == "medical_product_trade_items").unwrap();
        assert_eq!(items_rel.relation_type, "HasMany");
        assert_eq!(items_rel.target, "/api/v1/products/{product_code}/trade_items");
        assert_eq!(items_rel.embed_as, "trade_items");
    }
}
