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
}
