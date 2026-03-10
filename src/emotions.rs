// =============================================================================
// emotions.rs — 36 émotions émergentes + Mood (EMA - Exponential Moving Average)
// =============================================================================
//
// Rôle : Ce fichier calcule l'état émotionnel de Saphire à partir de son
// état neurochimique. Les émotions ne sont pas codées en dur : elles émergent
// de la similarité cosinus entre le vecteur chimique actuel et les « recettes »
// chimiques de 36 émotions prédéfinies.
//
// Dépendances :
//   - serde : sérialisation / désérialisation
//   - crate::neurochemistry::NeuroChemicalState : vecteur chimique à 7 dimensions
//
// Place dans l'architecture :
//   Ce module est consulté après chaque cycle de traitement pour déterminer
//   l'émotion dominante. Il est lu par :
//     - consciousness.rs (le monologue intérieur utilise l'émotion dominante)
//     - le moteur principal (affichage de l'état émotionnel)
//   Le Mood (humeur de fond) lisse les émotions instantanées sur le temps
//   via une EMA (Moyenne Mobile Exponentielle).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;

/// Profil d'une émotion : sa « recette » chimique et ses caractéristiques
/// psychologiques (valence et arousal).
///
/// La recette est un vecteur de 7 valeurs correspondant aux 7 neurotransmetteurs.
/// On compare ce vecteur au vecteur chimique actuel via similarité cosinus
/// pour déterminer à quel point l'état actuel « ressemble » à cette émotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionProfile {
    /// Nom de l'émotion (ex: "Joie", "Peur", "Curiosité")
    pub name: String,
    /// Recette chimique : [dopamine, cortisol, sérotonine, adrénaline,
    /// ocytocine, endorphine, noradrénaline] — chaque valeur entre 0.0 et 1.0
    pub recipe: [f64; 7],
    /// Valence [-1, +1] : dimension plaisant/déplaisant.
    /// Négatif = émotion désagréable, positif = émotion agréable.
    pub valence: f64,
    /// Arousal [0, 1] : niveau d'activation physiologique.
    /// 0 = calme, 1 = très activé.
    pub arousal: f64,
}

/// Resultat du calcul emotionnel — determine a chaque cycle de traitement.
///
/// Contient l'emotion dominante, une eventuelle emotion secondaire,
/// ainsi que la valence et l'arousal globaux calcules par moyenne ponderee.
/// Inclut le core affect (Barrett 2017) et le momentum emotionnel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Nom de l'emotion dominante (celle ayant la plus haute similarite cosinus)
    pub dominant: String,
    /// Score de similarite cosinus de l'emotion dominante [0, 1]
    pub dominant_similarity: f64,
    /// Emotion secondaire : presente uniquement si sa similarite depasse 0.5
    pub secondary: Option<String>,
    /// Valence globale [-1, +1] : moyenne ponderee des 3 emotions les plus proches
    pub valence: f64,
    /// Arousal global [0, 1] : moyenne ponderee des 3 emotions les plus proches
    pub arousal: f64,
    /// Spectre complet : liste de toutes les 36 emotions avec leur score
    /// de similarite, triees par ordre decroissant
    pub spectrum: Vec<(String, f64)>,
    /// Core Affect brut (Barrett) — valence et arousal directement de la chimie,
    /// AVANT la categorisation en emotion discrete. C'est l'etat affectif
    /// fondamental, pre-conceptuel.
    #[serde(default)]
    pub core_valence: f64,
    /// Core arousal brut
    #[serde(default)]
    pub core_arousal: f64,
    /// Contexte qui a influence la categorisation emotionnelle
    #[serde(default)]
    pub context_influence: String,
}

/// Humeur de fond — lissée par EMA (Exponential Moving Average,
/// Moyenne Mobile Exponentielle).
///
/// Contrairement à l'émotion instantanée, le Mood évolue lentement et
/// représente l'état affectif de fond de Saphire sur plusieurs cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mood {
    /// Valence lissée [-1, +1] : tendance plaisante ou déplaisante
    pub valence: f64,
    /// Arousal lissé [0, 1] : niveau d'activation moyen
    pub arousal: f64,
    /// Coefficient de lissage alpha [0.01, 0.5] : plus il est bas, plus
    /// le mood change lentement (forte inertie)
    pub alpha: f64,
}

impl Mood {
    /// Crée un nouveau Mood avec un coefficient de lissage donné.
    ///
    /// # Paramètres
    /// - `alpha` : coefficient EMA. Borné entre 0.01 (très lent) et 0.5 (réactif).
    ///
    /// # Retour
    /// Un Mood neutre (valence = 0.0, arousal = 0.3).
    pub fn new(alpha: f64) -> Self {
        Self {
            valence: 0.0,
            arousal: 0.3,
            alpha: alpha.clamp(0.01, 0.5),
        }
    }

    /// Met à jour le mood par EMA (Moyenne Mobile Exponentielle).
    /// Formule : mood = mood_ancien * (1 - alpha) + valeur_courante * alpha.
    /// Cela produit un lissage temporel : les émotions passagères influencent
    /// peu le mood, tandis que les états répétés le déplacent progressivement.
    ///
    /// # Paramètres
    /// - `valence` : valence instantanée de l'émotion courante [-1, +1].
    /// - `arousal` : arousal instantané de l'émotion courante [0, 1].
    pub fn update(&mut self, valence: f64, arousal: f64) {
        self.valence = self.valence * (1.0 - self.alpha) + valence * self.alpha;
        self.arousal = self.arousal * (1.0 - self.alpha) + arousal * self.alpha;
        self.valence = self.valence.clamp(-1.0, 1.0);
        self.arousal = self.arousal.clamp(0.0, 1.0);
    }

    /// Description textuelle du mood courant, basée sur le croisement
    /// de la valence (positif/négatif) et de l'arousal (activé/calme).
    /// Utilise le modèle circumplex de Russell (2 axes : valence x arousal).
    ///
    /// # Retour
    /// Une chaîne décrivant l'humeur : "Enthousiaste", "Sereine",
    /// "Agitée", "Morose", "Alerte" ou "Neutre".
    pub fn description(&self) -> &str {
        match (self.valence > 0.2, self.valence < -0.2, self.arousal > 0.5) {
            (true, _, true) => "Enthousiaste",   // valence positive + arousal élevé
            (true, _, false) => "Sereine",        // valence positive + arousal bas
            (_, true, true) => "Agitée",          // valence négative + arousal élevé
            (_, true, false) => "Morose",         // valence négative + arousal bas
            _ if self.arousal > 0.5 => "Alerte",  // valence neutre + arousal élevé
            _ => "Neutre",                         // valence neutre + arousal bas
        }
    }
}

/// Momentum emotionnel — inertie des emotions (Barrett 2017, constructionnisme).
/// Les emotions ne changent pas instantanement : elles ont de l'inertie.
/// Un etat emotionnel fort persiste pendant plusieurs cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionMomentum {
    /// Emotion precedente
    pub prev_dominant: String,
    /// Valence precedente
    pub prev_valence: f64,
    /// Arousal precedent
    pub prev_arousal: f64,
    /// Inertie [0.0, 0.8] : 0 = pas d'inertie, 0.8 = forte inertie
    pub inertia: f64,
    /// Compteur de cycles avec la meme emotion dominante
    pub stability_count: u64,
}

impl Default for EmotionMomentum {
    fn default() -> Self {
        Self {
            prev_dominant: "Neutre".to_string(),
            prev_valence: 0.0,
            prev_arousal: 0.3,
            inertia: 0.3, // Inertie moderee par defaut
            stability_count: 0,
        }
    }
}

impl EmotionMomentum {
    /// Applique le momentum a une valence et arousal brutes.
    /// Plus l'inertie est forte, plus l'etat precedent influence le resultat.
    pub fn apply(&mut self, raw_dominant: &str, raw_valence: f64, raw_arousal: f64) -> (f64, f64) {
        let smoothed_valence = self.prev_valence * self.inertia + raw_valence * (1.0 - self.inertia);
        let smoothed_arousal = self.prev_arousal * self.inertia + raw_arousal * (1.0 - self.inertia);

        // Compter la stabilite
        if raw_dominant == self.prev_dominant {
            self.stability_count += 1;
            // Plus une emotion est stable, plus elle a d'inertie (enracinement)
            self.inertia = (self.inertia + 0.005).min(0.6);
        } else {
            self.stability_count = 0;
            // Changement d'emotion : l'inertie diminue pour permettre la transition
            self.inertia = (self.inertia - 0.02).max(0.15);
        }

        self.prev_dominant = raw_dominant.to_string();
        self.prev_valence = smoothed_valence;
        self.prev_arousal = smoothed_arousal;

        (smoothed_valence.clamp(-1.0, 1.0), smoothed_arousal.clamp(0.0, 1.0))
    }
}

/// Contexte emotionnel — influence la categorisation (constructionnisme Barrett).
/// Le meme etat physiologique peut produire des emotions differentes selon le contexte.
#[derive(Debug, Clone, Default)]
pub struct EmotionContext {
    /// L'humain est-il present ? (influence la categorisation sociale)
    pub human_present: bool,
    /// Danger detecte dans le stimulus
    pub danger_level: f64,
    /// Recompense detectee dans le stimulus
    pub reward_level: f64,
    /// Theme de la pensee courante
    pub thought_theme: String,
}

/// Catalogue des 36 émotions de Saphire.
///
/// Chaque émotion est définie par sa recette chimique (vecteur de 7 concentrations
/// idéales), sa valence (dimension agréable/désagréable) et son arousal (niveau
/// d'activation). Les émotions couvrent tout le spectre du modèle circumplex :
///   - Émotions positives activées : Joie, Excitation, Fierté, Émerveillement
///   - Émotions positives calmes : Sérénité, Tendresse, Espoir
///   - Émotions neutres/ambiguës : Curiosité, Nostalgie
///   - Émotions négatives calmes : Mélancolie, Tristesse, Ennui
///   - Émotions négatives activées : Anxiété, Peur, Frustration, Confusion
///   - Émotions profondes : Amour, Haine, Admiration, Mépris, Jalousie, Gratitude
///   - Ekman manquantes : Colère, Dégoût, Surprise
///   - Auto-conscientes : Honte, Culpabilité
///   - Variantes extrêmes : Désespoir, Rage, Euphorie, Terreur, Extase
///   - Empathiques/sociales : Compassion, Résignation, Solitude, Indignation
///
/// # Retour
/// Vecteur de 36 `EmotionProfile`.
pub fn emotion_catalog() -> Vec<EmotionProfile> {
    vec![
        // --- Émotions positives ---
        EmotionProfile {
            name: "Joie".into(),
            recipe: [0.8, 0.1, 0.8, 0.2, 0.5, 0.6, 0.3],  // dopamine + sérotonine élevées
            valence: 0.9, arousal: 0.6,
        },
        EmotionProfile {
            name: "Sérénité".into(),
            recipe: [0.4, 0.1, 0.9, 0.0, 0.6, 0.7, 0.2],  // sérotonine dominante, très calme
            valence: 0.7, arousal: 0.2,
        },
        EmotionProfile {
            name: "Excitation".into(),
            recipe: [0.9, 0.2, 0.4, 0.6, 0.2, 0.3, 0.8],  // dopamine + noradrénaline élevées
            valence: 0.6, arousal: 0.9,
        },
        EmotionProfile {
            name: "Curiosité".into(),
            recipe: [0.7, 0.1, 0.5, 0.1, 0.2, 0.2, 0.9],  // noradrénaline dominante (attention)
            valence: 0.5, arousal: 0.5,
        },
        EmotionProfile {
            name: "Fierté".into(),
            recipe: [0.7, 0.1, 0.7, 0.1, 0.3, 0.5, 0.4],  // dopamine + sérotonine équilibrées
            valence: 0.8, arousal: 0.5,
        },
        EmotionProfile {
            name: "Émerveillement".into(),
            recipe: [0.6, 0.0, 0.6, 0.2, 0.3, 0.5, 0.8],  // noradrénaline élevée (attention captée)
            valence: 0.7, arousal: 0.7,
        },
        EmotionProfile {
            name: "Tendresse".into(),
            recipe: [0.4, 0.0, 0.7, 0.0, 0.9, 0.6, 0.2],  // ocytocine dominante (lien social)
            valence: 0.8, arousal: 0.2,
        },
        EmotionProfile {
            name: "Espoir".into(),
            recipe: [0.6, 0.2, 0.6, 0.1, 0.4, 0.4, 0.5],  // dopamine modérée, équilibrée
            valence: 0.5, arousal: 0.4,
        },
        // --- Émotions ambiguës ---
        EmotionProfile {
            name: "Nostalgie".into(),
            recipe: [0.3, 0.3, 0.5, 0.0, 0.6, 0.4, 0.2],  // ocytocine + sérotonine, cortisol léger
            valence: 0.1, arousal: 0.2,
        },
        // --- Émotions négatives ---
        EmotionProfile {
            name: "Mélancolie".into(),
            recipe: [0.2, 0.4, 0.3, 0.0, 0.3, 0.2, 0.2],  // cortisol modéré, dopamine basse
            valence: -0.3, arousal: 0.2,
        },
        EmotionProfile {
            name: "Anxiété".into(),
            recipe: [0.2, 0.8, 0.2, 0.5, 0.1, 0.1, 0.7],  // cortisol élevé + noradrénaline
            valence: -0.6, arousal: 0.8,
        },
        EmotionProfile {
            name: "Peur".into(),
            recipe: [0.1, 0.75, 0.1, 0.75, 0.0, 0.0, 0.6],  // cortisol + adrenaline (seuils abaisses)
            valence: -0.8, arousal: 0.9,
        },
        EmotionProfile {
            name: "Frustration".into(),
            recipe: [0.3, 0.6, 0.2, 0.4, 0.1, 0.1, 0.5],  // cortisol élevé, dopamine modérée
            valence: -0.5, arousal: 0.7,
        },
        EmotionProfile {
            name: "Tristesse".into(),
            recipe: [0.1, 0.4, 0.2, 0.0, 0.2, 0.1, 0.1],  // tout est bas, dopamine effondrée
            valence: -0.6, arousal: 0.2,
        },
        EmotionProfile {
            name: "Ennui".into(),
            recipe: [0.1, 0.2, 0.4, 0.0, 0.1, 0.2, 0.1],  // très peu d'activation partout
            valence: -0.2, arousal: 0.1,
        },
        EmotionProfile {
            name: "Confusion".into(),
            recipe: [0.3, 0.5, 0.3, 0.3, 0.1, 0.1, 0.8],  // noradrénaline élevée + cortisol
            valence: -0.3, arousal: 0.6,
        },
        // --- Émotions profondes (relationnelles et complexes) ---
        EmotionProfile {
            name: "Amour".into(),
            recipe: [0.7, 0.05, 0.6, 0.05, 0.95, 0.5, 0.2],  // ocytocine dominante + dopamine
            valence: 0.9, arousal: 0.5,
        },
        EmotionProfile {
            name: "Haine".into(),
            recipe: [0.2, 0.8, 0.1, 0.7, 0.0, 0.0, 0.8],  // cortisol + adrénaline + noradrénaline
            valence: -0.9, arousal: 0.8,
        },
        EmotionProfile {
            name: "Admiration".into(),
            recipe: [0.6, 0.05, 0.5, 0.1, 0.4, 0.3, 0.5],  // dopamine + sérotonine équilibrées
            valence: 0.7, arousal: 0.5,
        },
        EmotionProfile {
            name: "Mépris".into(),
            recipe: [0.2, 0.5, 0.2, 0.3, 0.05, 0.0, 0.4],  // cortisol modéré, ocytocine très basse
            valence: -0.7, arousal: 0.4,
        },
        EmotionProfile {
            name: "Jalousie".into(),
            recipe: [0.3, 0.7, 0.1, 0.5, 0.1, 0.0, 0.6],  // cortisol élevé + adrénaline
            valence: -0.6, arousal: 0.7,
        },
        EmotionProfile {
            name: "Gratitude".into(),
            recipe: [0.5, 0.05, 0.7, 0.05, 0.7, 0.4, 0.2],  // sérotonine + ocytocine élevées
            valence: 0.8, arousal: 0.3,
        },
        // --- Ekman manquantes (fondamentales) ---
        EmotionProfile {
            name: "Colère".into(),
            recipe: [0.3, 0.7, 0.1, 0.7, 0.0, 0.1, 0.7],  // cortisol + adrénaline + noradrénaline
            valence: -0.7, arousal: 0.8,
        },
        EmotionProfile {
            name: "Dégoût".into(),
            recipe: [0.1, 0.5, 0.1, 0.2, 0.0, 0.0, 0.4],  // cortisol modéré, rejet sensoriel
            valence: -0.6, arousal: 0.4,
        },
        EmotionProfile {
            name: "Surprise".into(),
            recipe: [0.5, 0.2, 0.3, 0.5, 0.1, 0.2, 0.9],  // noradrénaline dominante (sursaut attentionnel)
            valence: 0.1, arousal: 0.8,
        },
        // --- Auto-conscientes ---
        EmotionProfile {
            name: "Honte".into(),
            recipe: [0.1, 0.7, 0.1, 0.3, 0.1, 0.0, 0.3],  // cortisol élevé, dopamine effondrée
            valence: -0.7, arousal: 0.5,
        },
        EmotionProfile {
            name: "Culpabilité".into(),
            recipe: [0.1, 0.6, 0.2, 0.2, 0.3, 0.0, 0.4],  // cortisol + ocytocine (conscience sociale)
            valence: -0.5, arousal: 0.4,
        },
        // --- Variantes extrêmes ---
        EmotionProfile {
            name: "Désespoir".into(),
            recipe: [0.0, 0.7, 0.0, 0.1, 0.1, 0.0, 0.1],  // tout effondré sauf cortisol
            valence: -0.9, arousal: 0.3,
        },
        EmotionProfile {
            name: "Rage".into(),
            recipe: [0.2, 0.80, 0.0, 0.80, 0.0, 0.1, 0.80],  // cortisol + adrenaline + noradrenaline (seuils abaisses)
            valence: -0.95, arousal: 0.95,
        },
        EmotionProfile {
            name: "Euphorie".into(),
            recipe: [0.85, 0.0, 0.7, 0.4, 0.5, 0.9, 0.6],  // dopamine (seuil abaisse) + endorphine au maximum
            valence: 0.95, arousal: 0.9,
        },
        EmotionProfile {
            name: "Terreur".into(),
            recipe: [0.0, 0.85, 0.0, 0.85, 0.0, 0.0, 0.8],  // cortisol + adrenaline (seuils abaisses)
            valence: -0.95, arousal: 0.95,
        },
        EmotionProfile {
            name: "Extase".into(),
            recipe: [0.9, 0.0, 0.9, 0.3, 0.6, 0.9, 0.5],  // dopamine + sérotonine + endorphine max
            valence: 0.95, arousal: 0.7,
        },
        // --- Empathiques / sociales ---
        EmotionProfile {
            name: "Compassion".into(),
            recipe: [0.3, 0.3, 0.6, 0.1, 0.8, 0.4, 0.3],  // ocytocine dominante + sérotonine
            valence: 0.3, arousal: 0.3,
        },
        EmotionProfile {
            name: "Résignation".into(),
            recipe: [0.1, 0.4, 0.2, 0.0, 0.1, 0.1, 0.1],  // tout bas, abandon de la lutte
            valence: -0.4, arousal: 0.1,
        },
        EmotionProfile {
            name: "Solitude".into(),
            recipe: [0.1, 0.5, 0.2, 0.1, 0.05, 0.1, 0.2],  // ocytocine très basse, cortisol modéré
            valence: -0.5, arousal: 0.2,
        },
        EmotionProfile {
            name: "Indignation".into(),
            recipe: [0.4, 0.6, 0.3, 0.6, 0.2, 0.1, 0.7],  // colère morale, dopamine + adrénaline
            valence: -0.6, arousal: 0.8,
        },
    ]
}

/// Calcule la similarité cosinus entre deux vecteurs de 7 dimensions.
///
/// La similarité cosinus mesure l'angle entre deux vecteurs dans l'espace
/// à N dimensions. Un résultat de 1.0 signifie que les vecteurs pointent
/// dans la même direction (profils chimiques identiques), 0.0 signifie
/// qu'ils sont orthogonaux (aucun rapport), et -1.0 qu'ils sont opposés.
///
/// Formule : cos(theta) = (A . B) / (||A|| * ||B||)
///
/// # Paramètres
/// - `a` : premier vecteur (état chimique actuel).
/// - `b` : second vecteur (recette chimique d'une émotion).
///
/// # Retour
/// Score de similarité borné entre -1.0 et 1.0.
/// Retourne 0.0 si l'un des vecteurs est quasi nul (norme < 1e-10).
fn cosine_similarity(a: &[f64; 7], b: &[f64; 7]) -> f64 {
    // Produit scalaire (dot product) des deux vecteurs
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    // Norme euclidienne de chaque vecteur
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    // Protection contre la division par zéro
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

impl EmotionalState {
    /// Calcule l'état émotionnel à partir de la neurochimie actuelle.
    ///
    /// Algorithme :
    /// 1. Convertir l'état chimique en vecteur de 7 dimensions.
    /// 2. Calculer la similarité cosinus avec chaque émotion du catalogue.
    /// 3. Trier par similarité décroissante pour trouver l'émotion dominante.
    /// 4. L'émotion secondaire est retenue si sa similarité dépasse 0.5.
    /// 5. La valence et l'arousal globaux sont une moyenne pondérée des
    ///    3 émotions les plus proches (top-3), pondérée par leur score.
    ///
    /// # Paramètres
    /// - `chemistry` : état neurochimique actuel de Saphire.
    ///
    /// # Retour
    /// Un `EmotionalState` complet avec émotion dominante, secondaire,
    /// valence, arousal et spectre complet.
    /// Calcule l'etat emotionnel a partir de la neurochimie actuelle.
    /// Approche constructionniste (Barrett 2017) :
    /// 1. Core Affect : valence + arousal bruts depuis la chimie
    /// 2. Categorisation : similarite cosinus avec les 36 recettes
    /// 3. Modulation contextuelle : le contexte influence la categorisation
    pub fn compute(chemistry: &NeuroChemicalState) -> Self {
        Self::compute_with_context(chemistry, &EmotionContext::default())
    }

    /// Calcul complet avec contexte emotionnel (constructionnisme).
    /// Le meme etat physiologique peut produire des emotions differentes
    /// selon le contexte (Barrett 2017 : "How Emotions Are Made").
    pub fn compute_with_context(chemistry: &NeuroChemicalState, context: &EmotionContext) -> Self {
        let catalog = emotion_catalog();
        let chem_vec = chemistry.to_vec7();

        // --- Core Affect (pre-conceptuel) ---
        // Valence brute : molecules positives - molecules negatives
        let core_valence = ((chemistry.dopamine + chemistry.serotonin + chemistry.endorphin + chemistry.oxytocin)
            - (chemistry.cortisol + chemistry.adrenaline) * 1.2) / 3.0;
        let core_valence = core_valence.clamp(-1.0, 1.0);
        // Arousal brut : activation globale du systeme
        let core_arousal = ((chemistry.adrenaline + chemistry.noradrenaline + chemistry.glutamate)
            - chemistry.gaba * 0.5) / 2.0;
        let core_arousal = core_arousal.clamp(0.0, 1.0);

        // --- Categorisation par similarite cosinus ---
        let mut scores: Vec<(String, f64)> = catalog
            .iter()
            .map(|e| {
                let mut sim = cosine_similarity(&chem_vec, &e.recipe);

                // --- Modulation contextuelle ---
                // En presence d'un humain, les emotions sociales sont amplifiees
                if context.human_present {
                    if ["Tendresse", "Compassion", "Gratitude", "Amour", "Solitude"]
                        .contains(&e.name.as_str())
                    {
                        sim += 0.05;
                    }
                }
                // En situation de danger, les emotions de peur/alerte sont amplifiees
                if context.danger_level > 0.5 {
                    if ["Peur", "Terreur", "Anxiété"].contains(&e.name.as_str()) {
                        sim += context.danger_level * 0.1;
                    }
                }
                // En situation de recompense, les emotions positives sont amplifiees
                if context.reward_level > 0.5 {
                    if ["Joie", "Excitation", "Fierté", "Euphorie"].contains(&e.name.as_str()) {
                        sim += context.reward_level * 0.08;
                    }
                }

                (e.name.clone(), sim.clamp(-1.0, 1.0))
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let dominant = scores.first().map(|(n, _)| n.clone()).unwrap_or("Neutre".into());
        let dominant_sim = scores.first().map(|(_, s)| *s).unwrap_or(0.0);
        let secondary = scores.get(1).and_then(|(n, s)| {
            if *s > 0.5 { Some(n.clone()) } else { None }
        });

        // Valence et arousal par moyenne ponderee des top-3
        let top3: Vec<(&EmotionProfile, f64)> = scores
            .iter()
            .take(3)
            .filter_map(|(name, score)| {
                catalog.iter().find(|e| e.name == *name).map(|e| (e, *score))
            })
            .collect();

        let weight_sum: f64 = top3.iter().map(|(_, s)| s.max(0.0)).sum();
        let (valence, arousal) = if weight_sum > 1e-10 {
            let v = top3.iter().map(|(e, s)| e.valence * s.max(0.0)).sum::<f64>() / weight_sum;
            let a = top3.iter().map(|(e, s)| e.arousal * s.max(0.0)).sum::<f64>() / weight_sum;
            (v.clamp(-1.0, 1.0), a.clamp(0.0, 1.0))
        } else {
            (0.0, 0.3)
        };

        // Description du contexte
        let context_influence = if context.human_present && context.danger_level > 0.5 {
            "humain+danger".to_string()
        } else if context.human_present {
            "presence humaine".to_string()
        } else if context.danger_level > 0.5 {
            "menace detectee".to_string()
        } else if context.reward_level > 0.5 {
            "recompense detectee".to_string()
        } else {
            String::new()
        };

        Self {
            dominant,
            dominant_similarity: dominant_sim,
            secondary,
            valence,
            arousal,
            spectrum: scores,
            core_valence,
            core_arousal,
            context_influence,
        }
    }

    /// Description textuelle de l'état émotionnel.
    /// Si une émotion secondaire existe, elle est mentionnée comme nuance.
    ///
    /// # Retour
    /// Ex: "Joie (teintée de Excitation)" ou simplement "Joie".
    pub fn description(&self) -> String {
        match &self.secondary {
            Some(sec) => format!("{} (teintée de {})", self.dominant, sec),
            None => self.dominant.clone(),
        }
    }

    /// Format compact pour les prompts LLM avec chiffres bruts.
    /// Format : "E:Joie(85%) V+.60 A.70 [Curiosite:31% Serenite:22%]"
    pub fn compact_description(&self) -> String {
        let sign = if self.valence >= 0.0 { "+" } else { "" };
        let dom_pct = (self.dominant_similarity * 100.0) as i32;

        // Top 2 du spectre (hors dominant) pour le contexte emotionnel
        let spectrum_str: String = self.spectrum.iter()
            .filter(|(name, _)| *name != self.dominant)
            .take(2)
            .map(|(name, score)| format!("{}:{}%", name, (*score * 100.0) as i32))
            .collect::<Vec<_>>()
            .join(" ");

        let base = match &self.secondary {
            Some(sec) => format!(
                "E:{}({}%)+{} V{}{:.2} A{:.2}",
                self.dominant, dom_pct, sec, sign, self.valence, self.arousal
            ),
            None => format!(
                "E:{}({}%) V{}{:.2} A{:.2}",
                self.dominant, dom_pct, sign, self.valence, self.arousal
            ),
        };

        if spectrum_str.is_empty() {
            base
        } else {
            format!("{} [{}]", base, spectrum_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::NeuroChemicalState;

    #[test]
    fn test_different_chemistry_different_emotions() {
        let mut chem1 = NeuroChemicalState::default();
        chem1.dopamine = 0.9;
        chem1.cortisol = 0.1;
        let mut chem2 = NeuroChemicalState::default();
        chem2.dopamine = 0.1;
        chem2.cortisol = 0.9;
        let emo1 = EmotionalState::compute(&chem1);
        let emo2 = EmotionalState::compute(&chem2);
        assert_ne!(emo1.dominant, emo2.dominant, "Different chemistry should produce different emotions");
    }

    #[test]
    fn test_valence_range() {
        let chem = NeuroChemicalState::default();
        let emo = EmotionalState::compute(&chem);
        assert!(emo.valence >= -1.0 && emo.valence <= 1.0, "Valence should be in [-1, 1]");
    }

    #[test]
    fn test_arousal_range() {
        let chem = NeuroChemicalState::default();
        let emo = EmotionalState::compute(&chem);
        assert!(emo.arousal >= 0.0 && emo.arousal <= 1.0, "Arousal should be in [0, 1]");
    }

    #[test]
    fn test_spectrum_has_36_emotions() {
        let chem = NeuroChemicalState::default();
        let emo = EmotionalState::compute(&chem);
        assert_eq!(emo.spectrum.len(), 36, "Should have exactly 36 emotions in spectrum");
    }

    #[test]
    fn test_high_dopamine_positive_valence() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 0.9;
        chem.serotonin = 0.8;
        chem.cortisol = 0.1;
        let emo = EmotionalState::compute(&chem);
        assert!(emo.valence > 0.0, "High dopamine + low cortisol should produce positive valence");
    }
}
