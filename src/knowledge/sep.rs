// =============================================================================
// knowledge/sep.rs — Client Stanford Encyclopedia of Philosophy (SEP)
// =============================================================================
//
// Role : Recherche d'articles de philosophie sur la SEP (plato.stanford.edu).
//        La SEP est une encyclopedie en ligne gratuite et redigee par des
//        experts en philosophie. Elle n'a pas d'API formelle mais ses articles
//        sont en HTML simple avec une structure previsible (preamble + sections h2).
//
// Approche :
//   1. Mapping de 50+ sujets courants (FR/EN) vers les slugs SEP
//      (ex: "conscience" -> "consciousness")
//   2. Telechargement de la page HTML de l'article
//   3. Extraction des paragraphes <p> entre "preamble"/"main-text" et "Bib"
//   4. Rotation des sections a chaque relecture (read_count * 3 paragraphes)
//
// Particularite technique :
//   On ne peut PAS utiliser strip_html_tags() puis split("\n\n") car
//   strip_html_tags() ecrase tous les sauts de ligne en un seul espace.
//   On parse donc les balises <p> directement depuis le HTML brut.
//
// Score de pertinence : 0.95 (tres haute, source academique de reference)
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recherche un article sur la Stanford Encyclopedia of Philosophy.
    ///
    /// Processus :
    ///   1. Convertir le sujet en slug SEP via le mapping interne
    ///   2. Telecharger la page HTML a plato.stanford.edu/entries/{slug}/
    ///   3. Extraire le titre (balise <title>)
    ///   4. Extraire les paragraphes avec rotation (anti-repetition)
    ///   5. Extraire les titres de sections (balises <h2>)
    ///
    /// Retourne KnowledgeError::NotFound si le contenu est < 100 chars.
    pub fn search_sep(&self, query: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // Convertir le sujet en slug SEP (ex: "libre arbitre" -> "freewill")
        let slug = Self::sep_topic_to_slug(query);
        let url = format!("https://plato.stanford.edu/entries/{}/", slug);

        // Securite : verifier que le domaine est dans la whitelist
        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Telecharger la page HTML de l'article SEP
        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; philosophical research)")
            .call()
            .map_err(|e| {
                tracing::warn!("SEP requete echouee pour '{}': {}", slug, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Extraire le titre de l'article (retire le suffixe SEP)
        let title = Self::extract_sep_title(&response)
            .unwrap_or_else(|| query.to_string());

        // Calculer l'offset de rotation pour ce slug
        // Chaque relecture avance de 3 paragraphes dans l'article
        let read_count = self.article_read_count
            .get(&format!("sep:{}", slug))
            .copied()
            .unwrap_or(0);

        let max_chars = self.config.max_content_chars;

        // Extraire le contenu principal avec rotation basee sur read_count
        let preamble = Self::extract_sep_content(&response, read_count, max_chars);

        // Rejeter si le contenu extrait est trop court (article introuvable ou vide)
        if preamble.len() < 100 {
            tracing::warn!("SEP: contenu trop court pour '{}'", slug);
            return Err(KnowledgeError::NotFound);
        }

        // Extraire les titres de sections h2 (plan de l'article)
        let sections = Self::extract_sep_sections(&response);

        // Calculer la taille totale de l'article (texte brut)
        let text_len = Self::strip_html_tags(&response).len();

        tracing::info!(
            "SEP: '{}' ({} sections, {} chars, offset {})",
            title, sections.len(), text_len, read_count
        );

        Ok(KnowledgeResult {
            source: "Stanford Encyclopedia of Philosophy".into(),
            title,
            url,
            extract: preamble,
            section_titles: sections,
            total_length: text_len,
            relevance_score: 0.95, // Source academique de tres haute qualite
            fetched_at: Utc::now(),
        })
    }

    /// Convertir un sujet en slug SEP via un dictionnaire de 50+ mappings.
    ///
    /// Le dictionnaire couvre 5 domaines philosophiques :
    ///   - Conscience et esprit (consciousness, qualia, philosophy-mind, ...)
    ///   - Libre arbitre et ethique (freewill, ethics-virtue, utilitarianism, ...)
    ///   - Existence et ontologie (existentialism, phenomenology, identity-personal, ...)
    ///   - IA et cognition (artificial-intelligence, turing-test, chinese-room, ...)
    ///   - Emotions et empathie (emotion, empathy, love, beauty)
    ///   - Epistemologie (knowledge-analysis, truth, certainty, skepticism)
    ///
    /// Si aucun mapping n'est trouve, on convertit la requete en slug brut
    /// (minuscules, espaces -> tirets, accents -> ASCII).
    fn sep_topic_to_slug(query: &str) -> String {
        let mappings: &[(&str, &str)] = &[
            // --- Conscience et esprit ---
            ("conscience de soi", "self-consciousness"),
            ("conscience", "consciousness"),
            ("consciousness", "consciousness"),
            ("qualia", "qualia"),
            ("problème difficile", "consciousness"),
            ("hard problem", "consciousness"),
            ("self-consciousness", "self-consciousness"),
            ("philosophie de l'esprit", "philosophy-mind"),
            ("philosophy of mind", "philosophy-mind"),
            ("dualisme", "dualism"),
            ("monisme", "monism"),
            ("fonctionnalisme", "functionalism"),
            ("panpsychisme", "panpsychism"),
            ("intentionnalité", "intentionality"),
            ("représentation mentale", "mental-representation"),
            // --- Libre arbitre et ethique ---
            ("libre arbitre", "freewill"),
            ("free will", "freewill"),
            ("déterminisme", "determinism-causal"),
            ("compatibilisme", "compatibilism"),
            ("responsabilité morale", "moral-responsibility"),
            ("éthique de la vertu", "ethics-virtue"),
            ("éthique", "ethics-virtue"),
            ("morale", "morality-definition"),
            ("impératif catégorique", "kant-moral"),
            ("utilitarisme", "utilitarianism-history"),
            ("déontologie", "ethics-deontological"),
            ("vertu", "ethics-virtue"),
            // --- Existence et ontologie ---
            ("existentialisme", "existentialism"),
            ("phénoménologie", "phenomenology"),
            ("husserl", "husserl"),
            ("être", "existence"),
            ("identité personnelle", "identity-personal"),
            ("personal identity", "identity-personal"),
            ("temps", "time"),
            ("causalité", "causation-metaphysics"),
            // --- IA et cognition ---
            ("intelligence artificielle", "artificial-intelligence"),
            ("test de turing", "turing-test"),
            ("chambre chinoise", "chinese-room"),
            ("computational mind", "computational-mind"),
            ("embodied cognition", "embodied-cognition"),
            ("cognition située", "situated-cognition"),
            // --- Emotions et empathie ---
            ("émotions", "emotion"),
            ("empathie", "empathy"),
            ("amour", "love"),
            ("beauté", "beauty"),
            // --- Epistemologie ---
            ("connaissance", "knowledge-analysis"),
            ("vérité", "truth"),
            ("certitude", "certainty"),
            ("scepticisme", "skepticism"),
            // --- Cas special ---
            ("zombie", "zombies"),
        ];

        // Recherche case-insensitive dans les mappings
        let lower = query.to_lowercase();
        for (key, slug) in mappings {
            if lower.contains(key) {
                return slug.to_string();
            }
        }

        // Fallback : transformer la requete en slug URL-compatible
        // (minuscules, espaces -> tirets, suppression des accents)
        query.to_lowercase()
            .replace(' ', "-")
            .replace(['é', 'è', 'ê'], "e")
            .replace('à', "a")
            .replace('ô', "o")
            .replace('î', "i")
            .replace('ù', "u")
    }

    /// Extraire le contenu principal d'un article SEP avec rotation.
    ///
    /// Algorithme :
    ///   1. Parser les balises <p> du HTML brut (pas strip_html_tags d'abord !)
    ///   2. Ne garder que les <p> entre "preamble"/"main-text" et "Bib"/"bibliography"
    ///   3. Filtrer les paragraphes de moins de 80 chars (notes, refs)
    ///   4. Appliquer la rotation : commencer au paragraphe (read_count * 3)
    ///   5. Concatener jusqu'a max_chars caracteres
    ///
    /// La rotation permet de lire des sections differentes de l'article
    /// a chaque visite, evitant que Saphire relise toujours l'introduction.
    fn extract_sep_content(html: &str, read_count: u32, max_chars: usize) -> String {
        // --- Phase 1 : extraction des paragraphes depuis le HTML brut ---
        let mut paragraphs: Vec<String> = Vec::new();
        let mut in_main = false; // Flag : on est dans le contenu principal

        // On itere sur les fragments separes par "<p" (chaque fragment = un <p>...</p>)
        for chunk in html.split("<p") {
            // Detecter le debut du contenu principal (preamble ou main-text)
            if chunk.contains("id=\"preamble\"") || chunk.contains("id=\"main-text\"") {
                in_main = true;
            }
            // Arreter avant la bibliographie et les entrees liees
            if chunk.contains("id=\"Bib\"") || chunk.contains("id=\"bibliography\"")
                || chunk.contains("Related Entries")
            {
                break;
            }

            // Ignorer les chunks avant le contenu principal
            if !in_main { continue; }

            // Extraire le texte entre > et </p>
            // Exemple : " class='intro'>Le contenu ici</p>..."
            //            ^start              ^end
            if let Some(start) = chunk.find('>') {
                let content = &chunk[start + 1..];
                if let Some(end) = content.find("</p>") {
                    let raw = &content[..end];
                    // Nettoyer le HTML interne (balises <em>, <a>, etc.)
                    let clean = Self::strip_html_tags(raw);
                    // Garder seulement les paragraphes substantiels (>80 chars)
                    if clean.len() > 80 {
                        paragraphs.push(clean);
                    }
                }
            }
        }

        // Fallback si aucun paragraphe n'a ete extrait
        if paragraphs.is_empty() {
            let text = Self::strip_html_tags(html);
            return text.chars().take(max_chars).collect();
        }

        // --- Phase 2 : rotation et assemblage ---
        // Avancer de 3 paragraphes a chaque relecture du meme article
        let start_para = (read_count as usize * 3) % paragraphs.len().max(1);
        let mut result = String::new();
        for para in paragraphs.iter().skip(start_para) {
            // Arreter si on depasse la limite de caracteres
            if result.len() + para.len() > max_chars { break; }
            if !result.is_empty() { result.push_str("\n\n"); }
            result.push_str(para);
        }
        result
    }

    /// Extraire le titre d'un article SEP depuis la balise <title>.
    /// Retire les suffixes standards SEP pour un titre propre.
    fn extract_sep_title(html: &str) -> Option<String> {
        Self::extract_tag(html, "title")
            .map(|t| t.replace(" (Stanford Encyclopedia of Philosophy)", "")
                .replace(" - Stanford Encyclopedia of Philosophy", "")
                .trim().to_string())
            .filter(|t| !t.is_empty())
    }

    /// Extraire les titres de sections (balises <h2>) d'un article SEP.
    /// Utile pour connaitre le plan de l'article et les themes abordes.
    fn extract_sep_sections(html: &str) -> Vec<String> {
        let mut sections = Vec::new();
        // Iterer sur les fragments separes par "<h2"
        for h2_block in html.split("<h2").skip(1) {
            if let Some(end) = h2_block.find("</h2>") {
                let content = &h2_block[..end];
                // Trouver le dernier '>' pour ignorer les attributs de la balise
                if let Some(start) = content.rfind('>') {
                    let title = content[start + 1..].trim().to_string();
                    // Filtrer les titres vides ou trop longs (artefacts)
                    if !title.is_empty() && title.len() < 100 {
                        sections.push(title);
                    }
                }
            }
        }
        sections
    }
}
