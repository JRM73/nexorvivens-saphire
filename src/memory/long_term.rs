// long_term.rs — Long-term memory (PostgreSQL + pgvector)
//
// This module represents the third and final tier of the Saphire memory
// system: long-term memory (LTM).
//
// LTM stores memories that have been consolidated from episodic memory, as
// well as "founding memories" — pre-programmed initial memories that define
// Saphire's baseline identity and core values. This is analogous to the
// neocortical storage of semanticized knowledge after hippocampal replay
// during sleep consolidation.
//
// Characteristics:
//   - Permanent: memories in LTM NEVER undergo strength decay. Once
//     consolidated via hippocampal-neocortical transfer, a memory persists
//     indefinitely (though it may eventually be archived if capacity is exceeded).
//   - Vector-indexed: each memory is indexed by a dense embedding vector in
//     pgvector (a PostgreSQL extension for vector similarity search), enabling
//     cosine similarity retrieval. This models content-addressable memory
//     in the neocortex, where recall is cue-driven rather than sequential.
//   - The vector encoding is delegated to the LocalEncoder (vectorstore module),
//     which uses TF-IDF (Term Frequency-Inverse Document Frequency) with
//     FNV-1a hashing for dimensionality-reduced feature extraction.
//
// Dependencies:
//   - crate::vectorstore::encoder::LocalEncoder: local vector encoder,
//     re-exported here for convenient access from other memory modules.
//
// Architectural note: this module is intentionally lightweight because the
// actual LTM storage and retrieval logic resides in crate::db (the database
// access layer). This module primarily serves as a re-export point and
// documentation anchor for the LTM tier.

pub use crate::vectorstore::encoder::LocalEncoder;
