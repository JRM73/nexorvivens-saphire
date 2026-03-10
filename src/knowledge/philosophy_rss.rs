// =============================================================================
// knowledge/philosophy_rss.rs — Client RSS pour essais philosophiques
// =============================================================================
//
// Role : Recuperation d'essais philosophiques recents depuis des flux RSS.
//        Contrairement aux autres sources (recherche par mot-cle), ce module
//        recupere les DERNIERS articles publies, sans filtre de recherche.
//        C'est une source de "serendipite intellectuelle" pour Saphire.
//
// Sources RSS :
//   - Aeon.co : essais longs et profonds sur la conscience, l'existence,
//     la beaute, l'ethique, l'identite. Publies par des philosophes,
//     scientifiques et ecrivains.
//   - Daily Nous : blog academique sur l'actualite philosophique,
//     debats, publications, evenements dans le monde de la philosophie.
//
// Format RSS :
//   - <item><title>...<description>...<link>...</item>
//   - Le contenu peut etre encapsule dans CDATA : <![CDATA[...]]>
//   - Les descriptions contiennent souvent du HTML (balises <p>, <a>, etc.)
//   - On nettoie avec strip_cdata() puis strip_html_tags()
//
// Note User-Agent :
//   On utilise un User-Agent de navigateur (Mozilla/5.0) car certains
//   serveurs RSS rejettent les requetes avec des User-Agent generiques.
//
// Score de pertinence : 0.80
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recuperer les essais philosophiques recents via RSS (Aeon + Daily Nous).
    ///
    /// Processus :
    ///   1. Iterer sur les 2 flux RSS
    ///   2. Pour chaque flux, telecharger le XML
    ///   3. Parser les 3 premiers <item> de chaque flux
    ///   4. Extraire titre, lien et description (avec fallback sur content:encoded)
    ///   5. Nettoyer le contenu (CDATA + HTML)
    ///   6. Filtrer les articles trop courts (<100 chars de description)
    ///
    /// Retourne un Vec car on recupere jusqu'a 6 articles (3 par flux).
    /// Retourne KnowledgeError::NotFound si aucun article n'est trouve.
    pub fn search_philosophy_rss(&self) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // Les 2 flux RSS a interroger
        let feeds: &[(&str, &str)] = &[
            ("https://aeon.co/feed.rss", "Aeon"),
            ("https://dailynous.com/feed/", "Daily Nous"),
        ];

        let mut results = Vec::new();
        let max_chars = self.config.max_content_chars;

        for (url, source_name) in feeds {
            // Securite : verifier que le domaine est dans la whitelist
            if !Self::is_url_allowed(url) {
                continue;
            }

            // Telecharger le flux RSS
            // Note : User-Agent de navigateur pour eviter les rejets serveur
            let response = match self.http_client
                .get(url)
                .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
                .set("Accept", "application/rss+xml, application/xml, text/xml")
                .call()
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("RSS {} reponse illisible : {}", source_name, e);
                        continue; // Passer au flux suivant
                    }
                },
                Err(e) => {
                    tracing::warn!("RSS {} echoue : {}", source_name, e);
                    continue; // Passer au flux suivant
                }
            };

            // Parser les items RSS (on prend les 3 premiers)
            // Le XML est decoupes par "<item>" — skip(1) car le premier fragment
            // est l'en-tete du flux (avant le premier <item>)
            for item in response.split("<item>").skip(1).take(3) {
                // Extraire le titre (peut etre dans CDATA)
                let title = Self::extract_tag(item, "title")
                    .map(|t| Self::strip_cdata(&t))
                    .unwrap_or_default();

                // Extraire le lien de l'article
                let link = Self::extract_tag(item, "link")
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default();

                // Extraire la description avec fallback sur content:encoded
                // (certains flux mettent le contenu complet dans content:encoded
                //  et un resume court dans description)
                let desc_raw = Self::extract_tag(item, "description")
                    .map(|d| Self::strip_cdata(&d))
                    .filter(|d| d.len() > 20) // Ignorer les descriptions trop courtes
                    .or_else(|| {
                        // Fallback : utiliser content:encoded (contenu complet)
                        Self::extract_tag(item, "content:encoded")
                            .map(|d| Self::strip_cdata(&d))
                    })
                    .unwrap_or_default();

                // Nettoyer le HTML et tronquer a max_chars
                let desc_text = Self::strip_html_tags(&desc_raw)
                    .chars()
                    .take(max_chars)
                    .collect::<String>();

                // Ne garder que les articles avec titre et contenu substantiel
                if !title.is_empty() && desc_text.len() > 100 {
                    results.push(KnowledgeResult {
                        source: source_name.to_string(),
                        title: title.clone(),
                        url: link,
                        extract: desc_text,
                        section_titles: vec![], // Les RSS n'ont pas de sections
                        total_length: desc_raw.len(),
                        relevance_score: 0.80,
                        fetched_at: Utc::now(),
                    });
                }
            }
        }

        if results.is_empty() {
            tracing::info!("Philosophy RSS: aucun article trouve");
            Err(KnowledgeError::NotFound)
        } else {
            tracing::info!("Philosophy RSS: {} articles trouves", results.len());
            Ok(results)
        }
    }
}
