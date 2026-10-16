use crate::error::{OrcrustratorError, Result};
use crate::event::RelationalEventEnvelope;

/// Encodes a CloudEvents envelope into a length-prefixed MessagePack byte frame.
///
/// Framing Format:
/// - 4 bytes: `u32` Big-Endian Network Byte Order denoting the payload length $L$.
/// - $L$ bytes: Raw MessagePack payload.
pub fn encode_frame(envelope: &RelationalEventEnvelope) -> Result<Vec<u8>> {
    let msgpack_bytes = rmp_serde::to_vec(envelope)
        .map_err(|e| OrcrustratorError::EventStreamError(format!("MessagePack serialization failed: {}", e)))?;

    let len = msgpack_bytes.len() as u32;
    let mut frame = Vec::with_capacity(4 + msgpack_bytes.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&msgpack_bytes);

    Ok(frame)
}

/// Decodes a length-prefixed MessagePack frame from a byte buffer.
///
/// Returns `Ok(Some((envelope, consumed_bytes)))` if a complete frame was read,
/// or `Ok(None)` if more bytes are needed (partial read handling).
pub fn decode_frame(buf: &[u8]) -> Result<Option<(RelationalEventEnvelope, usize)>> {
    if buf.len() < 4 {
        return Ok(None);
    }

    let len_bytes: [u8; 4] = [buf[0], buf[1], buf[2], buf[3]];
    let frame_len = u32::from_be_bytes(len_bytes) as usize;

    if buf.len() < 4 + frame_len {
        return Ok(None);
    }

    let payload = &buf[4..4 + frame_len];
    let envelope: RelationalEventEnvelope = rmp_serde::from_slice(payload)
        .map_err(|e| OrcrustratorError::EventStreamError(format!("MessagePack deserialization failed: {}", e)))?;

    Ok(Some((envelope, 4 + frame_len)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let envelope = RelationalEventEnvelope::new(
            "orcrustrator://moh-service",
            "orcrustrator.relation.stitched",
            "stream-1",
            100,
            "patient-concept-link",
            "Stitched",
            Some(vec![1, 2, 3]),
        );

        let frame = encode_frame(&envelope).expect("Encoding should succeed");
        assert!(frame.len() > 4);

        let decoded = decode_frame(&frame).expect("Decoding should succeed");
        assert!(decoded.is_some());

        let (result, consumed) = decoded.unwrap();
        assert_eq!(consumed, frame.len());
        assert_eq!(result.id, envelope.id);
        assert_eq!(result.stream_id, "stream-1");
        assert_eq!(result.sequence, 100);
        assert_eq!(result.operation, "Stitched");
    }

    #[test]
    fn test_partial_read() {
        let envelope = RelationalEventEnvelope::new(
            "orcrustrator://moh-service",
            "orcrustrator.relation.stitched",
            "stream-1",
            1,
            "link",
            "Created",
            None,
        );

        let frame = encode_frame(&envelope).unwrap();
        // Pass partial bytes
        let partial = &frame[..frame.len() - 2];
        let decoded = decode_frame(partial).unwrap();
        assert!(decoded.is_none());
    }
}
