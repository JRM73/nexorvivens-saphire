// =============================================================================
// knowledge/openlibrary.rs — Client API Open Library
// =============================================================================
//
// Role : Recherche de livres sur Open Library (openlibrary.org), la plus grande
//        bibliotheque ouverte au monde (~30M de fiches). Permet a Saphire de
//        decouvrir des livres, lire leurs descriptions et sujets, et enrichir
//        sa culture generale.
//
// API : https://openlibrary.org/search.json
//       - Gratuite, sans cle API
//       - Champs demandes : key, title, author_name, first_sentence, subject,
//         first_publish_year
//       - Limite a 3 resultats par requete
//
// Donnees retournees :
//   - Titre et auteur du livre
//   - Premiere phrase (quand disponible) — utile pour decouvrir le style
//   - Sujets/tags (jusqu'a 10) — utile pour les associations thematiques
//   - Annee de premiere publication
//
// Score de pertinence : 0.70 (donnees bibliographiques, pas de texte integral)
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recherche un livre sur Open Library.
    ///
    /// Processus :
    ///   1. Encoder la requete et envoyer a l'API de recherche
    ///   2. Parser la reponse JSON (champ "docs" = tableau de livres)
    ///   3. Extraire les metadonnees du premier resultat
    ///   4. Formater un extrait avec titre, auteur, premiere phrase et sujets
    ///
    /// Retourne les sujets dans section_titles pour enrichir le contexte.
    pub fn search_openlibrary(&self, query: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // Encoder la requete pour l'URL
        let encoded = Self::url_encode(query);
        let url = format!(
            "https://openlibrary.org/search.json?q={}&limit=3&fields=key,title,author_name,first_sentence,subject,first_publish_year",
            encoded
        );

        // Securite : verifier que le domaine est dans la whitelist
        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Envoyer la requete a l'API
        let resp_str = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; literary discovery)")
            .call()
            .map_err(|e| {
                tracing::warn!("Open Library echoue pour '{}': {}", query, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Parser la reponse JSON
        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Les resultats sont dans le champ "docs"
        let docs = resp["docs"]
            .as_array()
            .ok_or(KnowledgeError::NotFound)?;

        if docs.is_empty() {
            tracing::info!("Open Library: aucun resultat pour '{}'", query);
            return Err(KnowledgeError::NotFound);
        }

        // Prendre le premier resultat (le plus pertinent)
        let book = &docs[0];
        let title = book["title"].as_str().unwrap_or("").to_string();
        let author = book["author_name"][0].as_str().unwrap_or("Inconnu").to_string();
        let year = book["first_publish_year"].as_u64().unwrap_or(0);

        // La premiere phrase peut etre un tableau de strings ou un string simple
        // (l'API Open Library n'est pas toujours coherente sur ce champ)
        let first_sentence = book["first_sentence"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .or_else(|| book["first_sentence"].as_str())
            .unwrap_or("")
            .to_string();

        // Extraire les 10 premiers sujets/tags du livre
        let subjects: Vec<String> = book["subject"]
            .as_array()
            .map(|arr| arr.iter()
                .take(10)
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
            ).unwrap_or_default();

        // Formater l'extrait avec toutes les metadonnees disponibles
        let extract = format!(
            "Livre : {} par {} ({})\n\nPremière phrase : {}\n\nSujets : {}",
            title, author, year,
            if first_sentence.is_empty() { "(non disponible)" } else { &first_sentence },
            if subjects.is_empty() { "(aucun)".to_string() } else { subjects.join(", ") }
        );

        // Construire l'URL de la page Open Library
        let key = book["key"].as_str().unwrap_or("");
        let page_url = format!("https://openlibrary.org{}", key);

        tracing::info!("Open Library: '{}' par {} ({})", title, author, year);

        // Note : total_length est calcule AVANT le move de `extract`
        // pour eviter le "borrow of moved value"
        Ok(KnowledgeResult {
            source: format!("Open Library — {}", author),
            title: format!("{} ({})", title, year),
            url: page_url,
            total_length: extract.len(),
            extract, // move ici, apres le calcul de total_length
            section_titles: subjects, // Les sujets servent de "sections"
            relevance_score: 0.7, // Donnees bibliographiques, pas de texte integral
            fetched_at: Utc::now(),
        })
    }
}
