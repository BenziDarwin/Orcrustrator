#[cfg(feature = "events")]
mod event_stream_tests {
    use orcrustrator::event::*;
    use orcrustrator::cache::{CacheProvider, MemoryCache};
    use orcrustrator::format::StructuredValue;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_channel_transport_broadcast() {
        let transport = Arc::new(ChannelTransport::new(32));
        let mut rx1 = transport.subscribe();
        let mut rx2 = transport.subscribe();

        let payload_bytes = rmp_serde::to_vec(&StructuredValue::String("Malaria Rapid Test".to_string())).unwrap();

        let event = RelationalEventEnvelope::new(
            "patient-registry",
            "rel.patient.created",
            "stream-patients",
            1,
            "rel_101",
            "Created",
            Some(payload_bytes),
        );

        transport.publish(&event).await.unwrap();

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        assert_eq!(received1.id, event.id);
        assert_eq!(received1.specversion, "1.0");
        assert_eq!(received1.datacontenttype, "application/msgpack");
        assert_eq!(received1.relation_id, "rel_101");
        assert_eq!(received1.operation, "Created");

        assert_eq!(received2.id, event.id);
        assert_eq!(received2.stream_id, "stream-patients");
    }

    #[test]
    fn test_length_prefixed_msgpack_roundtrip() {
        let payload_bytes = rmp_serde::to_vec(&StructuredValue::String("Cholera due to Vibrio cholerae".to_string())).unwrap();

        let event = RelationalEventEnvelope::new(
            "moh-terminology-service",
            "rel.terminology.synced",
            "stream-concepts",
            42,
            "concept_icd10_a00",
            "Updated",
            Some(payload_bytes.clone()),
        );

        let wire_buffer = encode_frame(&event).unwrap();

        assert!(wire_buffer.len() > 4);
        let frame_len = u32::from_be_bytes(wire_buffer[0..4].try_into().unwrap()) as usize;
        assert_eq!(wire_buffer.len(), 4 + frame_len);

        let (decoded, consumed) = decode_frame(&wire_buffer).unwrap().unwrap();

        assert_eq!(consumed, wire_buffer.len());
        assert_eq!(decoded.id, event.id);
        assert_eq!(decoded.relation_id, "concept_icd10_a00");
        assert_eq!(decoded.operation, "Updated");
        assert_eq!(decoded.data, Some(payload_bytes));
    }

    #[tokio::test]
    async fn test_cache_provider_invalidation_lifecycle() {
        let cache = MemoryCache::with_default_ttl(Duration::from_millis(500));

        let key = "concept:icd10:b54";
        let val = StructuredValue::String("Unspecified malaria".to_string());

        assert!(cache.get(key).await.unwrap().is_none());

        cache.set(key, &val, None).await.unwrap();
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved.as_ref(), Some(&val));

        cache.invalidate(key).await.unwrap();
        assert!(cache.get(key).await.unwrap().is_none());

        cache.set(key, &val, Some(Duration::from_millis(50))).await.unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(cache.get(key).await.unwrap().is_none());
    }
}
