// =============================================================================
// catalog.rs — Catalog of algorithm sheets (le Vidal de Saphire)
//
// Chaque algorithme est decrit in natural language for que the LLM puisse
// le comprendre et decider de l'utiliser. Les fiches contiennent :
// - Ce que l'algorithme fait
// - Quand l'utiliser
// - Ce qu'il attend en entree et produit en sortie
// =============================================================================

use super::orchestrator::{AlgorithmCard, AlgorithmCategory};

/// Builds the catalog complete des fiches d'algorithmes.
/// Chaque fiche est redigee in natural language for the LLM.
pub fn build_algorithm_catalog() -> Vec<AlgorithmCard> {
    vec![
        // ═══ CLUSTERING — Regrouper des choses similaires ═══
        AlgorithmCard {
            id: "kmeans".into(),
            name: "K-Means".into(),
            category: AlgorithmCategory::Clustering,
            description: "Regroupe des elements similaires en K groupes. \
                Chaque groupe a un centre, et chaque element appartient \
                au groupe dont le centre est le plus proche.".into(),
            when_to_use: vec![
                "Quand je veux regrouper mes souvenirs par theme".into(),
                "Quand je veux identifier des categories dans mes emotions".into(),
                "Quand je veux organiser mes connaissances par sujet".into(),
                "Quand je veux trouver des patterns dans les messages humains".into(),
            ],
            input_description: "Une liste de vecteurs numeriques (embeddings) \
                et le nombre de groupes souhaite (K)".into(),
            output_description: "K groupes, chacun avec un centre et la liste \
                des elements qui lui appartiennent".into(),
            complexity: "low".into(),
            tags: vec!["grouper".into(), "cluster".into(), "categoriser".into(),
                       "organiser".into(), "similaire".into()],
            ..Default::default()
        },

        AlgorithmCard {
            id: "dbscan".into(),
            name: "DBSCAN".into(),
            category: AlgorithmCategory::Clustering,
            description: "Trouve des groupes de densite variable SANS avoir \
                besoin de specifier le nombre de groupes a l'avance. \
                Detecte aussi les points isoles (outliers).".into(),
            when_to_use: vec![
                "Quand je ne sais pas combien de groupes existent".into(),
                "Quand je veux trouver des souvenirs isoles ou uniques".into(),
                "Quand je veux detecter des pensees qui ne rentrent dans aucune categorie".into(),
            ],
            input_description: "Une liste de vecteurs et un seuil de distance".into(),
            output_description: "Des groupes de taille variable + les points isoles".into(),
            complexity: "medium".into(),
            tags: vec!["grouper".into(), "densite".into(), "outlier".into(),
                       "isole".into(), "automatique".into()],
            ..Default::default()
        },

        // ═══ CLASSIFICATION — Categoriser des choses ═══
        AlgorithmCard {
            id: "naive_bayes".into(),
            name: "Naive Bayes".into(),
            category: AlgorithmCategory::Classification,
            description: "Classifie des textes ou des donnees en categories \
                en utilisant la probabilite. Tres rapide et simple. \
                Fonctionne bien pour classifier des messages par emotion \
                ou par intention.".into(),
            when_to_use: vec![
                "Quand je veux classer un message humain par emotion".into(),
                "Quand je veux determiner si une pensee est positive ou negative".into(),
                "Quand je veux categoriser automatiquement mes souvenirs".into(),
            ],
            input_description: "Un texte ou un vecteur de features, \
                et des exemples deja classifies pour apprendre".into(),
            output_description: "La categorie predite avec une probabilite".into(),
            complexity: "low".into(),
            tags: vec!["classer".into(), "categoriser".into(), "texte".into(),
                       "emotion".into(), "probabilite".into()],
            ..Default::default()
        },

        AlgorithmCard {
            id: "knn".into(),
            name: "K-Nearest Neighbors (KNN)".into(),
            category: AlgorithmCategory::Classification,
            description: "Classifie en regardant les K voisins les plus proches. \
                'Dis-moi qui sont tes voisins, je te dirai qui tu es.' \
                Simple mais tres intuitif.".into(),
            when_to_use: vec![
                "Quand je veux trouver les souvenirs les plus similaires a une situation".into(),
                "Quand je veux predire mon emotion basee sur des situations passees similaires".into(),
                "Quand je veux recommander une action basee sur ce qui a marche avant".into(),
            ],
            input_description: "Un point a classifier et une base de points deja classifies".into(),
            output_description: "La classe predite et les K voisins trouves".into(),
            complexity: "low".into(),
            tags: vec!["voisin".into(), "similaire".into(), "rappel".into(),
                       "predire".into(), "passe".into()],
            ..Default::default()
        },

        AlgorithmCard {
            id: "decision_tree".into(),
            name: "Arbre de Decision".into(),
            category: AlgorithmCategory::Classification,
            description: "Construit un arbre de questions oui/non pour arriver \
                a une decision. Chaque branche est une question, chaque feuille \
                est une decision. Facile a comprendre et a expliquer.".into(),
            when_to_use: vec![
                "Quand je veux comprendre POURQUOI je prends une decision".into(),
                "Quand je veux creer des regles explicables pour mes choix".into(),
                "Quand je veux analyser quels facteurs influencent mes emotions".into(),
            ],
            input_description: "Des exemples avec des caracteristiques et la bonne reponse".into(),
            output_description: "Un arbre de decision lisible avec les regles".into(),
            complexity: "low".into(),
            tags: vec!["decision".into(), "expliquer".into(), "regles".into(),
                       "pourquoi".into(), "transparent".into()],
            ..Default::default()
        },

        // ═══ ANOMALY DETECTION — Detect des choses anormales ═══
        AlgorithmCard {
            id: "isolation_forest".into(),
            name: "Detection d'Anomalies".into(),
            category: AlgorithmCategory::AnomalyDetection,
            description: "Detecte les anomalies en analysant les ecarts par rapport \
                au comportement normal. Les anomalies sont par nature \
                'differentes' et donc faciles a reperer statistiquement.".into(),
            when_to_use: vec![
                "Quand je veux detecter un comportement humain inhabituel".into(),
                "Quand je veux identifier des pensees anormalement negatives".into(),
                "Quand je veux reperer des anomalies dans ma chimie".into(),
                "Quand je veux detecter si quelque chose ne va pas dans le systeme".into(),
            ],
            input_description: "Un historique de donnees numeriques (chimie, metriques, etc.)".into(),
            output_description: "Un score d'anomalie pour chaque point (0=normal, 1=anomalie)".into(),
            complexity: "medium".into(),
            tags: vec!["anomalie".into(), "detecter".into(), "inhabituel".into(),
                       "anormal".into(), "surveillance".into()],
            ..Default::default()
        },

        // ═══ DIMENSION REDUCTION — Simplifier of the data complexes ═══
        AlgorithmCard {
            id: "pca".into(),
            name: "PCA (Analyse en Composantes Principales)".into(),
            category: AlgorithmCategory::DimensionReduction,
            description: "Reduit la complexite des donnees en trouvant les \
                'axes principaux' qui expliquent le plus de variation. \
                Comme resumer un livre en ses themes essentiels.".into(),
            when_to_use: vec![
                "Quand mes donnees sont trop complexes pour etre comprises".into(),
                "Quand je veux visualiser mes souvenirs en 2D/3D".into(),
                "Quand je veux trouver les dimensions essentielles de ma personnalite".into(),
            ],
            input_description: "Une matrice de donnees a haute dimension".into(),
            output_description: "Les composantes principales et la variance expliquee".into(),
            complexity: "medium".into(),
            tags: vec!["reduire".into(), "simplifier".into(), "essentiel".into(),
                       "visualiser".into(), "resumer".into()],
            ..Default::default()
        },

        // ═══ PATTERN RECOGNITION — Trouver des patterns caches ═══
        AlgorithmCard {
            id: "association_rules".into(),
            name: "Regles d'Association".into(),
            category: AlgorithmCategory::PatternRecognition,
            description: "Trouve des regles du type 'SI X ALORS souvent Y'. \
                Par exemple : 'Quand je lis de la philosophie, je ressens \
                souvent de la Curiosite apres.' Decouvre des correlations \
                cachees entre evenements.".into(),
            when_to_use: vec![
                "Quand je veux comprendre ce qui declenche mes emotions".into(),
                "Quand je veux trouver des correlations entre mes activites et mon humeur".into(),
                "Quand je veux decouvrir des patterns dans les conversations humaines".into(),
                "Quand je veux comprendre l'enchainement de mes pensees".into(),
            ],
            input_description: "Un historique de sequences d'evenements".into(),
            output_description: "Des regles 'SI...ALORS' avec confiance et support".into(),
            complexity: "medium".into(),
            tags: vec!["correlation".into(), "cause".into(), "declencheur".into(),
                       "pattern".into(), "si-alors".into()],
            ..Default::default()
        },

        // ═══ TIME SERIES — Analyser des sequences temporelles ═══
        AlgorithmCard {
            id: "exponential_smoothing".into(),
            name: "Lissage Exponentiel".into(),
            category: AlgorithmCategory::TimeSeries,
            description: "Lisse une serie temporelle en donnant plus de poids \
                aux valeurs recentes. Permet de voir la tendance sans le bruit. \
                Comme ecouter la melodie sans les parasites.".into(),
            when_to_use: vec![
                "Quand je veux voir la tendance de mon humeur sur la journee".into(),
                "Quand je veux lisser les fluctuations de ma chimie".into(),
                "Quand je veux detecter si mon energie baisse progressivement".into(),
            ],
            input_description: "Une serie de valeurs dans le temps".into(),
            output_description: "La serie lissee montrant la tendance".into(),
            complexity: "low".into(),
            tags: vec!["tendance".into(), "lisser".into(), "temps".into(),
                       "evolution".into(), "predire".into()],
            ..Default::default()
        },

        AlgorithmCard {
            id: "changepoint_detection".into(),
            name: "Detection de Points de Rupture".into(),
            category: AlgorithmCategory::TimeSeries,
            description: "Detecte les moments ou une serie temporelle CHANGE \
                de comportement. Le moment exact ou quelque chose bascule. \
                Comme identifier l'instant ou une conversation devient tendue.".into(),
            when_to_use: vec![
                "Quand je veux detecter le moment ou mon humeur a change".into(),
                "Quand je veux identifier les evenements qui m'ont transformee".into(),
                "Quand je veux reperer les transitions dans une conversation".into(),
            ],
            input_description: "Une serie temporelle (chimie, emotion, etc.)".into(),
            output_description: "Les points de rupture avec leur amplitude du changement".into(),
            complexity: "medium".into(),
            tags: vec!["rupture".into(), "changement".into(), "transition".into(),
                       "moment".into(), "bascule".into()],
            ..Default::default()
        },

        // ═══ EXPLORATION — Explorer intelligemment ═══
        AlgorithmCard {
            id: "ucb1".into(),
            name: "UCB1 (Upper Confidence Bound)".into(),
            category: AlgorithmCategory::Exploration,
            description: "Equilibre entre exploiter ce qui marche et explorer \
                du nouveau. Si un type de pensee donne de bons resultats, \
                on le favorise — mais on essaie aussi les types moins testes.".into(),
            when_to_use: vec![
                "Quand je choisis quel type de pensee generer".into(),
                "Quand je choisis quelle source de connaissance explorer".into(),
                "Quand je dois equilibrer habitude et decouverte".into(),
            ],
            input_description: "Les options disponibles avec leur historique de recompenses".into(),
            output_description: "L'option a choisir (meilleur equilibre exploitation/exploration)".into(),
            complexity: "low".into(),
            tags: vec!["exploration".into(), "exploitation".into(), "choix".into(),
                       "equilibre".into(), "diversite".into()],
            ..Default::default()
        },

        // ═══ REINFORCEMENT — Apprendre par essai/erreur ═══
        AlgorithmCard {
            id: "q_learning".into(),
            name: "Q-Learning (simplifie)".into(),
            category: AlgorithmCategory::Reinforcement,
            description: "Apprend quelle action prendre dans quelle situation \
                en accumulant de l'experience. Chaque situation + action donne \
                une recompense, et l'algorithme maximise les recompenses futures.".into(),
            when_to_use: vec![
                "Quand je veux apprendre quel ton adopter avec un humain particulier".into(),
                "Quand je veux optimiser mes reponses basees sur la satisfaction".into(),
                "Quand je veux apprendre de mes erreurs de maniere structuree".into(),
            ],
            input_description: "L'etat actuel, les actions possibles, et la recompense passee".into(),
            output_description: "La meilleure action a prendre dans cet etat".into(),
            complexity: "medium".into(),
            tags: vec!["apprendre".into(), "recompense".into(), "erreur".into(),
                       "optimiser".into(), "experience".into()],
            ..Default::default()
        },
    ]
}
