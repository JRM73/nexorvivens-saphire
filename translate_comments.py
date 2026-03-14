#!/usr/bin/env python3
"""
Translate French comments in Rust source files to English.
Only modifies comments (// or /// or //! or /* */ blocks).
Does NOT modify code, variable names, string literals, or log messages.
"""

import re
import os
import sys

# Comprehensive French-to-English translation dictionary for comment patterns
TRANSLATIONS = {
    # File headers
    "Role :": "Role:",
    "Rôle :": "Role:",
    "Dependances :": "Dependencies:",
    "Dépendances :": "Dependencies:",
    "Place dans l'architecture :": "Place in architecture:",
    "Place dans l'architecture:": "Place in architecture:",

    # Common comment phrases (longer first to avoid partial matches)
    "Ce fichier est la racine de la bibliotheque": "This file is the root of the library",
    "Ce fichier definit": "This file defines",
    "Ce fichier définit": "This file defines",
    "Ce fichier implemente": "This file implements",
    "Ce fichier implémente": "This file implements",
    "Ce fichier fournit": "This file provides",
    "Ce fichier calcule": "This file computes",
    "Ce fichier modélise": "This file models",
    "Ce fichier modelise": "This file models",
    "Ce module est": "This module is",
    "Ce module permet": "This module allows",
    "Ce module gere": "This module manages",
    "Ce module gère": "This module manages",
    "Ce module contient": "This module contains",
    "Il declare et expose": "It declares and exposes",
    "Il est lu par": "It is read by",
    "Il ne modifie aucun": "It does not modify any",
    "Il observe": "It observes",

    # Technical terms and concepts
    "neurotransmetteurs": "neurotransmitters",
    "neurotransmetteur": "neurotransmitter",
    "neurochimique": "neurochemical",
    "neurochimie": "neurochemistry",
    "neurorecepteurs": "neuroreceptors",
    "homeostasie": "homeostasis",
    "homéostasie": "homeostasis",
    "similarite cosinus": "cosine similarity",
    "similarité cosinus": "cosine similarity",
    "similarite": "similarity",
    "similarité": "similarity",
    "sérialisation": "serialization",
    "serialisation": "serialization",
    "désérialisation": "deserialization",
    "deserialisation": "deserialization",
    "somme pondérée": "weighted sum",
    "somme ponderee": "weighted sum",
    "moyenne pondérée": "weighted average",
    "moyenne ponderee": "weighted average",
    "moyenne ponderée": "weighted average",
    "Moyenne Mobile Exponentielle": "Exponential Moving Average",
    "monologue interieur": "inner monologue",
    "monologue intérieur": "inner monologue",
    "espace de travail global": "global workspace",
    "erreur de prediction": "prediction error",
    "erreur de prédiction": "prediction error",
    "rendements decroissants": "diminishing returns",
    "rendements décroissants": "diminishing returns",
    "saturation des recepteurs": "receptor saturation",
    "saturation des récepteurs": "receptor saturation",
    "biais de confirmation": "confirmation bias",
    "biais de selection": "selection bias",
    "biais de sélection": "selection bias",
    "detection de biais": "bias detection",
    "détection de biais": "bias detection",
    "completude cognitive": "cognitive completeness",
    "complétude cognitive": "cognitive completeness",
    "Metrique de Turing": "Turing Metric",
    "Métrique de Turing": "Turing Metric",
    "diversite emotionnelle": "emotional diversity",
    "diversité emotionnelle": "emotional diversity",
    "diversité émotionnelle": "emotional diversity",
    "diversite lexicale": "lexical diversity",
    "matiere grise": "grey matter",
    "matière grise": "grey matter",
    "champs electromagnetiques": "electromagnetic fields",
    "champs électromagnétiques": "electromagnetic fields",
    "colonne vertebrale": "spinal cord",
    "colonne vertébrale": "spinal cord",
    "systeme nerveux autonome": "autonomic nervous system",
    "système nerveux autonome": "autonomic nervous system",
    "pression de sommeil": "sleep pressure",
    "pensee autonome": "autonomous thought",
    "pensée autonome": "autonomous thought",
    "pensees autonomes": "autonomous thoughts",
    "pensées autonomes": "autonomous thoughts",
    "pensee libre": "free thought",
    "pensée libre": "free thought",
    "auto-reflexion": "self-reflection",
    "auto-réflexion": "self-reflection",
    "auto-estimation": "self-estimation",
    "auto-critique": "self-critique",
    "recherche par similarite": "similarity search",
    "recherche par similarité": "similarity search",
    "recherche vectorielle": "vector search",
    "memoire vectorielle": "vector memory",
    "mémoire vectorielle": "vector memory",
    "memoire episodique": "episodic memory",
    "mémoire épisodique": "episodic memory",
    "memoire a long terme": "long-term memory",
    "mémoire à long terme": "long-term memory",
    "memoire de travail": "working memory",
    "mémoire de travail": "working memory",
    "reseau de neurones": "neural network",
    "réseau de neurones": "neural network",
    "micro-reseau de neurones": "micro neural network",
    "micro-réseau de neurones": "micro neural network",
    "perceptron multi-couches": "multi-layer perceptron",
    "apprentissage local": "local learning",
    "conscience artificielle": "artificial consciousness",
    "conscience de soi": "self-awareness",
    "cognition artificielle": "artificial cognition",
    "instinct de survie": "survival instinct",
    "etincelle de vie": "spark of life",
    "étincelle de vie": "spark of life",
    "corps virtuel": "virtual body",
    "droit de veto": "veto right",
    "systeme ethique": "ethics system",
    "système éthique": "ethics system",
    "ethique personnelle": "personal ethics",
    "éthique personnelle": "personal ethics",
    "principes ethiques": "ethical principles",
    "principes éthiques": "ethical principles",
    "lois morales": "moral laws",
    "detection materielle": "hardware detection",
    "détection matérielle": "hardware detection",
    "genome deterministe": "deterministic genome",
    "génome déterministe": "deterministic genome",
    "graphe dynamique": "dynamic graph",
    "connexions neuronales": "neural connections",
    "synaptogenese": "synaptogenesis",
    "synaptogénèse": "synaptogenesis",
    "pruning synaptique": "synaptic pruning",
    "systeme de soins": "care system",
    "système de soins": "care system",
    "therapie psychologique": "psychological therapy",
    "thérapie psychologique": "psychological therapy",
    "art therapie": "art therapy",
    "art thérapie": "art therapy",
    "liens affectifs": "affective bonds",
    "style d'attachement": "attachment style",
    "situation familiale": "family situation",
    "backend LLM fictif": "mock LLM backend",
    "prompt substrat": "substrate prompt",
    "prompt systeme": "system prompt",
    "prompt système": "system prompt",
    "valeurs d'usine": "factory defaults",
    "remise a zero": "reset",
    "base de donnees": "database",
    "base de données": "database",
    "base de donnees de logs": "logs database",
    "serveur web": "web server",
    "signal d'arret": "shutdown signal",
    "cycle cognitif": "cognitive cycle",
    "cycle de traitement": "processing cycle",
    "cycle de vie": "life cycle",
    "boucle de vie": "life loop",
    "tick de sommeil": "sleep tick",
    "mode demonstration": "demo mode",
    "mode démonstration": "demo mode",

    # Doc comment patterns
    "Retour :": "Returns:",
    "# Retour": "# Returns",
    "Parametres :": "Parameters:",
    "Paramètres :": "Parameters:",
    "# Parametres": "# Parameters",
    "# Paramètres": "# Parameters",
    "Retourne": "Returns",
    "Cree un": "Creates a",
    "Crée un": "Creates a",
    "Cree une": "Creates a",
    "Crée une": "Creates a",
    "Genere": "Generates",
    "Génère": "Generates",
    "Calcule": "Computes",
    "Verifie": "Checks",
    "Vérifie": "Checks",
    "Detecte": "Detects",
    "Détecte": "Detects",
    "Applique": "Applies",
    "Enregistre": "Records",
    "Serialise": "Serializes",
    "Sérialise": "Serializes",
    "Initialise": "Initializes",
    "Reconstruit": "Reconstructs",
    "Convertit": "Converts",
    "Formatte": "Formats",
    "Affiche": "Displays",
    "Charge": "Loads",
    "Envoie": "Sends",
    "Obtient": "Gets",
    "Evalue": "Evaluates",
    "Évalue": "Evaluates",
    "Traite": "Processes",
    "Construit": "Builds",
    "Tronque": "Truncates",
    "Retire": "Removes",
    "Ajoute": "Adds",
    "Supprime": "Removes",
    "Met a jour": "Updates",
    "Met à jour": "Updates",
    "Lance": "Starts",
    "Modifie": "Modifies",

    # Common words/phrases in comments
    "chaine de caracteres": "string",
    "chaîne de caractères": "string",
    "chaine de caractères": "string",
    "valeur par defaut": "default value",
    "valeur par défaut": "default value",
    "valeurs par defaut": "default values",
    "valeurs par défaut": "default values",
    "nombre maximal": "maximum number",
    "taille maximale": "maximum size",
    "nombre de": "number of",
    "liste de": "list of",
    "tableau de": "array of",
    "vecteur de": "vector of",
    "score de": "score of",
    "niveau de": "level of",
    "etat de": "state of",
    "état de": "state of",
    "compteur de": "counter of",
    "historique de": "history of",
    "historique des": "history of",
    "dernier": "last",
    "derniere": "last",
    "dernière": "last",
    "precedent": "previous",
    "précédent": "previous",
    "precedente": "previous",
    "précédente": "previous",
    "courant": "current",
    "courante": "current",
    "actuel": "current",
    "actuelle": "current",
    "booleen": "boolean",
    "booléen": "boolean",
    "optionnel": "optional",
    "optionnelle": "optional",
    "entre 0.0 et 1.0": "between 0.0 and 1.0",
    "entre 0 et 1": "between 0 and 1",
    "borne entre": "bounded between",
    "borné entre": "bounded between",
    "bornee entre": "bounded between",
    "bornée entre": "bounded between",
    "avec un minimum": "with a minimum",
    "avec un maximum": "with a maximum",
    "garanti de": "guaranteed of",
    "jamais": "never",
    "toujours": "always",
    "aucun": "none",
    "aucune": "none",
    "chaque": "each",
}

def is_comment_line(line):
    """Check if a line is a comment (not inside a string)."""
    stripped = line.lstrip()
    return (stripped.startswith('//') or
            stripped.startswith('/*') or
            stripped.startswith('*') or
            stripped.startswith('*/'))

def is_in_string(line, pos):
    """Check if position is inside a string literal."""
    in_string = False
    string_char = None
    i = 0
    while i < pos:
        c = line[i]
        if not in_string and c in ('"', "'"):
            in_string = True
            string_char = c
        elif in_string and c == string_char:
            # Check for escape
            if i > 0 and line[i-1] != '\\':
                in_string = False
        i += 1
    return in_string

def has_french(text):
    """Detect if text contains French language patterns."""
    french_indicators = [
        r'\b(le|la|les|un|une|des|du|au|aux)\b',
        r'\b(est|sont|sera|etait|était)\b',
        r'\b(ce|cette|ces|cet)\b',
        r'\b(qui|que|dont|où|ou)\b',
        r'\b(dans|avec|pour|par|sur|sous|entre|vers|chez)\b',
        r'\b(pas|ne|ni|plus|moins|très|tres|aussi)\b',
        r'\b(peut|doit|faut|veut)\b',
        r'\b(fonction|methode|méthode|parametr|paramètr|retour|valeur)\b',
        r'\b(appel[eé]|utilis[eé]|calcul[eé]|cr[eé][eé]|d[eé]finit|contient|g[eé]n[eé]r|v[eé]rifi)\b',
        r'[àâäéèêëïîôùûüÿçÀÂÄÉÈÊËÏÎÔÙÛÜŸÇ]',
    ]
    for pattern in french_indicators:
        if re.search(pattern, text, re.IGNORECASE):
            return True
    return False

def translate_comment(comment_text):
    """Apply dictionary-based translation to a comment."""
    result = comment_text

    # Apply translations (longer phrases first for better matching)
    sorted_translations = sorted(TRANSLATIONS.items(), key=lambda x: -len(x[0]))
    for french, english in sorted_translations:
        result = result.replace(french, english)

    return result

def process_file(filepath):
    """Process a single Rust file, translating French comments to English."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"  ERROR reading {filepath}: {e}", file=sys.stderr)
        return False

    modified = False
    new_lines = []
    in_block_comment = False

    for i, line in enumerate(lines):
        stripped = line.lstrip()

        # Track block comments
        if '/*' in stripped and '*/' not in stripped:
            in_block_comment = True
        if '*/' in stripped:
            in_block_comment = False

        # Check if this is a comment line
        is_comment = (stripped.startswith('//') or
                     stripped.startswith('///') or
                     stripped.startswith('//!') or
                     in_block_comment or
                     stripped.startswith('/*') or
                     stripped.startswith('*') or
                     stripped.startswith('*/'))

        if is_comment and has_french(stripped):
            # Get the leading whitespace
            leading_ws = line[:len(line) - len(line.lstrip())]

            # Translate the comment
            translated = translate_comment(line)

            if translated != line:
                new_lines.append(translated)
                modified = True
                continue

        new_lines.append(line)

    if modified:
        try:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.writelines(new_lines)
            return True
        except Exception as e:
            print(f"  ERROR writing {filepath}: {e}", file=sys.stderr)
            return False

    return False

def main():
    base = "/mnt/Data1/code/saphire-lite/src"

    # Directories to process
    dirs = ['agent', 'algorithms', 'api', 'biology', 'body', 'care', 'cognition']

    # Root-level .rs files
    root_files = [
        'main.rs', 'lib.rs', 'consciousness.rs', 'consensus.rs',
        'display.rs', 'emotions.rs', 'factory.rs', 'llm.rs',
        'metacognition.rs', 'neurochemistry.rs', 'pipeline.rs',
        'scenarios.rs', 'stimulus.rs', 'temperament.rs'
    ]

    all_files = []

    # Collect root files
    for f in root_files:
        path = os.path.join(base, f)
        if os.path.exists(path):
            all_files.append(path)

    # Collect directory files
    for d in dirs:
        dir_path = os.path.join(base, d)
        for root, _, files in os.walk(dir_path):
            for f in files:
                if f.endswith('.rs'):
                    all_files.append(os.path.join(root, f))

    print(f"Found {len(all_files)} Rust files to process")

    modified_count = 0
    for filepath in sorted(all_files):
        rel = os.path.relpath(filepath, base)
        result = process_file(filepath)
        if result:
            modified_count += 1
            print(f"  TRANSLATED: {rel}")
        else:
            print(f"  (no changes): {rel}")

    print(f"\nDone. Modified {modified_count}/{len(all_files)} files.")

if __name__ == '__main__':
    main()
