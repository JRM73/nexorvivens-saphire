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
//   - L'encodage vectoriel est délégué au LocalEncoder (module vectorstore),
//     qui utilise TF-IDF (Term Frequency-Inverse Document Frequency =
//     Fréquence du Terme - Fréquence Inverse du Document) avec hachage FNV-1a.
//
// Dépendances :
//   - crate::vectorstore::encoder::LocalEncoder : encodeur vectoriel local
//     réexporté ici pour un accès pratique depuis les autres modules mémoire.
//
// Note architecturale : ce module est volontairement léger car la logique
// de stockage et recherche en LTM est implémentée dans crate::db (couche DB).
// Il sert principalement de réexportation et de point de documentation.

pub use crate::vectorstore::encoder::LocalEncoder;
