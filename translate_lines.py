#!/usr/bin/env python3
"""
Second-pass translator: Translates entire French comment lines to English.
Uses a line-level translation dictionary for common patterns.
"""

import re
import os
import sys

# Line-level translations: full French line → full English line
# Only the comment content is matched (after // or ///), preserving prefix and indentation
LINE_TRANSLATIONS = {
    # ── Header patterns ──
    "Point d'entree de Saphire": "Saphire entry point",
    "Les handlers HTTP/WebSocket sont dans le module saphire::api (src/api/).": "HTTP/WebSocket handlers are in the saphire::api module (src/api/).",
    "Point d'entree asynchrone principal.": "Main asynchronous entry point.",
    "Le macro `#[tokio::main]` cree le runtime tokio et execute cette fonction.": "The `#[tokio::main]` macro creates the tokio runtime and runs this function.",
    "Tick de sommeil et initiation du sommeil": "Sleep tick and sleep initiation",

    # ── Common phrases that appear in comments ──
    "Gere each cycle quand Saphire dort. Chaque phase produit des": "Manages each cycle when Saphire sleeps. Each phase produces",
    "effets specifiques sur la chimie, le corps, la memoire, les reves, etc.": "specific effects on chemistry, body, memory, dreams, etc.",
    "Execute un sleep tick : applique les effets de la phase current,": "Executes a sleep tick: applies the effects of the current phase,",
    "decremente le compteur, et transitionne vers la phase suivante si besoin.": "decrements the counter, and transitions to the next phase if needed.",
    "Collecter les facteurs de qualite a each tick": "Collect quality factors at each tick",
    "Chimie : apaisement progressif": "Chemistry: progressive calming",
    "Subconscient s'active": "Subconscious activates",
    "Energie remonte legerement": "Energy rises slightly",
    "Image hypnagogique en fin de phase": "Hypnagogic image at end of phase",
    "Compteur sommeil profond pour qualite": "Deep sleep counter for quality",
    "Subconscient au maximum": "Subconscious at maximum",
    "Debut de phase REM : generer un reve": "Start of REM phase: generate a dream",
    "Determiner le type de reve": "Determine the dream type",
    "Construire le prompt": "Build the prompt",
    "Log enrichi du reve": "Enriched dream log",
    "Collecter les facteurs de qualite du reve": "Collect dream quality factors",
    "Perturbation chimique du cauchemar": "Chemical disruption from nightmare",
    "Vectoriser le reve": "Vectorize the dream",
    "Processesr les emotions refoulees": "Process repressed emotions",
    "Computesr la qualite prevue pour adapter le reveil": "Compute expected quality to adapt waking",
    "Orages magnetiques degradent la qualite de sommeil": "Magnetic storms degrade sleep quality",
    "Chimie de reveil influencee par la qualite": "Wake-up chemistry influenced by quality",
    "Fatigue proportionnelle a la qualite du sommeil": "Fatigue proportional to sleep quality",
    "Ne devrait pas arriver pendant sleep_tick": "Should not happen during sleep_tick",
    "Decrementer le counter of phase et gerer les transitions": "Decrement the phase counter and handle transitions",
    "Computesr la prochaine phase (borrow immutable)": "Compute the next phase (immutable borrow)",
    "Si on revient en LightSleep depuis REM, c'est un nouveau cycle": "If returning to LightSleep from REM, it's a new cycle",
    "Synchroniser la phase avec le DreamOrchestrator": "Synchronize the phase with DreamOrchestrator",
    "Fin du sommeil — capturer les stats avant finalize": "End of sleep — capture stats before finalize",
    "finalize_wake_up calcule la qualite dynamique": "finalize_wake_up computes dynamic quality",
    "Recuperer la qualite calculee depuis le last record": "Retrieve computed quality from the last record",
    "Sauvegarder le record de sommeil en DB": "Save the sleep record to DB",
    "Broadcast l'etat du sommeil": "Broadcast sleep state",
    "Initie le processus d'endormissement.": "Initiates the falling-asleep process.",
    "Updates la sleep pressure et le subconscient de fond.": "Updates sleep pressure and background subconscious.",
    "Appele depuis la boucle principale quand Saphire est eveillee.": "Called from the main loop when Saphire is awake.",
    "Subconscient travaille en arriere-plan meme eveillee": "Subconscious works in the background even while awake",
    "Analyses algorithmiques periodiques du subconscient": "Periodic algorithmic analyses of the subconscious",
    "Detectsr et broadcaster les insights du subconscient": "Detect and broadcast subconscious insights",
    "Checks si Saphire devrait s'endormir (pression suffisante + pas en conversation).": "Checks if Saphire should fall asleep (sufficient pressure + not in conversation).",

    # ── factory_reset.rs ──
    "Helper synchrone : remet les 7 molecules aux baselines d'usine.": "Synchronous helper: resets the 7 molecules to factory baselines.",
    "Aussi remettre les baselines": "Also reset the baselines",
    "Helper synchrone : remet les parametres de fonctionnement aux factory defaults.": "Synchronous helper: resets operational parameters to factory defaults.",
    "Inclut le reset chimie. Returns la list ofs changements.": "Includes chemistry reset. Returns the list of changes.",
    "Reset parametres (inclut chimie)": "Reset parameters (includes chemistry)",
    "Effacer les souvenirs episodiques (LTM et founding preserves)": "Clear episodic memories (LTM and founding memories preserved)",
    "Effacer les apprentissages vectoriels (nn_learnings)": "Clear vector learnings (nn_learnings)",

    # ── controls.rs ──
    "Returns le nom du modele LLM currentment utilise (par ex.": "Returns the name of the currently used LLM model (e.g.",
    "depasse 80% de l'intervalle configure, l'intervalle est elargi a": "exceeds 80% of the configured interval, the interval is widened to",
    "Modifies la baseline (level of repos) d'un neurotransmitter.": "Modifies the baseline (resting level) of a neurotransmitter.",
    "Utilise par le genome pour appliquer les predispositions chimiques.": "Used by the genome to apply chemical predispositions.",
    "Modifies le poids de base d'un module cerebral dans le consensus.": "Modifies the base weight of a brain module in the consensus.",
    "Modifies un seuil de consensus (decision Oui ou Non).": "Modifies a consensus threshold (Yes or No decision).",
    "Modifies un parametre systeme de l'agent.": "Modifies a system parameter of the agent.",
    "Stabilisation d'urgence : remet immediatement les neurotransmitters": "Emergency stabilization: immediately resets neurotransmitters",
    "Utilise quand Saphire est dans un etat chimique extreme (stress tres": "Used when Saphire is in an extreme chemical state (very high stress",
    "Returns l'etat chimique current (9 neurotransmitters) en JSON pour l'API.": "Returns the current chemical state (9 neurotransmitters) as JSON for the API.",
    "Inclut : baselines chimiques, poids des modules, seuils de consensus,": "Includes: chemical baselines, module weights, consensus thresholds,",

    # ── persistence.rs ──
    "Appele au shutdown et periodiquement (tous les 50 cycles).": "Called at shutdown and periodically (every 50 cycles).",
    "Sauvegarde un reve individuel (appele quand un reve est genere).": "Saves an individual dream (called when a dream is generated).",
    "integre dans le lifecycle, donc cette methode n'est pas encore appelee.": "integrated into the lifecycle, so this method is not yet called.",
    "Sauvegarde un desir individuel (appele a la naissance d'un desir).": "Saves an individual desire (called when a desire is born).",
    "Sauvegarde une lecon individuelle (appele quand une lecon est apprise).": "Saves an individual lesson (called when a lesson is learned).",
    "Sauvegarde une blessure individuelle (appele quand une blessure est detectee).": "Saves an individual wound (called when a wound is detected).",

    # ── Generic patterns (partial line translations) ──
}

# Word-level fixes for Franglais artifacts from first pass
WORD_FIXES = [
    # Fix garbled translations
    (r'Computesr\b', 'Compute'),
    (r'Detectsr\b', 'Detect'),
    (r'Processesr\b', 'Process'),
    (r'Returns la list ofs changements', 'Returns the list of changes'),
    (r'counter of phase', 'phase counter'),

    # Common remaining French words in comments
    (r'\bchaque\b', 'each'),
    (r'\bmeme\b', 'even'),
    (r'\baussi\b', 'also'),
    (r'\bquand\b', 'when'),
    (r'\bcomme\b', 'like'),
    (r'\bentre\b', 'between'),
    (r'\bdepuis\b', 'from'),
    (r'\bvers\b', 'towards'),
    (r'\bsans\b', 'without'),
    (r'\bavec\b', 'with'),
    (r'\bpour\b', 'for'),
    (r'\bdans\b', 'in'),
    (r'\bselon\b', 'according to'),
    (r'\bmaximale\b', 'maximum'),
    (r'\bminimale\b', 'minimum'),
    (r'\bactif\b', 'active'),
    (r'\bactifs\b', 'active'),
    (r'\bactive\b(?!d|s|ly)', 'active'),
    (r'\bspecifique\b', 'specific'),
    (r'\bspecifiques\b', 'specific'),
    (r'\bdynamique\b', 'dynamic'),
    (r'\bdynamiques\b', 'dynamic'),
    (r'\bprogressif\b', 'progressive'),
    (r'\bprogressivement\b', 'progressively'),
    (r'\blegerement\b', 'slightly'),
    (r'\bimmediatement\b', 'immediately'),
    (r'\bperiodiquement\b', 'periodically'),
    (r'\bperiodiques\b', 'periodic'),
    (r'\bsynchrone\b', 'synchronous'),
    (r'\basynchrone\b', 'asynchronous'),
    (r'\boptionnelle?\b', 'optional'),
    (r'\bprincipale?\b', 'main'),
    (r'\bcourant\b', 'current'),
    (r'\bcourante\b', 'current'),
    (r'\beveillee\b', 'awake'),
    (r'\beveille\b', 'awake'),
    (r'\bsommeil\b', 'sleep'),
    (r'\breveil\b', 'waking'),
    (r'\breve\b', 'dream'),
    (r'\breves\b', 'dreams'),
    (r'\bcauchemar\b', 'nightmare'),
    (r'\bmolecules?\b', 'molecule(s)'),
    (r'\bsubconscient\b', 'subconscious'),
    (r'\bqualite\b', 'quality'),
    (r'\bchimie\b', 'chemistry'),
    (r'\bchimique\b', 'chemical'),
    (r'\bchimiques\b', 'chemical'),
    (r'\benergie\b', 'energy'),
    (r'\bcerveau\b', 'brain'),
    (r'\bcerebral\b', 'cerebral'),
    (r'\bparametre\b', 'parameter'),
    (r'\bparametres\b', 'parameters'),
    (r'\bfonctionnement\b', 'operation'),
    (r'\bconfigure\b', 'configured'),
    (r'\bconfigures\b', 'configured'),
    (r'\bseuil\b', 'threshold'),
    (r'\bseuils\b', 'thresholds'),
    (r'\bpoids\b', 'weight'),
    (r'\bmodele\b', 'model'),
    (r'\bmodeles\b', 'models'),
    (r'\bgenome\b', 'genome'),
    (r'\breponse\b', 'response'),
    (r'\breponses\b', 'responses'),
    (r'\brequete\b', 'request'),
    (r'\brequetes\b', 'requests'),
    (r'\bsouvenir\b', 'memory'),
    (r'\bsouvenirs\b', 'memories'),
    (r'\bblessure\b', 'wound'),
    (r'\bblessures\b', 'wounds'),
    (r'\bdesir\b', 'desire'),
    (r'\bdesirs\b', 'desires'),
    (r'\blecon\b', 'lesson'),
    (r'\blecons\b', 'lessons'),
    (r'\bapprentissage\b', 'learning'),
    (r'\bapprentissages\b', 'learnings'),
    (r'\bvectoriels?\b', 'vector'),
    (r'\bepistdiques?\b', 'episodic'),
    (r'\bepisodiques?\b', 'episodic'),
]

def is_comment_line(stripped):
    return (stripped.startswith('//') or
            stripped.startswith('/*') or
            stripped.startswith('*'))

def has_french(text):
    """Quick check for remaining French words in a comment line."""
    # Check for accented chars
    if re.search(r'[àâäéèêëïîôùûüÿçÀÂÄÉÈÊËÏÎÔÙÛÜŸÇ]', text):
        return True
    # Check for common French patterns (case insensitive)
    french_patterns = [
        r'\b(le|la|les|des|du|aux|cette?|ses?|sont|dans|avec|pour|par|sur|qui|que|dont|vers)\b',
        r'\b(chaque|meme|aussi|quand|comme|entre|depuis|sans|selon)\b',
        r'\b(appele|utilise|calcule|cree|defini|contient|genere|verifie|detecte|supprime|ajoute|modifie|initialise|envoie|traite)\b',
        r'\b(chimie|chimique|cerveau|cerebral|sommeil|reveil|reve|cauchemar|molecule|subconscient|energie)\b',
        r'\b(parametre|parametres|seuil|seuils|poids|modele|genome|reponse|requete)\b',
        r'\b(souvenir|souvenirs|blessure|desir|lecon|apprentissage)\b',
        r'\b(specifique|dynamique|progressif|legerement|immediatement|periodiquement|synchrone|asynchrone|optionnelle?)\b',
        r'\b(principale?|courant|courante|eveillee?|qualite|fonctionnement|configure)\b',
    ]
    for p in french_patterns:
        if re.search(p, text, re.IGNORECASE):
            return True
    return False

def apply_line_translations(line, stripped):
    """Try to match and replace full comment content."""
    # Extract comment content (after // or ///)
    for prefix in ['/// ', '// ', '//! ', '/* ', '* ']:
        if stripped.startswith(prefix):
            content = stripped[len(prefix):]
            for french, english in LINE_TRANSLATIONS.items():
                if french in content:
                    new_content = content.replace(french, english)
                    indent = line[:len(line) - len(line.lstrip())]
                    return indent + prefix + new_content + '\n'
    return None

def apply_word_fixes(line):
    """Apply word-level regex fixes."""
    result = line
    for pattern, replacement in WORD_FIXES:
        result = re.sub(pattern, replacement, result)
    return result

def process_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"  ERROR reading {filepath}: {e}", file=sys.stderr)
        return False

    modified = False
    new_lines = []

    for line in lines:
        stripped = line.lstrip()

        if is_comment_line(stripped):
            # Try line-level translation first
            translated = apply_line_translations(line, stripped)
            if translated and translated != line:
                new_lines.append(translated)
                modified = True
                continue

            # Then try word-level fixes
            if has_french(stripped):
                fixed = apply_word_fixes(line)
                if fixed != line:
                    new_lines.append(fixed)
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
        for root, _, files in os.walk(dir_path):
            for f in files:
                if f.endswith('.rs'):
                    all_files.append(os.path.join(root, f))

    print(f"Second pass: {len(all_files)} files")
    modified_count = 0
    for filepath in sorted(all_files):
        rel = os.path.relpath(filepath, base)
        result = process_file(filepath)
        if result:
            modified_count += 1
            print(f"  FIXED: {rel}")

    print(f"\nDone. Modified {modified_count}/{len(all_files)} files in second pass.")

if __name__ == '__main__':
    main()
