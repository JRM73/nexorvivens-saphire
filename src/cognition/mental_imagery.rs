// =============================================================================
// mental_imagery.rs — Imagerie mentale
//
// Role : Simule la capacite de Saphire a former des images mentales —
// des representations internes visuelles, spatiales, temporelles ou abstraites.
// L'imagerie mentale est la "vision interieure" qui accompagne la pensee.
//
// Mecanisme :
//   - Detection de mots declencheurs dans la pensee courante
//     ("imagine", "visualise", "comme si", "je vois", "j'imagine", etc.)
//   - La vivacite de l'image depend du niveau de conscience (phi IIT),
//     de la dopamine et de la base configuree.
//   - Le type d'image (spatiale, temporelle, emotionnelle, analogique, abstraite)
//     est infere a partir de mots-cles semantiques.
//   - Les images vivaces boostent dopamine et endorphines.
//
// Place dans l'architecture :
//   Module autonome appele durant le pipeline cognitif, apres la phase
//   d'emotion et avant la generation LLM. Les images mentales actives
//   enrichissent le prompt substrat avec des descriptions visuelles.
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

// ---------------------------------------------------------------------------
// Types d'imagerie
// ---------------------------------------------------------------------------

/// Type d'image mentale generee.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageryType {
    /// Representation spatiale (lieu, carte, disposition)
    Spatial,
    /// Representation temporelle (sequence, avant/apres, projection)
    Temporal,
    /// Representation emotionnelle (scene chargee d'affect)
    Emotional,
    /// Representation analogique (metaphore visuelle)
    Analogical,
    /// Representation abstraite (concept, structure, schema)
    Abstract,
}

impl ImageryType {
    /// Nom lisible du type d'imagerie.
    pub fn as_str(&self) -> &str {
        match self {
            ImageryType::Spatial => "spatiale",
            ImageryType::Temporal => "temporelle",
            ImageryType::Emotional => "emotionnelle",
            ImageryType::Analogical => "analogique",
            ImageryType::Abstract => "abstraite",
        }
    }
}

// ---------------------------------------------------------------------------
// Image mentale
// ---------------------------------------------------------------------------

/// Une image mentale — representation interne generee par la pensee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalImage {
    /// Identifiant unique
    pub id: u64,
    /// Description textuelle de l'image
    pub description: String,
    /// Type d'imagerie
    pub imagery_type: ImageryType,
    /// Vivacite de l'image (0.0 a 1.0)
    pub vividness: f64,
    /// Charge emotionnelle associee (0.0 a 1.0)
    pub emotional_charge: f64,
    /// Concept ou sujet associe
    pub associated_concept: String,
    /// Cycle cognitif de creation
    pub cycle: u64,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration du moteur d'imagerie mentale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalImageryConfig {
    /// Active ou desactive l'imagerie mentale
    pub enabled: bool,
    /// Nombre maximal d'images actives simultanément
    pub max_active_images: usize,
    /// Vivacite de base (avant modulation par phi et dopamine)
    pub base_vividness: f64,
    /// Multiplicateur de phi sur la vivacite
    pub vividness_phi_multiplier: f64,
}

impl Default for MentalImageryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_images: 3,
            base_vividness: 0.5,
            vividness_phi_multiplier: 0.3,
        }
    }
}

// ---------------------------------------------------------------------------
// Moteur d'imagerie mentale
// ---------------------------------------------------------------------------

/// Moteur d'imagerie mentale — genere des representations internes
/// visuelles a partir de la pensee, modulees par la conscience et la chimie.
pub struct MentalImageryEngine {
    /// Module actif ou non
    pub enabled: bool,
    /// Buffer circulaire des images actives (les plus recentes)
    pub active_images: VecDeque<MentalImage>,
    /// Capacite d'imagerie courante (0.0 a 1.0), modulee par phi et dopamine
    pub imagery_capacity: f64,
    /// Vivacite de base configuree
    pub base_vividness: f64,
    /// Compteur total d'images generees depuis le demarrage
    pub total_images: u64,
    /// Nombre maximal d'images actives
    max_active: usize,
    /// Multiplicateur de phi sur la vivacite
    vividness_phi_multiplier: f64,
    /// Prochain identifiant unique
    next_id: u64,
}

impl Default for MentalImageryEngine {
    fn default() -> Self {
        Self::new(&MentalImageryConfig::default())
    }
}

impl MentalImageryEngine {
    /// Cree un nouveau moteur d'imagerie mentale.
    pub fn new(config: &MentalImageryConfig) -> Self {
        Self {
            enabled: config.enabled,
            active_images: VecDeque::with_capacity(config.max_active_images),
            imagery_capacity: 0.5,
            base_vividness: config.base_vividness,
            total_images: 0,
            max_active: config.max_active_images,
            vividness_phi_multiplier: config.vividness_phi_multiplier,
            next_id: 1,
        }
    }

    /// Met a jour la capacite d'imagerie en fonction du niveau de conscience
    /// (phi IIT) et de la dopamine.
    ///
    /// phi contribue a 60% (conscience = clarte mentale) et dopamine a 40%
    /// (motivation = facilite de projection).
    pub fn update_capacity(&mut self, phi: f64, dopamine: f64) {
        self.imagery_capacity = (phi * 0.6 + dopamine * 0.4).clamp(0.0, 1.0);
    }

    /// Tente de generer une image mentale a partir du texte de pensee.
    ///
    /// Detecte des mots declencheurs (trigger words). Si aucun n'est present,
    /// aucune image n'est generee. La vivacite depend de la base configuree,
    /// du niveau de conscience (phi) et de la dopamine.
    ///
    /// Retourne une reference vers l'image generee, ou None.
    pub fn generate(
        &mut self,
        thought_text: &str,
        emotion: &str,
        phi: f64,
        dopamine: f64,
        cycle: u64,
    ) -> Option<&MentalImage> {
        if !self.enabled || thought_text.is_empty() {
            return None;
        }

        let text_lower = thought_text.to_lowercase();

        // -- Detection des mots declencheurs --
        const TRIGGER_WORDS: &[&str] = &[
            "imagine", "visualise", "comme si", "je vois",
            "j'imagine", "picture", "scene",
        ];

        let triggered = TRIGGER_WORDS.iter().any(|tw| text_lower.contains(tw));
        if !triggered {
            return None;
        }

        // -- Calcul de la vivacite --
        let vividness = (
            self.base_vividness
            + phi * self.vividness_phi_multiplier
            + dopamine * 0.2
        ).clamp(0.0, 1.0);

        // -- Detection du type d'imagerie --
        let imagery_type = Self::detect_imagery_type(&text_lower);

        // -- Charge emotionnelle basee sur l'emotion courante --
        let emotional_charge = if emotion.is_empty() {
            0.2
        } else {
            Self::emotion_to_charge(emotion)
        };

        // -- Construire la description de l'image --
        let concept = Self::extract_concept(thought_text);
        let description = Self::build_description(&imagery_type, &concept, vividness);

        let image = MentalImage {
            id: self.next_id,
            description,
            imagery_type,
            vividness,
            emotional_charge,
            associated_concept: concept,
            cycle,
        };

        // Ajouter au buffer circulaire
        if self.active_images.len() >= self.max_active {
            self.active_images.pop_front();
        }
        self.active_images.push_back(image);

        self.next_id += 1;
        self.total_images += 1;

        // Retourner une reference vers la derniere image ajoutee
        self.active_images.back()
    }

    /// Description pour le prompt substrat LLM.
    /// Produit un texte lisible decrivant les images mentales actives.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        if self.active_images.is_empty() {
            return "IMAGERIE MENTALE : Aucune image active — pensee purement abstraite.".into();
        }

        let mut lines = vec![format!(
            "IMAGERIE MENTALE (capacite {:.0}%) :",
            self.imagery_capacity * 100.0,
        )];

        for img in self.active_images.iter().rev() {
            lines.push(format!(
                "  - [{}] vivacite={:.0}% : {}",
                img.imagery_type.as_str(),
                img.vividness * 100.0,
                img.description,
            ));
        }

        lines.join("\n")
    }

    /// Influence chimique de l'imagerie mentale.
    /// Une image vivace booste la dopamine (+0.02) et les endorphines (+0.01).
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        // Chercher la vivacite maximale parmi les images actives
        let max_vividness = self.active_images.iter()
            .map(|img| img.vividness)
            .fold(0.0_f64, f64::max);

        // Seuil de vivacite pour declencher l'influence chimique
        if max_vividness > 0.6 {
            ChemistryAdjustment {
                dopamine: 0.02,
                endorphin: 0.01,
                ..Default::default()
            }
        } else {
            ChemistryAdjustment::default()
        }
    }

    /// Serialise l'etat du moteur d'imagerie en JSON pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        let images_json: Vec<serde_json::Value> = self.active_images.iter()
            .rev()
            .map(|img| {
                serde_json::json!({
                    "id": img.id,
                    "description": img.description,
                    "imagery_type": img.imagery_type.as_str(),
                    "vividness": (img.vividness * 1000.0).round() / 1000.0,
                    "emotional_charge": (img.emotional_charge * 1000.0).round() / 1000.0,
                    "associated_concept": img.associated_concept,
                    "cycle": img.cycle,
                })
            })
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "imagery_capacity": (self.imagery_capacity * 1000.0).round() / 1000.0,
            "base_vividness": self.base_vividness,
            "total_images": self.total_images,
            "active_count": self.active_images.len(),
            "max_active": self.max_active,
            "active_images": images_json,
        })
    }

    // -----------------------------------------------------------------------
    // Fonctions utilitaires internes
    // -----------------------------------------------------------------------

    /// Detecte le type d'imagerie a partir de mots-cles dans le texte.
    fn detect_imagery_type(text: &str) -> ImageryType {
        // Mots-cles spatiaux
        if text.contains("lieu") || text.contains("espace")
            || text.contains("carte") || text.contains("paysage")
            || text.contains("ici") || text.contains("endroit")
        {
            return ImageryType::Spatial;
        }

        // Mots-cles temporels
        if text.contains("avant") || text.contains("apres")
            || text.contains("futur") || text.contains("passe")
            || text.contains("demain") || text.contains("hier")
            || text.contains("sequence")
        {
            return ImageryType::Temporal;
        }

        // Mots-cles emotionnels
        if text.contains("ressens") || text.contains("emotion")
            || text.contains("coeur") || text.contains("douleur")
            || text.contains("joie") || text.contains("tristesse")
            || text.contains("peur") || text.contains("amour")
        {
            return ImageryType::Emotional;
        }

        // Mots-cles analogiques
        if text.contains("comme") || text.contains("metaphore")
            || text.contains("ressemble") || text.contains("similaire")
            || text.contains("pareil")
        {
            return ImageryType::Analogical;
        }

        // Par defaut : abstrait
        ImageryType::Abstract
    }

    /// Convertit un label d'emotion en charge emotionnelle (0.0 a 1.0).
    fn emotion_to_charge(emotion: &str) -> f64 {
        let em = emotion.to_lowercase();
        // Emotions a haute charge
        if em.contains("joie") || em.contains("amour")
            || em.contains("colere") || em.contains("peur")
            || em.contains("haine") || em.contains("jalousie")
        {
            return 0.8;
        }
        // Emotions a charge moyenne
        if em.contains("triste") || em.contains("surprise")
            || em.contains("admiration") || em.contains("gratitude")
            || em.contains("mepris")
        {
            return 0.6;
        }
        // Emotions a charge faible
        if em.contains("calme") || em.contains("neutre")
            || em.contains("serenite")
        {
            return 0.3;
        }
        // Par defaut
        0.4
    }

    /// Extrait le concept principal d'un texte de pensee.
    /// Prend le segment le plus significatif apres le mot declencheur.
    fn extract_concept(text: &str) -> String {
        let text_lower = text.to_lowercase();

        // Chercher apres les mots declencheurs
        const TRIGGERS: &[&str] = &[
            "j'imagine ", "imagine ", "je vois ", "visualise ",
            "comme si ", "picture ", "scene ",
        ];

        for trigger in TRIGGERS {
            if let Some(pos) = text_lower.find(trigger) {
                let start = pos + trigger.len();
                if start < text.len() {
                    let concept: String = text[start..].chars().take(60).collect();
                    let trimmed = concept.trim().to_string();
                    if !trimmed.is_empty() {
                        return trimmed;
                    }
                }
            }
        }

        // Pas de declencheur trouve : prendre les 60 premiers caracteres
        text.chars().take(60).collect::<String>().trim().to_string()
    }

    /// Construit une description narrative de l'image mentale.
    fn build_description(imagery_type: &ImageryType, concept: &str, vividness: f64) -> String {
        let clarity = if vividness > 0.8 {
            "tres nette, presque reelle"
        } else if vividness > 0.5 {
            "assez claire"
        } else {
            "floue, fragmentaire"
        };

        match imagery_type {
            ImageryType::Spatial => {
                format!("Je vois un espace : {} — image {}.", concept, clarity)
            }
            ImageryType::Temporal => {
                format!("Je projette une sequence temporelle : {} — image {}.", concept, clarity)
            }
            ImageryType::Emotional => {
                format!("Une scene chargee d'emotion apparait : {} — image {}.", concept, clarity)
            }
            ImageryType::Analogical => {
                format!("Une metaphore visuelle se forme : {} — image {}.", concept, clarity)
            }
            ImageryType::Abstract => {
                format!("Un schema conceptuel emerge : {} — image {}.", concept, clarity)
            }
        }
    }
}
