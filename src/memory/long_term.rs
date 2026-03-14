// long_term.rs — Mémoire à long terme (PostgreSQL + pgvector)
//
// Ce module représente le troisième et dernier niveau du système mnésique
// de Saphire : la mémoire à long terme (LTM = Long-Term Memory).
//
// La LTM stocke les souvenirs consolidés depuis la mémoire épisodique
// ainsi que les « founding memories » (souvenirs fondateurs = souvenirs
// initiaux programmés qui définissent l'identité de base de Saphire).
//
// Caractéristiques :
//   - Permanente : les souvenirs ici ne subissent JAMAIS de décroissance.
//     Une fois consolidé, un souvenir reste indéfiniment.
//   - Vectorielle : chaque souvenir est indexé par un vecteur d'embedding
//     dans pgvector (extension PostgreSQL pour la recherche vectorielle),
//     permettant la recherche par similarité cosinus.
//   - L'encodage vectoriel est délégué au TextEncoder (module vectorstore),
//     qui peut être un OllamaEncoder (sémantique, 768-dim) ou un
//     LocalEncoder (FNV-1a, fallback) selon la disponibilité d'Ollama.
//
// Dépendances :
//   - crate::vectorstore::encoder::TextEncoder : trait d'encodage vectoriel
//     réexporté ici pour un accès pratique depuis les autres modules mémoire.

pub use crate::vectorstore::encoder::LocalEncoder;
pub use crate::vectorstore::encoder::TextEncoder;
