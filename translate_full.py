#!/usr/bin/env python3
"""
Comprehensive French→English translation for Rust comments.
Handles full lines, Franglais patterns, and individual French words.
"""

import os
import re

# ─── Full line translations (comment content only) ──────────────────
# Format: (French pattern, English replacement)
# These match the text AFTER the comment prefix (// or ///)
LINE_TRANSLATIONS = [
    # File headers
    (r"Traitement post-LLM", "Post-LLM processing"),
    (r"Ce fichier contient les phases de traitement apres l'appel au LLM\.", "This file contains the processing phases after the LLM call."),
    (r"Cela inclut :", "This includes:"),
    (r"Demande d'algorithme", "Algorithm request"),
    (r"Pipeline cerebral \(process_stimulus\)", "Brain pipeline (process_stimulus)"),
    (r"Memoire de travail \+ echo memoriel", "Working memory + memory echo"),
    (r"Recompense UCB1 \+ ethique \+ formulation morale", "UCB1 reward + ethics + moral formulation"),
    (r"Feedback humain RLHF", "Human RLHF feedback"),
    (r"Collecte LoRA", "LoRA collection"),
    (r"Bonus connaissance web", "Web knowledge bonus"),
    (r"Log pensee \+ profilage OCEAN", "Thought log + OCEAN profiling"),
    (r"Trace cognitive complete", "Complete cognitive trace"),
    (r"Broadcast \+ metriques", "Broadcast + metrics"),

    # Phase headers
    (r"Phase 21 : Demande d'algorithme par le LLM", "Phase 21: Algorithm request from the LLM"),
    (r"Phase 22 : Pipeline cerebral \(process_stimulus\)", "Phase 22: Brain pipeline (process_stimulus)"),
    (r"Phase 23 : Memoire de travail", "Phase 23: Working memory"),
    (r"Phase 23b : Echo memoriel post-pensee", "Phase 23b: Post-thought memory echo"),
    (r"Phase 24 : Recompense UCB1 \+ tracking ethique \+ formulation morale", "Phase 24: UCB1 reward + ethics tracking + moral formulation"),
    (r"Phase 24c : Collecte LoRA.*", "Phase 24c: LoRA collection — save high-quality thoughts"),
    (r"Phase : Verification de derive de persona \(P0\)", "Phase: Persona drift check (P0)"),
    (r"Phase : Verification des predictions \(feedback loop P1\)", "Phase: Prediction verification (feedback loop P1)"),

    # Common function doc comments (Franglais patterns)
    (r"Similarite cosinus between deux vecteurs\.", "Cosine similarity between two vectors."),
    (r"Records l'historique LLM for la autonomous thought\.", "Records the LLM history for autonomous thought."),
    (r"Compare l'embedding de la pensee generee aux N lasts pensees\.", "Compares the embedding of the generated thought to the N last thoughts."),
    (r"Si la cosine similarity depasse le threshold \(0\.85\), la pensee est", "If the cosine similarity exceeds the threshold (0.85), the thought is"),
    (r"consideree like une repetition et le cycle est abandonne\.", "considered a repetition and the cycle is aborted."),
    (r"Stocke l'embedding in un ring buffer de 20 lasts pensees\.", "Stores the embedding in a ring buffer of the 20 most recent thoughts."),
    (r"Checks que la pensee generee reste coherente with le persona de Saphire\.", "Checks that the generated thought remains coherent with Saphire's persona."),
    (r"Utilise le moniteur de derive for comparer l'embedding de la pensee", "Uses the drift monitor to compare the thought's embedding"),
    (r"au centroide d'identite pre-calcule\.", "to the pre-computed identity centroid."),
    (r"Checksr tous les 3 cycles for ne pas surcharger l'encodeur", "Check every 3 cycles to avoid overloading the encoder"),
    (r"Detects et traite les demandes d'algorithme in la response du LLM\.", "Detects and processes algorithm requests in the LLM's response."),
    (r"Removesr le prefixe UTILISER_ALGO du texte de pensee", "Remove the UTILISER_ALGO prefix from the thought text"),
    (r"Processes la pensee generee like un stimulus interne via le pipeline", "Processes the generated thought as an internal stimulus through the"),
    (r"cerebral complet \(NLP, consensus, chemistry, emotion, conscience, regulation\)\.", "full brain pipeline (NLP, consensus, chemistry, emotion, consciousness, regulation)."),
    (r"Micro-drift autonome d'oxytocine \(rendements decroissants\)", "Autonomous micro-drift of oxytocin (diminishing returns)"),
    (r"Pousse la pensee in la working memory et gere les ejections\.", "Pushes the thought into working memory and handles ejections."),
    (r"Apres la generation LLM, cherche si la pensee produite resonne with", "After LLM generation, checks if the produced thought resonates with"),
    (r"des memories LTM existants\. Si oui, booste leur acces \(renforcement", "existing LTM memories. If so, boosts their access (Hebbian"),
    (r'hebbien : "neurons that fire together wire together"\)\.', 'reinforcement: "neurons that fire together wire together").'),
    (r"Computes la recompense for le bandit UCB1, track les reflexions morales", "Computes the reward for the UCB1 bandit, tracks moral reflections"),
    (r"et tente une formulation morale si conditions reunies\.", "and attempts a moral formulation if conditions are met."),
    (r"Recompense composite : quality \+ coherence \+ signal umami \(neurochimique\)", "Composite reward: quality + coherence + umami signal (neurochemical)"),
    (r"Purger also le inner monologue .* sinon il re-alimente la boucle", "Also purge the inner monologue — otherwise it feeds the loop"),
    (r"Poser le flag anti-stagnation.*", "Set the anti-stagnation flag: on the next cycle, the prompt"),
    (r"injectera une directive forte de changement de sujet", "will inject a strong directive to change subject"),
    (r"A\* lexical : chercher des alternatives in le connectome for each mot obsessionnel", "A* lexical: search for alternatives in the connectome for each obsessive word"),
    (r"Apres une pensee de quality, pose une question contextuelle in le chat\.", "After a quality thought, asks a contextual question in chat."),
    (r"Collecte les pensees de haute quality in la table lora_training_data\.", "Collects high-quality thoughts into the lora_training_data table."),
    (r"Le system_prompt est condens.* for .*viter de polluer le dataset LoRA", "The system_prompt is condensed to avoid polluting the LoRA dataset"),
    (r"with l'identit.* compl.*te \(ce qui renforcerait les boucles d'auto-pr.*sentation\)\.", "with the full identity (which would reinforce self-introduction loops)."),
    (r"Condens.* du system_prompt : garder les consignes without l'identit.* compl.*te", "Condensed system_prompt: keep instructions without the full identity"),
    (r"for .*viter que le fine-tuning renforce.*", "to avoid the fine-tuning reinforcing \"I am Saphire...\" in each sample"),
    (r"Condense le system_prompt for le stockage LoRA\.", "Condenses the system_prompt for LoRA storage."),
    (r"et ne garde que les consignes comportementales\.", "and keeps only the behavioral instructions."),
    (r"Extraire .* partir des CONSIGNES \(la partie utile for le fine-tuning\)", "Extract from CONSIGNES (the useful part for fine-tuning)"),
    (r"Applies le bonus chemical d'learning et log les connaissances acquises\.", "Applies the chemical learning bonus and logs acquired knowledge."),
    (r"Filtre anti-hallucination : verifier que la reflexion LLM", "Anti-hallucination filter: verify that the LLM reflection"),
    (r"n'est pas une reformulation repetitive des reflexions previouss\.", "is not a repetitive reformulation of previous reflections."),
    (r"Si la cosine similarity depasse 0\.45, on rejette la reflexion", "If the cosine similarity exceeds 0.45, the reflection is rejected"),
    (r"for eviter la consolidation de faux memories en boucle\.", "to avoid loop consolidation of false memories."),
    (r"P3 : Recordsr la d.*couverte for rassasier la curiosit.*", "P3: Record the discovery to satisfy curiosity"),
    (r"Records la pensee in thought_log et episodic memory\.", "Records the thought in thought_log and episodic memory."),
    (r"Observe le comportement for le profil OCEAN et recalcule si necessaire\.", "Observes the behavior for the OCEAN profile and recalculates if necessary."),
    (r"Complete et sauvegarde la trace cognitive with NLP, LLM, memoire,", "Completes and saves the cognitive trace with NLP, LLM, memory,"),
    (r"Diffuse l'evenement aux plugins et met a jour les interfaces\.", "Broadcasts the event to plugins and updates the interfaces."),
    (r"Sauvegarde le snapshot de metriques for le dashboard\.", "Saves the metrics snapshot for the dashboard."),
    (r"Sensibilite des recepteurs", "Receptor sensitivity"),
    (r"Checks les premonitions dont le delai est ecoule et met a jour", "Checks premonitions whose deadline has passed and updates"),
    (r"On ne stocke PAS l'embedding d'une pensee rejetee", "Do NOT store the embedding of a rejected thought"),
    (r"Stocker l'embedding \(ring buffer de 20\)", "Store the embedding (ring buffer of 20)"),
    (r"Comparer with les embeddings recents", "Compare with recent embeddings"),
    (r"Trop court pour etre pertinent", "Too short to be relevant"),

    # algorithms/orchestrator.rs
    (r"Le pont between le LLM \(langage naturel\) et les algorithmes \(code Rust\)\.", "The bridge between the LLM (natural language) and algorithms (Rust code)."),
    (r"Le LLM ne peut pas lire du code ni executer des fonctions .* mais il peut", "The LLM cannot read code or execute functions — but it can"),
    (r"lire des fiches descriptives et demander l'execution d'un algorithme\.", "read descriptive sheets and request the execution of an algorithm."),
    (r"L'orchestrateur traduit in les deux sens :", "The orchestrator translates in both directions:"),
    (r"Fiche d'algorithme \(le Vidal de Saphire\)", "Algorithm sheet (Saphire's Vidal)"),
    (r"Description en langage naturel .* c'est ca que le LLM lit", "Natural language description — this is what the LLM reads"),
    (r"Tags for la recherche rapide", "Tags for quick search"),
    (r"Entree generique for les algorithmes", "Generic input for algorithms"),
    (r"Sortie generique .* inclut un resultat en langage naturel for le LLM", "Generic output — includes a natural language result for the LLM"),
    (r"Resultat en langage naturel \(for le LLM\)", "Natural language result (for the LLM)"),
    (r"Resultat structure \(for le code\)", "Structured result (for the code)"),
    (r"Checks si le resultat contient des anomalies critiques", "Checks if the result contains critical anomalies"),
    (r"Demande d'algorithme parsee from la response du LLM", "Algorithm request parsed from the LLM response"),
    (r"Interface que each algorithme implemente", "Interface that each algorithm implements"),
    (r"Catalogue des fiches d'algorithmes", "Catalog of algorithm sheets"),
    (r"Resultats des analyses automatiques", "Results of automatic analyses"),
    (r"Creates a nouvel orchestrateur with le catalogue et les implementations", "Creates a new orchestrator with the catalog and implementations"),
    (r"Nombre d'algorithmes utilises au moins une fois", "Number of algorithms used at least once"),
    (r"ETAPE 1 : Decrire les outils disponibles for le LLM", "STEP 1: Describe available tools for the LLM"),
    (r"Generates une description en langage naturel des algorithmes pertinents", "Generates a natural language description of relevant algorithms"),
    (r"for le contexte donne\. Incluse in le substrate prompt\.", "for the given context. Included in the substrate prompt."),
    (r"ETAPE 2 : Trouver les algorithmes pertinents", "STEP 2: Find relevant algorithms"),
    (r"Recherche les algorithmes pertinents for un contexte donne", "Searches for relevant algorithms for a given context"),
    (r"Score base sur les tags", "Score based on tags"),
    (r"Score base sur when_to_use", "Score based on when_to_use"),
    (r"ETAPE 3 : Parser la demande du LLM", "STEP 3: Parse the LLM's request"),
    (r"Parse la response du LLM for detecter une demande d'algorithme\.", "Parses the LLM's response to detect an algorithm request."),
    (r"Checksr que l'algorithme existe", "Check that the algorithm exists"),
    (r"Execute un algorithme with les donnees fournies", "Executes an algorithm with the provided data"),
    (r"Execute un algorithme en mode automatique et stocke le resultat", "Executes an algorithm in automatic mode and stores the result"),
    (r"Stocker le resultat according to le type", "Store the result according to the type"),
    (r"Mettre a jour le compteur in le catalogue", "Update the counter in the catalog"),
    (r"ETAPE 5 : Recordsr la satisfaction", "STEP 5: Record satisfaction"),
    (r"Records la satisfaction apres execution for l'learning", "Records satisfaction after execution for learning"),
    (r"Mettre a jour la fiche", "Update the sheet"),
    (r"Mettre a jour la Q-table \(alpha = 0\.1\)", "Update the Q-table (alpha = 0.1)"),
    (r"Enrichissement du substrate prompt", "Substrate prompt enrichment"),
    (r"Generates le contexte des analyses automatiques for le substrate prompt", "Generates the context of automatic analyses for the substrate prompt"),
    (r"Generates un JSON de l'etat for le dashboard/API", "Generates a JSON of the state for the dashboard/API"),
    (r"Generates le JSON du catalogue complet", "Generates the JSON of the complete catalog"),
    (r"Restaure la Q-table et les compteurs from un JSON persiste", "Restores the Q-table and counters from a persisted JSON"),
    (r"Restaurer les compteurs du catalogue", "Restore the catalog counters"),
    (r"Restaurer la Q-table", "Restore the Q-table"),

    # algorithms/pca.rs
    (r"Impl.*mente la PCA, une technique de r.*duction de dimensionnalit.* qui", "Implements PCA, a dimensionality reduction technique that"),
    (r"projette les donn.*es sur les axes de variance maximum \(composantes", "projects data onto the axes of maximum variance (principal"),
    (r"principales\)\. L'algorithme calcule la matrice de covariance puis les", "components). The algorithm computes the covariance matrix then the"),
    (r"vecteurs propres dominants de la matrice de covariance\.", "dominant eigenvectors of the covariance matrix."),
    (r"portent le plus d'information, et for r.*duire la dimensionnalit.* des", "carry the most information, and to reduce the dimensionality of"),
    (r"repr.*sentations internes avant visualisation ou analyse\. Fait partie du", "internal representations before visualization or analysis. Part of the"),
    (r"R.*sultat de la PCA .* contient les donn.*es projet.*es, les composantes", "PCA result — contains the projected data, the principal"),
    (r"principales et la variance expliqu.*e par chacune\.", "components and the variance explained by each."),
]

# ─── Word/phrase-level replacements (applied to comment text) ────────
# These are applied as regex replacements within comment lines
WORD_REPLACEMENTS = [
    # Franglais verb forms (from the broken script)
    (r"\bChecksr\b", "Check"),
    (r"\bRemovesr\b", "Remove"),
    (r"\bGeneratesr\b", "Generate"),
    (r"\bRecordsr\b", "Record"),
    (r"\bComputesr\b", "Compute"),
    (r"\bAppliesr\b", "Apply"),
    (r"\bDetectsr\b", "Detect"),
    (r"\bProcessesr\b", "Process"),
    (r"\bLoadse\b", "Loaded"),
    (r"\bLoadsr\b", "Load"),
    (r"\bStartsr\b", "Start"),
    (r"\bUpdatesr\b", "Update"),
    (r"\bBuildsr\b", "Build"),
    (r"\bReturnsr\b", "Return"),
    (r"\bCreatesr\b", "Create"),
    (r"\bStoresr\b", "Store"),
    (r"\bResetsr\b", "Reset"),
    (r"\bSetsr\b", "Set"),
    (r"\bAddsr\b", "Add"),
    (r"\bConvertitr\b", "Convert"),

    # Franglais prepositions/articles
    (r"\bfor le\b", "for the"),
    (r"\bfor la\b", "for the"),
    (r"\bfor les\b", "for the"),
    (r"\bfor l'\b", "for the "),
    (r"\bfor un\b", "for a"),
    (r"\bfor une\b", "for a"),
    (r"\bfor des\b", "for"),
    (r"\bin le\b", "in the"),
    (r"\bin la\b", "in the"),
    (r"\bin les\b", "in the"),
    (r"\bin l'\b", "in the "),
    (r"\bin un\b", "in a"),
    (r"\bin une\b", "in a"),
    (r"\bwith le\b", "with the"),
    (r"\bwith la\b", "with the"),
    (r"\bwith les\b", "with the"),
    (r"\bwith l'\b", "with the "),
    (r"\bwith un\b", "with a"),
    (r"\bwith une\b", "with a"),
    (r"\bfrom le\b", "from the"),
    (r"\bfrom la\b", "from the"),
    (r"\bfrom les\b", "from the"),
    (r"\bfrom l'\b", "from the "),
    (r"\bfrom un\b", "from a"),
    (r"\bfrom une\b", "from a"),
    (r"\bof le\b", "of the"),
    (r"\bof la\b", "of the"),
    (r"\bof les\b", "of the"),
    (r"\bof l'\b", "of the "),
    (r"\bto le\b", "to the"),
    (r"\bto la\b", "to the"),
    (r"\bto les\b", "to the"),
    (r"\baccording to le\b", "according to the"),
    (r"\blike un\b", "as a"),
    (r"\blike une\b", "as a"),
    (r"\beach\b", "each"),  # Already English, keep
    (r"\btowards le\b", "towards the"),
    (r"\btowards la\b", "towards the"),
    (r"\btowards un\b", "towards a"),
    (r"\bwhen le\b", "when the"),
    (r"\bwhen la\b", "when the"),
    (r"\bwhen les\b", "when the"),
    (r"\bwhen l'\b", "when the "),

    # Common French words in comments
    (r"\ble LLM\b", "the LLM"),
    (r"\ble pipeline\b", "the pipeline"),
    (r"\ble prompt\b", "the prompt"),
    (r"\ble dashboard\b", "the dashboard"),
    (r"\ble WebSocket\b", "the WebSocket"),
    (r"\ble contexte\b", "the context"),
    (r"\ble cycle\b", "the cycle"),
    (r"\ble connectome\b", "the connectome"),
    (r"\ble thread\b", "the thread"),
    (r"\ble trait\b", "the trait"),
    (r"\ble module\b", "the module"),
    (r"\ble stimulus\b", "the stimulus"),
    (r"\ble systeme\b", "the system"),
    (r"\ble substrate\b", "the substrate"),
    (r"\ble catalogue\b", "the catalog"),
    (r"\ble code\b", "the code"),
    (r"\ble type\b", "the type"),
    (r"\ble score\b", "the score"),
    (r"\ble texte\b", "the text"),
    (r"\ble niveau\b", "the level"),
    (r"\ble nombre\b", "the number"),
    (r"\ble champ\b", "the field"),
    (r"\ble facteur\b", "the factor"),
    (r"\ble compteur\b", "the counter"),
    (r"\ble seuil\b", "the threshold"),
    (r"\ble resultat\b", "the result"),
    (r"\ble rythme\b", "the rhythm"),
    (r"\ble coeur\b", "the heart"),
    (r"\ble corps\b", "the body"),
    (r"\ble BPM\b", "the BPM"),
    (r"\ble JSON\b", "the JSON"),
    (r"\ble TOML\b", "the TOML"),
    (r"\ble fichier\b", "the file"),
    (r"\ble mot\b", "the word"),
    (r"\ble repas\b", "the meal"),
    (r"\ble repos\b", "the rest"),
    (r"\ble profil\b", "the profile"),
    (r"\ble trait\b", "the trait"),
    (r"\ble dernier\b", "the last"),
    (r"\ble premier\b", "the first"),
    (r"\bla response\b", "the response"),
    (r"\bla pensee\b", "the thought"),
    (r"\bla chimie\b", "the chemistry"),
    (r"\bla neurochemistry\b", "the neurochemistry"),
    (r"\bla config\b", "the config"),
    (r"\bla configuration\b", "the configuration"),
    (r"\bla trace\b", "the trace"),
    (r"\bla table\b", "the table"),
    (r"\bla boucle\b", "the loop"),
    (r"\bla dose\b", "the dose"),
    (r"\bla fiche\b", "the sheet"),
    (r"\bla Q-table\b", "the Q-table"),
    (r"\bla DB\b", "the DB"),
    (r"\bla liste\b", "the list"),
    (r"\bla matrice\b", "the matrix"),
    (r"\bla variance\b", "the variance"),
    (r"\bla valeur\b", "the value"),
    (r"\bla qualite\b", "the quality"),
    (r"\bla phase\b", "the phase"),
    (r"\bla capacite\b", "the capacity"),
    (r"\bla chaine\b", "the chain"),
    (r"\bla coherence\b", "the coherence"),
    (r"\bla force\b", "the force"),
    (r"\bla sante\b", "the health"),
    (r"\bla pression\b", "the pressure"),
    (r"\bla temperature\b", "the temperature"),
    (r"\bla frequence\b", "the frequency"),
    (r"\bla douleur\b", "the pain"),
    (r"\bla vitalite\b", "the vitality"),
    (r"\bla tension\b", "the tension"),
    (r"\bla condition\b", "the condition"),
    (r"\bla guerison\b", "the healing"),
    (r"\bla desensibilisation\b", "the desensitization"),
    (r"\bla cosine\b", "the cosine"),
    (r"\bla similarity\b", "the similarity"),
    (r"\bla recherche\b", "the search"),
    (r"\bla fonction\b", "the function"),
    (r"\bla seance\b", "the session"),
    (r"\bla description\b", "the description"),
    (r"\bla recompense\b", "the reward"),
    (r"\bla reflexion\b", "the reflection"),
    (r"\bla production\b", "the production"),
    (r"\bla formation\b", "the formation"),
    (r"\bla consolidation\b", "the consolidation"),
    (r"\bla conscience\b", "the consciousness"),
    (r"\bla curiosite\b", "the curiosity"),
    (r"\bles algorithmes\b", "the algorithms"),
    (r"\bles tags\b", "the tags"),
    (r"\bles donnees\b", "the data"),
    (r"\bles phases\b", "the phases"),
    (r"\bles conditions\b", "the conditions"),
    (r"\bles endorphines\b", "the endorphins"),
    (r"\bles neurotransmitters\b", "the neurotransmitters"),
    (r"\bles interfaces\b", "the interfaces"),
    (r"\bles compteurs\b", "the counters"),
    (r"\bles consignes\b", "the instructions"),
    (r"\bles extremes\b", "the extremes"),
    (r"\bles maillons\b", "the links"),
    (r"\bles ejections\b", "the ejections"),
    (r"\bles parameters\b", "the parameters"),
    (r"\bles signaux\b", "the signals"),
    (r"\bles carences\b", "the deficiencies"),
    (r"\bles niveaux\b", "the levels"),
    (r"\bles nutrients\b", "the nutrients"),
    (r"\bles vitamines\b", "the vitamins"),
    (r"\bles bonus\b", "the bonuses"),
    (r"\bles volets\b", "the sections"),
    (r"\bles soins\b", "the care"),
    (r"\bles therapies\b", "the therapies"),
    (r"\bles medicaments\b", "the medications"),
    (r"\bles reflexions\b", "the reflections"),
    (r"\bles memories\b", "the memories"),
    (r"\bles embeddings\b", "the embeddings"),
    (r"\bles plugins\b", "the plugins"),
    (r"\bles ruptures\b", "the breaks"),
    (r"\bles pensees\b", "the thoughts"),
    (r"\bles logs\b", "the logs"),
    (r"\bles seuils\b", "the thresholds"),
    (r"\bles metriques\b", "the metrics"),
    (r"\bles profils\b", "the profiles"),
    (r"\bdu LLM\b", "from the LLM"),
    (r"\bdu pipeline\b", "from the pipeline"),
    (r"\bdu system_prompt\b", "of the system_prompt"),
    (r"\bdu connectome\b", "of the connectome"),
    (r"\bdu monologue\b", "of the monologue"),
    (r"\bdu raisonnement\b", "of the reasoning"),
    (r"\bdu texte\b", "of the text"),
    (r"\bdu last\b", "of the last"),
    (r"\bdu resultat\b", "of the result"),
    (r"\bdu traitement\b", "of the treatment"),
    (r"\bdu diagnostic\b", "of the diagnosis"),
    (r"\bdu repos\b", "of the rest"),
    (r"\bdu sommeil\b", "of sleep"),
    (r"\bdu coeur\b", "of the heart"),
    (r"\bdu substrat\b", "of the substrate"),
    (r"\bdes algorithmes\b", "of the algorithms"),
    (r"\bdes analyses\b", "of the analyses"),
    (r"\bdes traumas\b", "of the traumas"),
    (r"\bdes phobies\b", "of the phobias"),
    (r"\bdes addictions\b", "of the addictions"),
    (r"\bdes recepteurs\b", "of the receptors"),
    (r"\bdes champs\b", "of the fields"),
    (r"\bdes composantes\b", "of the components"),
    (r"\bdes profils\b", "of the profiles"),
    (r"\bdes besoins\b", "of the needs"),
    (r"\bdes handicaps\b", "of the disabilities"),
    (r"\bdes donnees\b", "of the data"),
    (r"\bdes vitamines\b", "of the vitamins"),
    (r"\bdes acides amines\b", "of the amino acids"),
    (r"\bdes proteines\b", "of the proteins"),
    (r"\bdes connexions\b", "of the connections"),
    (r"\bdes signaux\b", "of the signals"),

    # French verbs and phrases
    (r"\bmet a jour\b", "updates"),
    (r"\bMet a jour\b", "Updates"),
    (r"\bmise a jour\b", "update"),
    (r"\bMise a jour\b", "Update"),
    (r"\ba jour\b", "up to date"),
    (r"\best ecoule\b", "has elapsed"),
    (r"\best inferieur\b", "is less than"),
    (r"\best eleve\b", "is high"),
    (r"\best basse\b", "is low"),
    (r"\best forte\b", "is strong"),
    (r"\best faible\b", "is weak"),
    (r"\best sain\b", "is healthy"),
    (r"\best calme\b", "is calm"),
    (r"\best actif\b", "is active"),
    (r"\best active\b", "is active"),
    (r"\best completement arrete\b", "is completely stopped"),
    (r"\bplus on avance, plus ca aide\b", "the further along, the more it helps"),
    (r"\bpar defaut\b", "by default"),
    (r"\bpar la neurochemistry\b", "by neurochemistry"),
    (r"\bsi necessaire\b", "if necessary"),
    (r"\bsous les thresholds\b", "below the thresholds"),
    (r"\bsous stress\b", "under stress"),
    (r"\bau calme\b", "when calm"),
    (r"\bau repos\b", "at rest"),
    (r"\bau debut\b", "at the beginning"),
    (r"\ba each cycle\b", "at each cycle"),
    (r"\ba partir\b", "from"),
    (r"\ben mode automatique\b", "in automatic mode"),
    (r"\ben JSON\b", "in JSON"),
    (r"\ben langage naturel\b", "in natural language"),
    (r"\ben fonction de\b", "based on"),
    (r"\ben boucle\b", "in a loop"),
    (r"\bsur le temps\b", "over time"),
    (r"\bsur la neurochemistry\b", "on the neurochemistry"),
    (r"\bsur le long terme\b", "over the long term"),
    (r"\bau moins une fois\b", "at least once"),
    (r"\bau moment de\b", "at the time of"),
    (r"\bau demarrage\b", "at startup"),
    (r"\bau reveil\b", "on waking"),
    (r"\bou non\b", "or not"),
    (r"\bou desactive\b", "or deactivated"),

    # Standalone French words in technical comments
    (r"\bnouveau\b", "new"),
    (r"\bnouvelle\b", "new"),
    (r"\bnouvel\b", "new"),
    (r"\bnouveaux\b", "new"),
    (r"\bnouvelles\b", "new"),
    (r"\bgenere\b", "generated"),
    (r"\bgeneree\b", "generated"),
    (r"\bgeneriques\b", "generic"),
    (r"\bcomplet\b", "complete"),
    (r"\bcomplete\b", "complete"),
    (r"\bcompletement\b", "completely"),
    (r"\bactif\b", "active"),
    (r"\bactifs\b", "active"),
    (r"\bactive\b", "active"),
    (r"\bactivement\b", "actively"),
    (r"\bactivation\b", "activation"),
    (r"\bdefaut\b", "default"),
    (r"\bglobal\b", "overall"),
    (r"\bglobale\b", "overall"),
    (r"\bglobalement\b", "overall"),
    (r"\bprogressively\b", "progressively"),
    (r"\bprogressivement\b", "progressively"),
    (r"\bautomatiquement\b", "automatically"),
    (r"\bindividuellement\b", "individually"),
    (r"\binversement\b", "inversely"),
    (r"\bproportionnelle\b", "proportional"),
    (r"\bproportionnellement\b", "proportionally"),
    (r"\basymptote\b", "asymptote"),
    (r"\bsinusoidale\b", "sinusoidal"),
    (r"\bproprement\b", "properly"),
    (r"\blentement\b", "slowly"),
    (r"\brapidement\b", "quickly"),
    (r"\bsilencieusement\b", "silently"),
    (r"\bexplicitement\b", "explicitly"),
    (r"\bpartiellement\b", "partially"),
    (r"\btotalement\b", "completely"),
    (r"\bcomportementales\b", "behavioral"),
    (r"\btherapeutique\b", "therapeutic"),
    (r"\btherapeutiques\b", "therapeutic"),
    (r"\beprouvant\b", "challenging"),
    (r"\bliberateur\b", "liberating"),
    (r"\bvolontaires\b", "voluntary"),
    (r"\bdeterministe\b", "deterministic"),

    # Common phrases
    (r"\bReturns la list ofs\b", "Returns the list of"),
    (r"\bReturns l'state ofpuis\b", "Restores state from"),
    (r"\bRestaure l'state ofpuis\b", "Restores state from"),
    (r"\bla list ofs\b", "the list of"),
    (r"\bl'state ofpuis\b", "state from"),
    (r" of le ", " of the "),
    (r" of la ", " of the "),
    (r" of les ", " of the "),

    # Fix double spaces
    (r"  +", " "),
]

# ─── Complete line translations (match stripped comment content exactly) ────
# These translate complete comment lines
EXACT_LINE_TRANSLATIONS = {
    # sleep_tick.rs
    "Chimie douce": "Gentle chemistry",
    "Energie remonte": "Energy rises",
    "Coeur ralentit": "Heart slows down",
    "Subconscient travaille": "Subconscious is working",
    "Fatigue attentionnelle se reduit": "Attentional fatigue decreases",
    "Algorithmes de sleep leger (une seule fois, debut de phase)": "Light sleep algorithms (once only, start of phase)",

    # api/needs.rs
    "manuellement les actions de manger/boire.": "manually the eat/drink actions.",
    "Appliesr le boost sur la physiologie": "Apply the boost to the physiology",

    # api/grey_matter.rs
    "api/grey_matter.rs — Handlers du substrat cerebral physique": "api/grey_matter.rs — Physical brain substrate handlers",

    # General patterns
    "Etat complet": "Full state",
    "Etat du coeur": "Heart state",
    "Etat somatique": "Somatic state",
    "Etat physiologique": "Physiological state",
    "Etat du repos force": "Forced rest state",
    "Etat du trouble alimentaire": "Eating disorder state",
    "Etat des phobies": "Phobias state",
    "Etat de la cinetose": "Motion sickness state",
    "Etat des handicaps": "Disabilities state",
    "Etat des conditions extremes": "Extreme conditions state",
    "Etat des addictions": "Addictions state",
}


def is_comment_line(line: str) -> bool:
    """Check if a line is a Rust comment."""
    stripped = line.strip()
    return (stripped.startswith("//") or
            stripped.startswith("///") or
            stripped.startswith("//!") or
            stripped.startswith("/*") or
            stripped.startswith("* ") or
            stripped.startswith("*/"))


def has_french(text: str) -> bool:
    """Heuristic: does the text contain French words?"""
    french_indicators = [
        r'\ble\b', r'\bla\b', r'\bles\b', r'\bun\b', r'\bune\b', r'\bdes\b',
        r'\bdu\b', r'\bde\b', r'\best\b', r'\bsont\b', r'\bdans\b', r'\bpour\b',
        r'\bavec\b', r'\bsur\b', r'\bqui\b', r'\bque\b', r'\bmais\b',
        r'\bcette\b', r'\bces\b', r'\btout\b', r'\bbien\b', r'\baux\b',
        r'\bau\b', r'\bou\b', r'\bpar\b', r'\bet\b', r'\bsi\b',
        r'\bpas\b', r'\bplus\b', r'\bne\b', r'\bon\b',
        r'\bentre\b', r'\bvers\b', r'\bsous\b', r'\bapres\b', r'\bavant\b',
        r'\bseul\b', r'\bseule\b', r'\bchaque\b', r'\btous\b', r'\btoutes\b',
        # Accented words
        r'[àâäéèêëïîôùûüç]',
        # Common French suffixes
        r'\b\w+tion\b', r'\b\w+ement\b', r'\b\w+eur\b',
        # Franglais artifacts
        r'\bChecksr\b', r'\bRemovesr\b', r'\bGeneratesr\b', r'\bRecordsr\b',
        r'\bComputesr\b', r'\bAppliesr\b', r'\bDetectsr\b', r'\bProcessesr\b',
        r'\bLoadse\b', r'\bfor le\b', r'\bfor la\b', r'\bfor les\b',
        r'\bin le\b', r'\bin la\b', r'\bwith le\b', r'\bwith la\b',
        r'\bwith les\b', r'\bfrom le\b', r'\bfrom la\b',
    ]
    for pattern in french_indicators:
        if re.search(pattern, text):
            return True
    return False


def translate_comment_text(text: str) -> str:
    """Translate the content of a comment line (after the // prefix)."""
    result = text

    # First try exact line translations
    stripped = result.strip()
    if stripped in EXACT_LINE_TRANSLATIONS:
        indent = len(result) - len(result.lstrip())
        return " " * indent + EXACT_LINE_TRANSLATIONS[stripped]

    # Try full-line regex translations
    for pattern, replacement in LINE_TRANSLATIONS:
        result = re.sub(pattern, replacement, result)

    # Apply word-level replacements
    for pattern, replacement in WORD_REPLACEMENTS:
        result = re.sub(pattern, replacement, result)

    return result


def process_line(line: str) -> str:
    """Process a single line, translating comments only."""
    stripped = line.strip()

    # Skip empty lines
    if not stripped:
        return line

    # Handle comment lines
    if stripped.startswith("///") or stripped.startswith("//!"):
        prefix_match = re.match(r'^(\s*///!?\s?)', line)
        if prefix_match:
            prefix = prefix_match.group(1)
            content = line[len(prefix):]
            if has_french(content):
                translated = translate_comment_text(content)
                return prefix + translated
    elif stripped.startswith("//"):
        prefix_match = re.match(r'^(\s*//\s?)', line)
        if prefix_match:
            prefix = prefix_match.group(1)
            content = line[len(prefix):]
            if has_french(content):
                translated = translate_comment_text(content)
                return prefix + translated

    # Handle inline comments (code // comment)
    inline_match = re.match(r'^(.*?)\s*(//\s?)(.*)', line)
    if inline_match and not stripped.startswith("//"):
        code_part = inline_match.group(1)
        comment_prefix = inline_match.group(2)
        comment_text = inline_match.group(3)
        # Only if the code part doesn't contain a string with //
        if '"' not in code_part or code_part.count('"') % 2 == 0:
            if has_french(comment_text):
                translated = translate_comment_text(comment_text)
                return f"{code_part} {comment_prefix}{translated}"

    return line


def fix_file(filepath: str) -> bool:
    """Process a single file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except:
        return False

    new_lines = []
    modified = False

    for line in lines:
        new_line = process_line(line)
        if new_line != line:
            modified = True
        new_lines.append(new_line)

    if modified:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.writelines(new_lines)
        return True
    return False


def main():
    base = "/mnt/Data1/code/saphire-lite/src"
    dirs = ['agent', 'algorithms', 'api', 'biology', 'body', 'care', 'cognition']
    root_files = [
        'main.rs', 'lib.rs', 'consciousness.rs', 'consensus.rs',
        'display.rs', 'emotions.rs', 'factory.rs', 'llm.rs',
        'metacognition.rs', 'neurochemistry.rs', 'pipeline.rs',
        'scenarios.rs', 'stimulus.rs', 'temperament.rs'
    ]

    all_files = []
    for f in root_files:
        path = os.path.join(base, f)
        if os.path.exists(path):
            all_files.append(path)
    for d in dirs:
        dir_path = os.path.join(base, d)
        if os.path.exists(dir_path):
            for root, _, files in os.walk(dir_path):
                for f in files:
                    if f.endswith('.rs'):
                        all_files.append(os.path.join(root, f))

    fixed = 0
    for filepath in sorted(all_files):
        if fix_file(filepath):
            fixed += 1
            rel = os.path.relpath(filepath, base)
            print(f"  FIXED: {rel}")

    print(f"\nFixed {fixed} files.")


if __name__ == '__main__':
    main()
