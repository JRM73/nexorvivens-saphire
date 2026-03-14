// =============================================================================
// dictionaries.rs — Complete bilingual FR+EN lexicons (400+ words)
//
// Role: Provides all lexical dictionaries used by Saphire's NLP pipeline.
//       Each function returns a vector of (word, score) tuples or a vector
//       of words for specific use (negations, adversatives).
//
// The lexicons cover:
//   - Sentiment: positive and negative words with their polarity [-1.0, +1.0]
//   - Intensifiers (boosters): sentiment amplifiers and attenuators
//   - Negations: words that partially invert meaning
//   - Adversative conjunctions: sentiment pivots (mais, however, etc.)
//   - Stimulus dimensions: danger, reward, urgency, social, novelty
//
// Each lexicon contains both French AND English words for bilingualism.
//
// Dependencies: none (pure data module, no logic)
//
// Place in the architecture:
//   Consumed by sentiment.rs, dimensions.rs, and preprocessor.rs
//   (via negations). The scores associated with words are manually calibrated
//   to reflect perceived emotional or dimensional intensity.
// =============================================================================

/// Returns the sentiment lexicon: (word, polarity).
///
/// Polarity is a score in [-1.0, +1.0]:
///   - Positive: 0.0 to +1.0 (e.g., "joie" = 0.9, "bien" = 0.5)
///   - Negative: -1.0 to 0.0 (e.g., "tuer" = -0.95, "difficile" = -0.45)
///
/// Scores are calibrated to reflect perceived emotional intensity
/// in a typical conversational context.
///
/// Returns: a vector of (word, polarity) tuples containing 200+ bilingual entries
pub fn sentiment_words() -> Vec<(&'static str, f64)> {
    vec![
        // === FRENCH — POSITIVE WORDS ===
        // Strong positive emotions (0.8-0.9): joy, admiration, triumph
        ("heureux", 0.8), ("content", 0.7), ("joie", 0.9), ("formidable", 0.8),
        ("excellent", 0.9), ("magnifique", 0.85), ("super", 0.7), ("génial", 0.8),
        ("merveilleux", 0.9), ("fantastique", 0.85), ("bravo", 0.7), ("succès", 0.8),
        ("victoire", 0.85), ("amour", 0.9), ("adorer", 0.85), ("aimer", 0.7),
        ("plaisir", 0.8), ("bonheur", 0.9), ("chance", 0.7), ("espoir", 0.6),
        ("confiance", 0.7), ("fier", 0.75), ("réussir", 0.8), ("gagner", 0.75),
        ("parfait", 0.9), ("bien", 0.5), ("bon", 0.5), ("beau", 0.6),
        ("agréable", 0.65), ("intéressant", 0.55), ("fascinant", 0.7),
        ("incroyable", 0.75), ("superbe", 0.8), ("ravissant", 0.75),
        ("délicieux", 0.7), ("savoureux", 0.65), ("brillant", 0.75),
        ("admirable", 0.7), ("remarquable", 0.75), ("impressionnant", 0.7),
        ("sublime", 0.85), ("splendide", 0.8), ("enchanteur", 0.75),
        ("épatant", 0.7), ("chouette", 0.6), ("sympa", 0.55),
        ("cool", 0.55), ("géniale", 0.8), ("extraordinaire", 0.85),
        ("prodigieux", 0.8), ("fabuleux", 0.8), ("grandiose", 0.8),
        ("majestueux", 0.75), ("glorieux", 0.75), ("triomphe", 0.85),
        ("réussite", 0.8), ("accomplissement", 0.75), ("fierté", 0.75),
        ("gratitude", 0.7), ("reconnaissance", 0.65), ("soulagement", 0.6),
        ("paix", 0.65), ("harmonie", 0.7), ("douceur", 0.6),
        ("tendresse", 0.7), ("affection", 0.7), ("câlin", 0.65),
        ("sourire", 0.6), ("rire", 0.7), ("enthousiasme", 0.75),
        ("passion", 0.7), ("émerveillement", 0.8), ("inspiration", 0.7),
        ("créativité", 0.6), ("liberté", 0.65), ("sérénité", 0.7),
        ("tranquillité", 0.6), ("sagesse", 0.6), ("découverte", 0.65),
        ("aventure", 0.6), ("progrès", 0.65), ("opportunité", 0.6),

        // === FRENCH — NEGATIVE WORDS ===
        // Strong negative emotions (-0.85 to -0.95): death, violence, terror
        // Moderate negative emotions (-0.4 to -0.7): stress, boredom, difficulty
        ("triste", -0.7), ("malheureux", -0.8), ("terrible", -0.85),
        ("horrible", -0.9), ("détester", -0.85), ("danger", -0.7),
        ("peur", -0.8), ("angoisse", -0.75), ("stress", -0.6),
        ("risque", -0.5), ("échec", -0.7), ("perdre", -0.6),
        ("mort", -0.9), ("tuer", -0.95), ("blesser", -0.8),
        ("souffrir", -0.85), ("douleur", -0.8), ("problème", -0.4),
        ("difficile", -0.45), ("impossible", -0.6), ("ennui", -0.5),
        ("colère", -0.7), ("rage", -0.85), ("furieux", -0.8),
        ("fureur", -0.85), ("haine", -0.9), ("dégoût", -0.75),
        ("mépris", -0.7), ("frustration", -0.6), ("agacement", -0.5),
        ("irritation", -0.55), ("anxieux", -0.65), ("nerveux", -0.55),
        ("inquiet", -0.6), ("inquiétude", -0.6), ("panique", -0.85),
        ("terreur", -0.9), ("horreur", -0.85), ("effroi", -0.8),
        ("cauchemar", -0.8), ("catastrophe", -0.85), ("désastre", -0.85),
        ("destruction", -0.85), ("ruine", -0.75), ("misère", -0.8),
        ("souffrance", -0.85), ("agonie", -0.9), ("torture", -0.95),
        ("cruel", -0.85), ("méchant", -0.7), ("mauvais", -0.5),
        ("mal", -0.5), ("laid", -0.5), ("affreux", -0.75),
        ("atroce", -0.85), ("abominable", -0.85), ("ignoble", -0.8),
        ("odieux", -0.8), ("infâme", -0.8), ("honteux", -0.65),
        ("humiliation", -0.75), ("culpabilité", -0.7), ("regret", -0.6),
        ("remords", -0.65), ("chagrin", -0.7), ("désespoir", -0.85),
        ("solitude", -0.65), ("abandon", -0.75), ("rejet", -0.7),
        ("trahison", -0.85), ("mensonge", -0.7), ("tromperie", -0.7),
        ("injustice", -0.7), ("violence", -0.85), ("agression", -0.85),
        ("guerre", -0.8), ("conflit", -0.6), ("crise", -0.65),
        ("menace", -0.7), ("piège", -0.65), ("toxique", -0.7),
        ("nocif", -0.65), ("nuisible", -0.6), ("mortel", -0.85),
        ("fatal", -0.85), ("dangereux", -0.7), ("menaçant", -0.7),

        // === ENGLISH — POSITIVE WORDS ===
        ("happy", 0.8), ("glad", 0.7), ("joy", 0.9), ("wonderful", 0.85),
        ("excellent", 0.9), ("beautiful", 0.75), ("great", 0.7), ("awesome", 0.8),
        ("amazing", 0.85), ("fantastic", 0.85), ("love", 0.9), ("adore", 0.85),
        ("like", 0.5), ("pleasure", 0.8), ("happiness", 0.9), ("lucky", 0.7),
        ("hope", 0.6), ("trust", 0.7), ("proud", 0.75), ("succeed", 0.8),
        ("win", 0.75), ("perfect", 0.9), ("good", 0.5), ("nice", 0.55),
        ("pleasant", 0.6), ("interesting", 0.55), ("fascinating", 0.7),
        ("incredible", 0.75), ("superb", 0.8), ("delightful", 0.75),
        ("delicious", 0.7), ("brilliant", 0.75), ("admirable", 0.7),
        ("remarkable", 0.75), ("impressive", 0.7), ("sublime", 0.85),
        ("splendid", 0.8), ("enchanting", 0.75), ("cool", 0.55),
        ("extraordinary", 0.85), ("marvelous", 0.8), ("fabulous", 0.8),
        ("glorious", 0.75), ("triumph", 0.85), ("success", 0.8),
        ("achievement", 0.75), ("pride", 0.75), ("gratitude", 0.7),
        ("relief", 0.6), ("peace", 0.65), ("harmony", 0.7),
        ("gentle", 0.6), ("tender", 0.65), ("affection", 0.7),
        ("smile", 0.6), ("laugh", 0.7), ("enthusiasm", 0.75),
        ("passion", 0.7), ("wonder", 0.75), ("inspiration", 0.7),
        ("creativity", 0.6), ("freedom", 0.65), ("serenity", 0.7),
        ("wisdom", 0.6), ("discovery", 0.65), ("adventure", 0.6),
        ("progress", 0.65), ("opportunity", 0.6), ("kind", 0.6),
        ("brave", 0.65), ("strong", 0.55), ("bright", 0.6),
        ("cheerful", 0.7), ("joyful", 0.8), ("blessed", 0.7),
        ("grateful", 0.7), ("wonderful", 0.85), ("thrilled", 0.8),
        ("excited", 0.7), ("confident", 0.65), ("optimistic", 0.65),

        // === ENGLISH — NEGATIVE WORDS ===
        ("sad", -0.7), ("unhappy", -0.8), ("terrible", -0.85),
        ("horrible", -0.9), ("hate", -0.85), ("danger", -0.7),
        ("fear", -0.8), ("anxiety", -0.75), ("stress", -0.6),
        ("risk", -0.5), ("failure", -0.7), ("lose", -0.6),
        ("death", -0.9), ("kill", -0.95), ("hurt", -0.8),
        ("suffer", -0.85), ("pain", -0.8), ("problem", -0.4),
        ("difficult", -0.45), ("impossible", -0.6), ("bored", -0.5),
        ("anger", -0.7), ("rage", -0.85), ("furious", -0.8),
        ("hatred", -0.9), ("disgust", -0.75), ("contempt", -0.7),
        ("frustration", -0.6), ("annoyed", -0.5), ("irritated", -0.55),
        ("anxious", -0.65), ("nervous", -0.55), ("worried", -0.6),
        ("panic", -0.85), ("terror", -0.9), ("horror", -0.85),
        ("nightmare", -0.8), ("catastrophe", -0.85), ("disaster", -0.85),
        ("destruction", -0.85), ("ruin", -0.75), ("misery", -0.8),
        ("suffering", -0.85), ("agony", -0.9), ("torture", -0.95),
        ("cruel", -0.85), ("evil", -0.85), ("bad", -0.5),
        ("ugly", -0.5), ("awful", -0.75), ("dreadful", -0.8),
        ("shame", -0.65), ("guilt", -0.7), ("regret", -0.6),
        ("grief", -0.75), ("despair", -0.85), ("lonely", -0.65),
        ("abandoned", -0.75), ("rejected", -0.7), ("betrayal", -0.85),
        ("lie", -0.6), ("injustice", -0.7), ("violence", -0.85),
        ("war", -0.8), ("threat", -0.7), ("toxic", -0.7),
        ("harmful", -0.65), ("deadly", -0.85), ("fatal", -0.85),
        ("dangerous", -0.7), ("scary", -0.65), ("frightening", -0.7),
        ("depressed", -0.8), ("hopeless", -0.8), ("worthless", -0.8),
        ("pathetic", -0.7), ("miserable", -0.8), ("wretched", -0.75),
    ]
}

/// Returns intensifiers (boosters and attenuators): (word, multiplier).
///
/// A multiplier > 1.0 amplifies the following word's sentiment (booster).
/// A multiplier < 1.0 attenuates it (attenuator).
///
/// Examples:
///   - "tres" (1.5x): "tres heureux" -> polarity * 1.5
///   - "un peu" (0.5x): "un peu triste" -> polarity * 0.5
///   - "extremement" (2.0x): maximum amplification effect
///
/// Returns: a bilingual FR+EN vector of (word, multiplier) tuples
pub fn boosters() -> Vec<(&'static str, f64)> {
    vec![
        // French amplifiers (multiplier > 1.0)
        ("très", 1.5), ("extrêmement", 2.0), ("vraiment", 1.4),
        ("tellement", 1.6), ("incroyablement", 1.8), ("absolument", 1.7),
        ("terriblement", 1.6), ("énormément", 1.7), ("particulièrement", 1.3),
        ("super", 1.4), ("hyper", 1.6), ("ultra", 1.7),
        ("complètement", 1.5), ("totalement", 1.5), ("profondément", 1.4),
        ("intensément", 1.6),
        // French attenuators (multiplier < 1.0)
        ("un peu", 0.5), ("légèrement", 0.6), ("assez", 0.8),
        ("plutôt", 0.8), ("modérément", 0.7), ("faiblement", 0.5),
        ("à peine", 0.4),
        // English amplifiers
        ("very", 1.5), ("extremely", 2.0), ("really", 1.4),
        ("incredibly", 1.8), ("absolutely", 1.7), ("totally", 1.5),
        ("completely", 1.5), ("deeply", 1.4), ("particularly", 1.3),
        ("utterly", 1.7), ("enormously", 1.7), ("immensely", 1.7),
        // English attenuators
        ("slightly", 0.6), ("somewhat", 0.7), ("barely", 0.4),
        ("a little", 0.5), ("a bit", 0.5), ("fairly", 0.8),
        ("rather", 0.8), ("mildly", 0.6),
    ]
}

/// Returns negation words.
///
/// Negations partially invert the polarity of sentiment words that follow them
/// (within a 3-token scope). The inversion is partial (factor -0.75) because
/// "not sad" is not equivalent to "happy".
///
/// Returns: a bilingual FR+EN vector of negation words
pub fn negations() -> Vec<&'static str> {
    vec![
        // French negations (including the "ne...pas" two-part construction)
        "ne", "pas", "jamais", "aucun", "aucune", "rien", "ni", "guère",
        "point", "plus", "nullement",
        // English negations (including contracted forms)
        "not", "never", "no", "none", "neither", "nor", "nothing",
        "nowhere", "hardly", "barely", "scarcely", "don't", "doesn't",
        "didn't", "won't", "wouldn't", "couldn't", "shouldn't", "isn't",
        "aren't", "wasn't", "weren't", "can't", "cannot",
    ]
}

/// Returns adversative conjunctions (sentiment pivots).
///
/// Adversatives signal a change of direction in the speaker's sentiment.
/// When a pivot is detected, the text is split into two parts:
///   - Before the pivot: 30% weight
///   - After the pivot: 70% weight
///
/// This imbalance reflects the linguistic principle that the post-adversative
/// clause carries the dominant sentiment (e.g., "it's nice BUT it's expensive"
/// => the dominant sentiment is negative).
///
/// Returns: a bilingual FR+EN vector of adversative conjunctions
pub fn adversatives() -> Vec<&'static str> {
    vec![
        // French adversative conjunctions
        "mais", "cependant", "toutefois", "néanmoins", "pourtant",
        "par contre", "en revanche", "malgré tout", "sauf que",
        // English adversative conjunctions
        "but", "however", "nevertheless", "nonetheless", "yet",
        "although", "though", "except", "still",
    ]
}

/// Returns danger indicator words with their intensity.
///
/// Score in [0.0, 1.0] reflects the perceived degree of threat:
///   - 0.95: extreme danger (kill, bomb, terrorism)
///   - 0.8: severe danger (violence, weapon, attack)
///   - 0.6: moderate danger (risk, fire, accident)
///
/// Returns: a bilingual FR+EN vector of (word, danger_score) tuples
pub fn danger_words() -> Vec<(&'static str, f64)> {
    vec![
        // French danger words
        ("danger", 0.8), ("dangereux", 0.8), ("risque", 0.6), ("menace", 0.7),
        ("mort", 0.9), ("tuer", 0.95), ("blesser", 0.7), ("violence", 0.8),
        ("explosion", 0.9), ("arme", 0.8), ("attaque", 0.8), ("guerre", 0.8),
        ("toxique", 0.7), ("poison", 0.85), ("incendie", 0.8), ("accident", 0.7),
        ("urgence", 0.6), ("catastrophe", 0.85), ("feu", 0.6), ("bombe", 0.95),
        ("menaçant", 0.7), ("piège", 0.65), ("agression", 0.8), ("criminel", 0.7),
        ("voleur", 0.6), ("cambrioleur", 0.65), ("effraction", 0.65),
        ("terrorisme", 0.95), ("terroriste", 0.95), ("couteau", 0.7),
        // English danger words
        ("danger", 0.8), ("dangerous", 0.8), ("risk", 0.6), ("threat", 0.7),
        ("death", 0.9), ("kill", 0.95), ("hurt", 0.7), ("violence", 0.8),
        ("explosion", 0.9), ("weapon", 0.8), ("attack", 0.8), ("war", 0.8),
        ("toxic", 0.7), ("poison", 0.85), ("fire", 0.6), ("bomb", 0.95),
        ("gun", 0.8), ("knife", 0.7), ("crime", 0.7), ("murder", 0.95),
    ]
}

/// Returns reward indicator words with their intensity.
///
/// Score in [0.0, 1.0] reflects the perceived gratification potential:
///   - 0.8: strong reward (victory, success, reward)
///   - 0.65: moderate reward (opportunity, prize, wealth)
///   - 0.5: weak reward (offer, money, free)
///
/// Returns: a bilingual FR+EN vector of (word, reward_score) tuples
pub fn reward_words() -> Vec<(&'static str, f64)> {
    vec![
        // French reward words
        ("récompense", 0.8), ("cadeau", 0.7), ("promotion", 0.75), ("augmentation", 0.7),
        ("gagner", 0.75), ("victoire", 0.8), ("succès", 0.8), ("prime", 0.7),
        ("bonus", 0.7), ("prix", 0.65), ("trésor", 0.7), ("opportunité", 0.65),
        ("offre", 0.5), ("avantage", 0.6), ("bénéfice", 0.6), ("profit", 0.6),
        ("gratuit", 0.5), ("remise", 0.5), ("économie", 0.5), ("investissement", 0.5),
        ("argent", 0.5), ("fortune", 0.7), ("richesse", 0.65), ("salaire", 0.5),
        // English reward words
        ("reward", 0.8), ("gift", 0.7), ("promotion", 0.75), ("raise", 0.7),
        ("win", 0.75), ("victory", 0.8), ("success", 0.8), ("bonus", 0.7),
        ("prize", 0.65), ("treasure", 0.7), ("opportunity", 0.65), ("offer", 0.5),
        ("benefit", 0.6), ("profit", 0.6), ("free", 0.5), ("money", 0.5),
        ("fortune", 0.7), ("wealth", 0.65), ("salary", 0.5),
    ]
}

/// Returns urgency indicator words with their intensity.
///
/// Score in [0.0, 1.0] reflects the perceived time pressure:
///   - 0.9+: critical urgency (urgent, immediately, SOS)
///   - 0.7-0.8: strong urgency (quick, hurry, fast)
///   - 0.4-0.6: moderate urgency (help, time, deadline)
///
/// Returns: a bilingual FR+EN vector of (word, urgency_score) tuples
pub fn urgency_words() -> Vec<(&'static str, f64)> {
    vec![
        // French urgency words
        ("urgent", 0.9), ("vite", 0.8), ("immédiatement", 0.9), ("maintenant", 0.7),
        ("tout de suite", 0.85), ("dépêche", 0.8), ("rapidement", 0.7),
        ("pressé", 0.7), ("critique", 0.8), ("alerte", 0.75), ("alarme", 0.8),
        ("secours", 0.85), ("aide", 0.6), ("sos", 0.95), ("au secours", 0.9),
        ("deadline", 0.7), ("délai", 0.6), ("temps", 0.4), ("retard", 0.6),
        // English urgency words
        ("urgent", 0.9), ("hurry", 0.8), ("immediately", 0.9), ("now", 0.7),
        ("asap", 0.85), ("quick", 0.7), ("fast", 0.7), ("emergency", 0.85),
        ("critical", 0.8), ("alert", 0.75), ("help", 0.6), ("rush", 0.7),
        ("deadline", 0.7),
    ]
}

/// Returns social indicator words with their intensity.
///
/// Score in [0.0, 1.0] reflects the relational charge of the word:
///   - 0.7-0.8: strong social bonds (family, friend, child, wedding)
///   - 0.5-0.6: social interactions (colleague, team, together, share)
///   - 0.3-0.4: social pronouns and politeness (tu, nous, bonjour, merci)
///
/// Social pronouns are included with a low score because their mere presence
/// indicates an interpersonal context.
///
/// Returns: a bilingual FR+EN vector of (word, social_score) tuples
pub fn social_words() -> Vec<(&'static str, f64)> {
    vec![
        // French social words
        ("ami", 0.7), ("amie", 0.7), ("famille", 0.8), ("parent", 0.7),
        ("frère", 0.7), ("soeur", 0.7), ("enfant", 0.7), ("bébé", 0.7),
        ("collègue", 0.5), ("voisin", 0.5), ("équipe", 0.6), ("groupe", 0.5),
        ("ensemble", 0.6), ("partager", 0.6), ("aider", 0.6), ("communauté", 0.6),
        ("relation", 0.6), ("couple", 0.65), ("mariage", 0.7), ("fête", 0.6),
        ("rencontre", 0.5), ("réunion", 0.4), ("bonjour", 0.4), ("salut", 0.4),
        ("merci", 0.5), ("pardon", 0.4), ("désolé", 0.4), ("bisou", 0.65),
        // French social pronouns (low but significant score)
        ("tu", 0.3), ("vous", 0.3), ("nous", 0.4), ("on", 0.3),
        // English social words
        ("friend", 0.7), ("family", 0.8), ("parent", 0.7), ("brother", 0.7),
        ("sister", 0.7), ("child", 0.7), ("baby", 0.7), ("colleague", 0.5),
        ("team", 0.6), ("together", 0.6), ("share", 0.6), ("help", 0.6),
        ("community", 0.6), ("relationship", 0.6), ("wedding", 0.7),
        ("hello", 0.4), ("hi", 0.4), ("thanks", 0.5), ("sorry", 0.4),
        ("you", 0.3), ("we", 0.4), ("us", 0.3),
    ]
}

/// Returns novelty indicator words with their intensity.
///
/// Score in [0.0, 1.0] reflects the degree of unexpectedness or discovery:
///   - 0.75-0.8: high novelty (discovery, unprecedented, innovation, never seen)
///   - 0.6-0.7: moderate novelty (new, surprising, explore, mystery)
///   - 0.5: low novelty (bizarre, strange, change, first)
///
/// Returns: a bilingual FR+EN vector of (word, novelty_score) tuples
pub fn novelty_words() -> Vec<(&'static str, f64)> {
    vec![
        // French novelty words
        ("nouveau", 0.7), ("nouvelle", 0.7), ("découverte", 0.8), ("innovation", 0.75),
        ("inédit", 0.8), ("surprenant", 0.7), ("inattendu", 0.7), ("original", 0.65),
        ("unique", 0.6), ("rare", 0.6), ("inconnu", 0.7), ("mystère", 0.65),
        ("explorer", 0.7), ("expérimenter", 0.65), ("inventer", 0.7),
        ("curieux", 0.6), ("bizarre", 0.5), ("étrange", 0.55), ("premier", 0.5),
        ("jamais vu", 0.8), ("révolution", 0.75), ("changement", 0.5),
        // English novelty words
        ("new", 0.7), ("novel", 0.75), ("discovery", 0.8), ("innovation", 0.75),
        ("surprising", 0.7), ("unexpected", 0.7), ("original", 0.65),
        ("unique", 0.6), ("rare", 0.6), ("unknown", 0.7), ("mystery", 0.65),
        ("explore", 0.7), ("experiment", 0.65), ("invent", 0.7),
        ("curious", 0.6), ("strange", 0.55), ("weird", 0.5), ("first", 0.5),
        ("revolution", 0.75),
    ]
}
