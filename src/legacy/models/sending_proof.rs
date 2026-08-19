//! What the sending-proof route gives back.

/// The proof of sending.
///
/// The wire answers one of two bodies: `{ content }` with the PDF in base64, or
/// `{ message }` when the e-mail has no delivery status yet. The SDK decodes
/// the PDF for you and keeps the raw base64 reachable, so exactly one of `pdf`
/// and `message` is filled in.
///
/// The message branch is **not** an error: it means "ask again later", and
/// raising there would make callers match on a string to tell a wait from a
/// failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendingProof {
    /// The proof, decoded. `None` while the gateway only has a message.
    pub pdf: Option<Vec<u8>>,
    /// The base64 exactly as the gateway sent it.
    pub content_base64: Option<String>,
    /// The gateway's sentence when the proof is not available yet.
    pub message: Option<String>,
}
