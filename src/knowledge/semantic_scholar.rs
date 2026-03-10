// =============================================================================
// knowledge/semantic_scholar.rs — Client API Semantic Scholar
// =============================================================================
//
// Role : Recherche d'articles academiques sur Semantic Scholar, une base de
//        donnees gratuite couvrant 200M+ articles scientifiques (informatique,
//        neurosciences, psychologie, philosophie, biologie, etc.).
//
// API : https://api.semanticscholar.org/graph/v1/paper/search
//       - Gratuite, sans cle API
//       - Limite : 100 requetes / 5 minutes
//       - Champs demandes : title, abstract, url, authors, year, citationCount
//       - Retourne jusqu'a 3 resultats par requete
//
// Traduction : les requetes en francais sont traduites en anglais via
//              translate_query_to_english() (partage avec arxiv.rs).
//
// Score de pertinence : 0.85 + bonus citations (max +0.10 pour 1000+ citations)
//   Formule : 0.85 + min(citationCount / 1000, 0.10)
//   Un article avec 500 citations -> score 0.90
//   Un article avec 2000 citations -> score 0.95 (cap)
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recherche des articles academiques sur Semantic Scholar.
    ///
    /// Processus :
    ///   1. Traduire la requete FR -> EN (dictionnaire de termes courants)
    ///   2. Encoder l'URL et envoyer la requete a l'API Graph v1
    ///   3. Parser les resultats JSON (title, abstract, authors, year, citations)
    ///   4. Filtrer les articles sans abstract substantiel (<50 chars)
    ///   5. Calculer le score de pertinence (base + bonus citations)
    ///
    /// Retourne un Vec car l'API peut retourner jusqu'a 3 resultats.
    pub fn search_semantic_scholar(&self, query: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // Traduire la requete en anglais (la majorite des articles sont en EN)
        // Utilise la meme fonction que arxiv.rs (pub(crate))
        let english_query = Self::translate_query_to_english(query);
        let encoded = Self::url_encode(&english_query);

        // Construire l'URL de l'API avec les champs necessaires
        let url = format!(
            "https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit=3&fields=title,abstract,url,authors,year,citationCount",
            encoded
        );

        // Securite : verifier que le domaine est dans la whitelist
        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Envoyer la requete a l'API
        let resp_str = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; academic research)")
            .call()
            .map_err(|e| {
                tracing::warn!("Semantic Scholar echoue pour '{}': {}", english_query, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Parser la reponse JSON
        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Les resultats sont dans le champ "data" (tableau de papers)
        let papers = resp["data"]
            .as_array()
            .ok_or(KnowledgeError::NotFound)?;

        let max_chars = self.config.max_content_chars;

        // Transformer chaque paper en KnowledgeResult
        let results: Vec<KnowledgeResult> = papers.iter()
            .filter_map(|paper| {
                let title = paper["title"].as_str()?.to_string();
                let abstract_text = paper["abstract"].as_str()
                    .unwrap_or("").to_string();
                let paper_url = paper["url"].as_str()
                    .unwrap_or("").to_string();
                let year = paper["year"].as_u64().unwrap_or(0);
                let citations = paper["citationCount"].as_u64().unwrap_or(0);

                // Extraire les noms des auteurs
                let authors: Vec<String> = paper["authors"]
                    .as_array()
                    .map(|a| a.iter()
                        .filter_map(|auth| auth["name"].as_str().map(String::from))
                        .collect()
                    ).unwrap_or_default();

                // Ignorer les articles sans abstract substantiel
                if abstract_text.len() < 50 { return None; }

                let first_author = authors.first()
                    .cloned()
                    .unwrap_or_else(|| "Unknown".into());

                Some(KnowledgeResult {
                    source: format!("Semantic Scholar ({}, {} citations)", year, citations),
                    title: format!("{} — {}", title, first_author),
                    url: paper_url,
                    extract: abstract_text.chars().take(max_chars).collect(),
                    section_titles: vec![], // Les abstracts n'ont pas de sections
                    total_length: abstract_text.len(),
                    // Score de pertinence : base 0.85 + bonus proportionnel aux citations
                    // Le bonus est plafonné a 0.10 (pour 1000+ citations)
                    relevance_score: 0.85 + (citations as f64 / 1000.0).min(0.1),
                    fetched_at: Utc::now(),
                })
            })
            .collect();

        if results.is_empty() {
            tracing::info!("Semantic Scholar: aucun resultat pour '{}' (traduit: '{}')",
                query, english_query);
            Err(KnowledgeError::NotFound)
        } else {
            tracing::info!("Semantic Scholar: {} resultats pour '{}' (traduit: '{}')",
                results.len(), query, english_query);
            Ok(results)
        }
    }
}
