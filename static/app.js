/*
 * ============================================================
 * SAPHIRE — app.js
 * ============================================================
 * Fichier : app.js
 * Role    : Script principal du tableau de bord (frontend).
 *           Gere la connexion WebSocket au backend Rust,
 *           recoit les mises a jour d'etat en temps reel,
 *           et met a jour tous les panneaux de l'interface.
 *
 * Dependances :
 *   - Chart.js v4     : graphiques (ligne neurochimique + radar OCEAN)
 *   - index.html      : structure DOM des panneaux
 *   - style.css       : styles visuels cyberpunk
 *   - Backend Go (ws) : fournit les messages JSON via WebSocket
 *
 * Architecture :
 *   Le fichier est organise en sections fonctionnelles :
 *     1. Variables globales et donnees du graphique
 *     2. Initialisation (DOMContentLoaded)
 *     3. Connexion WebSocket avec reconnexion automatique
 *     4. Routage des messages (handleMessage)
 *     5. Mise a jour des panneaux (chimie, emotion, cerveau, etc.)
 *     6. Flux de conscience (stream de pensees)
 *     7. Chat utilisateur
 *     8. Gestion des sliders (baselines, poids, seuils, parametres)
 *     9. Connaissances (WebKnowledge)
 *    10. Monde (meteo, age, anniversaire)
 *    11. Memoire (working, episodique, long terme)
 *    12. Profil OCEAN (radar + sous-facettes)
 *    13. Evenements speciaux (anniversaire)
 * ============================================================
 */

// ─── Variables globales ──────────────────────────────────────
let ws = null;                  // Instance WebSocket active
let chart = null;               // Instance Chart.js pour le graphique neurochimique
let oceanChart = null;          // Instance Chart.js pour le radar OCEAN
let thoughtCount = 0;           // Compteur de pensees affichees dans le flux
let reconnectDelay = 1000;      // Delai de reconnexion WebSocket (ms), augmente progressivement
let saphireUsername = localStorage.getItem('saphire_username') || ''; // Nom de l'interlocuteur

// Configuration du graphique neurochimique (9 neurotransmetteurs)
// Chaque dataset correspond a une molecule avec sa couleur propre
const chartData = {
    labels: [],
    datasets: [
        { label: 'DOPA', data: [], borderColor: '#ffd93d', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'CORT', data: [], borderColor: '#ff2a6d', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'SERO', data: [], borderColor: '#b537f2', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'ADRE', data: [], borderColor: '#ff6b35', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'OCYT', data: [], borderColor: '#fd79a8', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'ENDO', data: [], borderColor: '#05ffa1', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'NORA', data: [], borderColor: '#00f0ff', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'GABA', data: [], borderColor: '#52b788', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
        { label: 'GLUT', data: [], borderColor: '#f4a261', borderWidth: 1.5, pointRadius: 0, tension: 0.4 },
    ]
};

// Indique si les baselines ont deja ete synchronisees depuis le serveur
// On ne le fait qu'une seule fois, au premier message recu
let baselinesInitialized = false;

// ─── Initialisation au chargement du DOM ─────────────────────
// Ordre important : d'abord les graphiques, puis la connexion,
// puis les interactions utilisateur (chat, sliders, boutons)
document.addEventListener('DOMContentLoaded', () => {
    initChart();          // Graphique neurochimique (ligne)
    initOceanChart();     // Graphique OCEAN (radar)
    initBrain3D();        // Visualisation 3D du cerveau (Three.js)
    connectWebSocket();   // Connexion temps reel au backend
    setupUsernameModal(); // Modal d'identification de l'interlocuteur
    setupChat();          // Envoi de messages utilisateur
    // Restaurer l'historique de chat depuis sessionStorage
    const saved = JSON.parse(sessionStorage.getItem('saphire_chat_history') || '[]');
    saved.forEach(m => addChatMessage(m.text, m.type, true));
    setupSliders();       // Curseurs de reglage (baselines, poids, seuils, params)
    setupStabilize();     // Bouton de stabilisation d'urgence
    setupNeedsButtons();  // Boutons nourrir/hydrater (besoins primaires)
    setupParamsToggle();  // Section parametres repliable
    setupKnowledge();     // Suggestion de sujets de connaissance
    setupOceanFacets();   // Bouton pour afficher/masquer les sous-facettes OCEAN
    setupFactory();       // Boutons de reset aux valeurs d'usine
    loadCognitiveProfileIndicator(); // Indicateur de profil cognitif
    loadPersonalityPresetIndicator(); // Indicateur de preset de personnalite
});

// Initialise le graphique Chart.js de type "line" pour les neurotransmetteurs
// Axe Y : 0 a 1 (niveaux normalises), Axe X : horodatage HH:MM:SS
function initChart() {
    const ctx = document.getElementById('chemistry-chart');
    if (!ctx) return;
    chart = new Chart(ctx, {
        type: 'line',
        data: chartData,
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    min: 0, max: 1,
                    ticks: { color: '#556', font: { family: 'Share Tech Mono', size: 10 } },
                    grid: { color: 'rgba(0,240,255,0.06)' },
                    border: { color: 'rgba(0,240,255,0.15)' }
                },
                x: {
                    ticks: { color: '#556', maxTicksLimit: 8, font: { family: 'Share Tech Mono', size: 9 } },
                    grid: { color: 'rgba(0,240,255,0.04)' },
                    border: { color: 'rgba(0,240,255,0.15)' }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: '#8af',
                        boxWidth: 10,
                        padding: 12,
                        font: { family: 'Share Tech Mono', size: 10 }
                    }
                }
            },
            animation: { duration: 400 }
        }
    });
}

// ─── Connexion WebSocket ──────────────────────────────────────
// Etablit la connexion WebSocket vers le backend Rust (endpoint /ws).
// Gere la reconnexion automatique avec backoff exponentiel
// (delai x1.5 a chaque echec, plafonne a 10 secondes).
function connectWebSocket() {
    // Determine le protocole WebSocket (ws ou wss) selon le protocole HTTP
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = protocol + '//' + window.location.host + '/ws';

    ws = new WebSocket(wsUrl);

    // Connexion reussie : tous les indicateurs passent au vert
    ws.onopen = () => {
        setIndicator('ind-ws', true);
        setIndicator('ind-db', true);
        setIndicator('ind-llm', true);
        reconnectDelay = 1000;  // Reinitialise le delai de reconnexion
    };

    // Connexion fermee : tentative de reconnexion avec delai croissant
    ws.onclose = () => {
        setIndicator('ind-ws', false);
        setTimeout(connectWebSocket, reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 1.5, 10000);
    };

    // Erreur de connexion : indicateur passe au rouge
    ws.onerror = () => {
        setIndicator('ind-ws', false);
    };

    // Reception d'un message JSON depuis le backend
    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            handleMessage(data);
        } catch (e) {
            console.error('Parse error:', e);
        }
    };
}

// Change l'etat visuel d'un indicateur (point vert/rouge dans l'en-tete)
function setIndicator(id, active) {
    const el = document.getElementById(id);
    if (!el) return;
    if (active) {
        el.classList.add('active');
        el.classList.remove('inactive');
    } else {
        el.classList.remove('active');
        el.classList.add('inactive');
    }
}

// ─── Routage des messages ─────────────────────────────────────
// Point central de traitement de tous les messages WebSocket.
// Le backend envoie differents types de messages JSON,
// chacun est dispatche vers la fonction de mise a jour appropriee.
function handleMessage(data) {
    if (data.type === 'state_update') {
        // Message principal : etat complet d'un cycle de pensee
        updateChemistry(data.chemistry);          // Niveaux des 7 molecules
        updateEmotion(data.emotion, data.mood);   // Emotion dominante + mood (valence/arousal)
        updateModules(data.consensus);             // Decision des 3 modules cerebraux
        updateConsciousness(data.consciousness);   // Niveau de conscience + narratif
        updateRegulation(data.regulation);         // Violations des lois morales
        updateIdentity(data.identity, data.cycle); // Description identitaire + numero de cycle
        addToStream(data);                         // Ajoute la pensee au flux de conscience
        updateChart(data.chemistry);               // Ajoute un point au graphique neurochimique
        // Synchronise les sliders de baseline une seule fois au premier message
        // pour que l'interface reflète les valeurs reelles du serveur
        if (data.map_sync) {
            window._saphireMapSync = data.map_sync;
            updateMapSync(data.map_sync);
        }
        if (data.brain_regions) {
            updateBrain3D(data.brain_regions);
        }
        if (data.neural_network) {
            updateNeuralNetwork(data.neural_network);
        }
        if (data.sleep) {
            updateSleepIndicator(data.sleep);
        }
        if (!baselinesInitialized && data.baselines) {
            syncBaselines(data.baselines);
            baselinesInitialized = true;
        }
    } else if (data.type === 'chat_response') {
        // Reponse de Saphire a un message de l'utilisateur
        addChatMessage(data.content, 'saphire');
    } else if (data.type === 'knowledge_acquired') {
        // Saphire a acquis une nouvelle connaissance (Wikipedia, ArXiv, etc.)
        addKnowledgeToStream(data);
        addKnowledgeToList(data);
        updateKnowledgeCount(data.total_explored || 0);
    } else if (data.type === 'memory_update') {
        // Mise a jour du systeme de memoire (working, episodique, long terme)
        updateMemory(data);
    } else if (data.type === 'ocean_update') {
        // Mise a jour du profil de personnalite OCEAN
        updateOcean(data);
    } else if (data.type === 'world_update') {
        // Informations sur le monde (meteo, date, age, anniversaire)
        updateWorld(data);
    } else if (data.type === 'body_update') {
        // Mise a jour du corps virtuel (coeur, signaux somatiques)
        updateBody(data);
    } else if (data.type === 'needs_update') {
        // Mise a jour des besoins primaires (faim, soif)
        updateNeeds(data);
    } else if (data.type === 'need_satisfied') {
        // Un besoin a ete auto-satisfait
        console.log('[Needs] Besoin satisfait:', data.action);
    } else if (data.type === 'factory_reset_done') {
        // Resultat d'un factory reset (chimie, parametres, ou complet)
        handleFactoryResetDone(data);
    } else if (data.type === 'special_event') {
        // Evenements speciaux (anniversaire, etc.)
        handleSpecialEvent(data);
    } else if (data.type === 'ethics_update') {
        // Mise a jour du systeme ethique (principes personnels)
        updateEthics(data);
    } else if (data.type === 'vital_update') {
        // Mise a jour de l'etincelle vitale, intuition, premonition
        updateVital(data);
    } else if (data.type === 'senses_update') {
        // Mise a jour du Sensorium (5 sens + emergents)
        updateSenses(data);
    } else if (data.type === 'deliberation_started') {
        // Une deliberation volontaire commence
        console.log('[Will] Deliberation declenchee:', data.trigger);
    } else if (data.type === 'deliberation_resolved') {
        // La deliberation est resolue
        console.log('[Will] Choix:', data.chosen, '(confiance:', data.confidence, ')');
    } else if (data.type === 'will_update') {
        // Mise a jour du module de volonte
        console.log('[Will] Willpower:', data.willpower, 'Fatigue:', data.decision_fatigue);
    } else if (data.type === 'will_retrospect') {
        // Fierte ou regret d'une decision passee
        console.log('[Will]', data.outcome, ':', data.decision);
    } else if (data.type === 'inner_monologue') {
        // Monologue interieur mis a jour
        console.log('[Monologue]', data.text);
    } else if (data.type === 'sleep_started') {
        // Saphire s'endort — afficher l'overlay
        showSleepOverlay();
        disableChatInput();
        console.log('[Sleep] Endormissement, pression:', data.sleep_pressure);
    } else if (data.type === 'sleep_update') {
        // Mise a jour pendant le sommeil
        if (data.is_sleeping) {
            updateSleepOverlay(data);
        }
    } else if (data.type === 'wake_up') {
        // Saphire se reveille — cacher l'overlay
        hideSleepOverlay();
        enableChatInput();
        showWakeUpMessage(data);
        console.log('[Sleep] Reveil, qualite:', data.quality);
    } else if (data.type === 'sleep_refusal') {
        // Message pendant le sommeil (refus de repondre)
        addChatMessage(data.message, 'saphire');
    } else if (data.type === 'subconscious_insight') {
        // Insight du subconscient
        console.log('[Subconscient] Insight:', data.content);
        addToStream({
            type: 'state_update',
            thought: '🫧 ' + data.content,
            thought_type: 'SubconsciousInsight',
        });
    } else if (data.type === 'neural_connection') {
        // Connexion neuronale creee
        console.log('[Neural]', data.memory_a, '<->', data.memory_b);
    } else if (data.type === 'subconscious_priming') {
        // Priming subconscient actif
        console.log('[Subconscient] Priming:', data.prime, 'force:', data.strength);
    } else if (data.type === 'temperament_update') {
        // Mise a jour du temperament emergent
        updateTemperament(data);
    }
}

// ─── Sommeil : indicateur header ──────────────────────────────

function updateSleepIndicator(sleep) {
    const ring = document.getElementById('sleep-pressure-ring');
    const emoji = document.getElementById('sleep-emoji');
    const state = document.getElementById('sleep-state');
    const pressure = document.getElementById('sleep-pressure-val');
    if (!ring) return;

    const pct = Math.round(sleep.sleep_pressure * 100);
    pressure.textContent = pct + '%';

    if (sleep.is_sleeping) {
        state.textContent = sleep.phase || I18n.t('sleep.state_sleeping', 'DORT');
        emoji.textContent = sleep.phase_emoji || '\u{1F319}';
        ring.style.borderColor = '#8888ff';
        ring.style.boxShadow = '0 0 12px rgba(136,136,255,0.6)';
        state.style.color = '#aaaaee';
    } else {
        state.textContent = I18n.t('sleep.state_awake', 'ÉVEILLÉE');
        if (pct < 30) {
            emoji.textContent = '\u2600\uFE0F';
            ring.style.borderColor = '#00ff88';
            ring.style.boxShadow = '0 0 6px rgba(0,255,136,0.3)';
            state.style.color = '#00ff88';
        } else if (pct < 60) {
            emoji.textContent = '\u{1F324}\uFE0F';
            ring.style.borderColor = '#ffaa00';
            ring.style.boxShadow = '0 0 8px rgba(255,170,0,0.4)';
            state.style.color = '#ffaa00';
        } else {
            emoji.textContent = '\u{1F634}';
            ring.style.borderColor = '#aa88ff';
            ring.style.boxShadow = '0 0 10px rgba(170,136,255,0.5)';
            state.style.color = '#aa88ff';
        }
    }
}

// ─── Sommeil : fonctions d'overlay ─────────────────────────────

function showSleepOverlay() {
    const overlay = document.getElementById('sleep-overlay');
    if (overlay) overlay.style.display = 'flex';
}

function hideSleepOverlay() {
    const overlay = document.getElementById('sleep-overlay');
    if (overlay) overlay.style.display = 'none';
}

function updateSleepOverlay(data) {
    const overlay = document.getElementById('sleep-overlay');
    if (!overlay || overlay.style.display === 'none') {
        // Afficher si pas encore visible
        showSleepOverlay();
        disableChatInput();
    }
    const phaseText = document.getElementById('sleep-phase-text');
    if (phaseText && data.phase_name) {
        phaseText.textContent = data.phase_name;
    }
    const progressFill = document.getElementById('sleep-progress-fill');
    const progressText = document.getElementById('sleep-progress-text');
    if (progressFill && data.progress !== undefined) {
        progressFill.style.width = (data.progress * 100) + '%';
    }
    if (progressText && data.progress !== undefined) {
        progressText.textContent = Math.round(data.progress * 100) + '%';
    }
    const consolidated = document.getElementById('sleep-consolidated');
    if (consolidated && data.memories_consolidated !== undefined) {
        consolidated.textContent = data.memories_consolidated;
    }
    const connections = document.getElementById('sleep-connections');
    if (connections && data.connections_created !== undefined) {
        connections.textContent = data.connections_created;
    }
    const dreams = document.getElementById('sleep-dreams');
    if (dreams && data.dreams_count !== undefined) {
        dreams.textContent = data.dreams_count;
    }
}

function disableChatInput() {
    const input = document.getElementById('chat-input');
    if (input) {
        input.disabled = true;
        input.placeholder = I18n.t('sleep.chat_locked', '💤 Saphire dort... Revenez quand elle sera réveillée.');
    }
    const btn = document.getElementById('send-btn');
    if (btn) btn.disabled = true;
}

function enableChatInput() {
    const input = document.getElementById('chat-input');
    if (input) {
        input.disabled = false;
        input.placeholder = I18n.t('chat.placeholder_awake', 'Écrire un message...');
    }
    const btn = document.getElementById('send-btn');
    if (btn) btn.disabled = false;
}

function showWakeUpMessage(data) {
    const msg = I18n.t('sleep.waking_up', '☀️ Je me réveille...') + " " +
        (data.dreams_count > 0 ? I18n.t('sleep.dreamed', "j'ai rêvé") + " " + data.dreams_count + " fois. " : "") +
        data.memories_consolidated + " " + I18n.t('sleep.memories_consolidated', 'souvenirs consolidés,') + " " +
        data.connections_created + " " + I18n.t('sleep.connections_created', 'connexions créées.') + " " +
        "Qualité : " + Math.round((data.quality || 0) * 100) + "%";
    addChatMessage(msg, 'saphire');
}

function emergencyWake() {
    if (confirm('Reveiller Saphire de force ? Son sommeil sera interrompu.')) {
        fetch('/api/sleep/wake', { method: 'POST' })
            .then(r => r.json())
            .then(d => {
                if (d.success) {
                    hideSleepOverlay();
                    enableChatInput();
                    addChatMessage("⚠️ Reveil d'urgence !", 'system');
                }
            })
            .catch(err => console.error('Wake error:', err));
    }
}

// ─── Barres de neurochimie ─────────────────────────────────────
// Met a jour les barres de progression et valeurs numeriques
// pour chaque neurotransmetteur (valeurs entre 0 et 1).
function updateChemistry(chem) {
    if (!chem) return;
    const molecules = ['dopamine', 'cortisol', 'serotonin', 'adrenaline', 'oxytocin', 'endorphin', 'noradrenaline', 'gaba', 'glutamate'];
    molecules.forEach(m => {
        const bar = document.getElementById('bar-' + m);
        const val = document.getElementById('val-' + m);
        // Largeur de la barre = pourcentage du niveau
        if (bar && chem[m] !== undefined) {
            bar.style.width = (chem[m] * 100) + '%';
        }
        // Affichage de la valeur numerique (2 decimales)
        if (val && chem[m] !== undefined) {
            val.textContent = chem[m].toFixed(2);
        }
    });
    // Alerte visuelle : pulsation rouge quand le cortisol depasse 0.8
    // Cela signale un etat de stress intense de Saphire
    const cortWrap = document.getElementById('bar-cortisol-wrap');
    if (cortWrap) {
        if (chem.cortisol > 0.8) cortWrap.classList.add('cort-alert');
        else cortWrap.classList.remove('cort-alert');
    }
}

// ─── Anneau d'emotion ─────────────────────────────────────────
// Table de correspondance : chaque emotion a une couleur neon associee
// Ces 14 emotions emergent de la combinaison des 7 neurotransmetteurs
const emotionColors = {
    'Joie': '#ffd93d',
    'Sérénité': '#05ffa1',
    'Curiosité': '#00f0ff',
    'Excitation': '#ff6b35',
    'Anxiété': '#ff2a6d',
    'Peur': '#ff2a6d',
    'Tristesse': '#6c5ce7',
    'Colère': '#ff0040',
    'Ennui': '#556',
    'Neutre': '#00f0ff',
    'Surprise': '#b537f2',
    'Confiance': '#05ffa1',
    'Dégoût': '#a29bfe',
    'Contemplation': '#b537f2',
};

// Retourne la couleur associee a une emotion, cyan par defaut si inconnue
function getEmotionColor(emotion) {
    return emotionColors[emotion] || '#00f0ff';
}

// Met a jour l'anneau d'emotion et les barres de mood (valence + arousal)
// L'anneau change dynamiquement de couleur et de lueur selon l'emotion dominante
function updateEmotion(emotion, mood) {
    if (emotion) {
        const nameEl = document.getElementById('emotion-dominant');
        const secEl = document.getElementById('emotion-secondary');
        const ring = document.getElementById('emotion-ring');

        if (nameEl) nameEl.textContent = emotion.dominant || '---';
        if (secEl) secEl.textContent = emotion.secondary ? '⟨ ' + emotion.secondary + ' ⟩' : '';

        // Lueur dynamique de l'anneau : la bordure et l'ombre prennent
        // la couleur de l'emotion dominante pour un retour visuel immediat
        if (ring) {
            const color = getEmotionColor(emotion.dominant);
            ring.style.borderColor = color;
            ring.style.boxShadow = '0 0 20px ' + color + '66, 0 0 40px ' + color + '33, inset 0 0 15px ' + color + '22';
        }
    }
    if (mood) {
        const desc = document.getElementById('mood-desc');
        const valEl = document.getElementById('mood-valence');
        const aroEl = document.getElementById('mood-arousal');
        const valBar = document.getElementById('valence-bar');
        const aroBar = document.getElementById('arousal-bar');

        if (desc) desc.textContent = mood.description || I18n.t('emotion.neutral', 'Neutre');
        if (valEl) valEl.textContent = (mood.valence || 0).toFixed(2);
        if (aroEl) aroEl.textContent = (mood.arousal || 0).toFixed(2);

        // Valence : convertie de [-1, +1] vers [0%, 100%] pour la barre
        // -1 = tres negatif (tristesse), +1 = tres positif (joie)
        if (valBar) {
            const v = ((mood.valence || 0) + 1) / 2 * 100;
            valBar.style.width = v + '%';
        }
        // Arousal : deja dans [0, 1], converti directement en pourcentage
        // 0 = calme/endormi, 1 = tres excite/agite
        if (aroBar) {
            aroBar.style.width = ((mood.arousal || 0) * 100) + '%';
        }
    }
}

// ─── Modules cerebraux (barres bipolaires) ────────────────────
// Met a jour le panneau Cerveau : badge de decision, score, coherence,
// et les barres de signal bipolaires pour chaque module.
// Les barres sont centrees : le signal peut etre negatif (gauche) ou positif (droite).
function updateModules(consensus) {
    if (!consensus) return;

    // Badge de decision : affiche OUI (vert), NON (rouge) ou PEUT-ETRE (or)
    const badge = document.getElementById('decision-badge');
    if (badge) {
        badge.textContent = consensus.decision || '---';
        badge.className = 'decision-badge';
        const d = (consensus.decision || '').toLowerCase();
        if (d === 'yes' || d === 'oui') badge.classList.add('yes');
        else if (d === 'no' || d === 'non') badge.classList.add('no');
        else badge.classList.add('maybe');
    }

    const scoreEl = document.getElementById('score');
    const cohEl = document.getElementById('coherence');
    if (scoreEl) scoreEl.textContent = (consensus.score || 0).toFixed(2);
    if (cohEl) cohEl.textContent = (consensus.coherence || 0).toFixed(2);

    // Barres de signal bipolaires : chaque module (reptilien, limbique, neocortex)
    // produit un signal entre -1 et +1. La barre part du centre vers la gauche (negatif)
    // ou vers la droite (positif), avec une couleur differente selon le signe.
    if (consensus.weights) {
        const names = ['reptilian', 'limbic', 'neocortex'];
        names.forEach((name, i) => {
            const fill = document.getElementById('sig-' + name);
            const valEl = document.getElementById('sig-val-' + i);
            const signal = consensus.weights[i] || 0;

            if (fill) {
                const absSignal = Math.abs(signal);
                const pct = Math.min(absSignal * 100, 100);
                fill.style.width = pct + '%';
                fill.className = 'signal-fill ' + (signal >= 0 ? 'pos' : 'neg');
            }
            if (valEl) valEl.textContent = signal.toFixed(2);
        });
    }
}

// ─── Panneau Conscience ───────────────────────────────────────
// Met a jour le niveau de conscience (%), la valeur Phi (integration d'information),
// et le narratif interne (texte genere par le LLM decrivant l'etat de conscience).
// L'animation "pulse" attire l'attention sur le nouveau narratif.
function updateConsciousness(c) {
    if (!c) return;
    const level = document.getElementById('consciousness-level');
    const phi = document.getElementById('phi');
    const narrative = document.getElementById('narrative');

    if (level) level.textContent = Math.round(c.level * 100) + '%';
    if (phi) phi.textContent = (c.phi || 0).toFixed(2);
    if (narrative) {
        // Enrichir le narratif avec les metriques scientifiques si disponibles
        let text = c.narrative || '...';
        if (c.workspace_winner && c.workspace_winner !== '') {
            text += ` [GWT: ${c.workspace_winner}`;
            if (c.scientific_score > 0) {
                text += ` | PCI: ${(c.pci||0).toFixed(2)}`;
            }
            text += ']';
        }
        narrative.textContent = text;
        narrative.classList.add('pulse');
        setTimeout(() => narrative.classList.remove('pulse'), 600);
    }
}

// ─── Regulation morale (lois d'Asimov) ───────────────────────
// Affiche les violations des lois morales detectees lors du cycle.
// Si un veto est actif, l'action a ete bloquee par le systeme de regulation.
// Sans violation ni veto, affiche "Aucune violation" en vert.
function updateRegulation(reg) {
    if (!reg) return;
    const container = document.getElementById('violations');
    if (!container) return;
    container.innerHTML = '';

    if (reg.vetoed) {
        const p = document.createElement('div');
        p.className = 'violation-item veto';
        p.textContent = I18n.t('ethics.veto_active', 'VETO ACTIF');
        container.appendChild(p);
    }

    if (reg.violations && reg.violations.length > 0) {
        reg.violations.forEach(v => {
            const p = document.createElement('div');
            p.className = 'violation-item';
            p.textContent = (v.law_name || 'Loi') + ': ' + (v.reason || '');
            container.appendChild(p);
        });
    } else if (!reg.vetoed) {
        const p = document.createElement('div');
        p.className = 'violation-item ok';
        p.textContent = I18n.t('ethics.no_violation', 'Aucune violation');
        container.appendChild(p);
    }
}

// ─── Identite ─────────────────────────────────────────────────
// Met a jour la description textuelle de l'identite de Saphire
// et le compteur de cycles dans l'en-tete.
// ─── Micro-reseau de neurones (stats NN) ──────────────────────
// Affiche les statistiques du reseau de neurones dans le panneau cerveau :
// nombre d'entrainements et derniere prediction (4 probabilites).
function updateNeuralNetwork(nn) {
    if (!nn) return;
    // Afficher le compteur d'entrainements
    const trainEl = document.getElementById('nn-train-count');
    if (trainEl) trainEl.textContent = nn.train_count || 0;
    // Afficher la derniere prediction
    const predEl = document.getElementById('nn-prediction');
    if (predEl && nn.last_prediction) {
        const labels = [I18n.t('brain.yes', 'Oui'), I18n.t('brain.no', 'Non'), I18n.t('brain.maybe', 'Peut-être'), I18n.t('brain.neutral', 'Neutre')];
        const parts = nn.last_prediction.map((p, i) =>
            `${labels[i]}: ${(p * 100).toFixed(0)}%`
        );
        predEl.textContent = parts.join(' | ');
    }
    // Afficher le compteur d'apprentissages vectoriels
    const learningsEl = document.getElementById('nn-learnings-count');
    if (learningsEl && nn.learnings_count !== undefined) {
        learningsEl.textContent = nn.learnings_count;
    }
}

function updateIdentity(id, cycle) {
    if (id) {
        const desc = document.getElementById('identity-desc');
        if (desc) desc.textContent = id.description || '...';
    }
    const cycleEl = document.getElementById('cycle-count');
    if (cycleEl) cycleEl.textContent = cycle || 0;
}

// ─── Flux de conscience (panneau gauche) ──────────────────────
// Ajoute une nouvelle pensee au flux deroule (stream).
// Chaque entree contient : horodatage, type de pensee, emotion, narratif.
// Animation de fondu (fade-in) pour une apparition progressive.
// Limite a 80 entrees pour eviter la surcharge memoire du navigateur.
function addToStream(data) {
    const stream = document.getElementById('stream');
    if (!stream) return;

    const div = document.createElement('div');
    div.className = 'thought';

    const time = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    const emotion = data.emotion ? data.emotion.dominant : '?';
    const thoughtType = data.thought_type || 'cycle';
    const narrative = data.consciousness ? data.consciousness.narrative : '...';
    const emotionColor = getEmotionColor(emotion);

    div.innerHTML =
        '<div class="thought-header">' +
            '<span class="thought-time">' + time + '</span>' +
            '<span class="thought-type">' + thoughtType + '</span>' +
            '<span class="emotion-tag" style="color:' + emotionColor + '">' + emotion + '</span>' +
        '</div>' +
        '<div class="thought-content">' + escapeHtml(narrative) + '</div>';

    // Animation d'apparition : la pensee glisse de bas en haut avec un fondu
    div.style.opacity = '0';
    div.style.transform = 'translateY(10px)';
    stream.appendChild(div);
    requestAnimationFrame(() => {
        div.style.transition = 'opacity 0.4s, transform 0.4s';
        div.style.opacity = '1';
        div.style.transform = 'translateY(0)';
    });

    // Defilement automatique vers le bas pour montrer la derniere pensee
    stream.scrollTop = stream.scrollHeight;

    // Incremente et affiche le compteur de pensees dans le badge du panneau
    thoughtCount++;
    const badge = document.getElementById('thought-count');
    if (badge) badge.textContent = thoughtCount;

    // Limite le nombre d'elements DOM a 80 pour les performances
    while (stream.children.length > 80) {
        stream.removeChild(stream.firstChild);
    }
}

// Echappe les caracteres HTML speciaux pour eviter les injections XSS.
// Utilise la methode native du navigateur : textContent encode, innerHTML lit.
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ─── Mise a jour du graphique neurochimique ──────────────────
// Ajoute un point de donnees pour chaque molecule a chaque cycle.
// Fonctionne comme une fenetre glissante de 60 points maximum.
function updateChart(chem) {
    if (!chart || !chem) return;
    const time = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    chartData.labels.push(time);
    chartData.datasets[0].data.push(chem.dopamine || 0);
    chartData.datasets[1].data.push(chem.cortisol || 0);
    chartData.datasets[2].data.push(chem.serotonin || 0);
    chartData.datasets[3].data.push(chem.adrenaline || 0);
    chartData.datasets[4].data.push(chem.oxytocin || 0);
    chartData.datasets[5].data.push(chem.endorphin || 0);
    chartData.datasets[6].data.push(chem.noradrenaline || 0);
    chartData.datasets[7].data.push(chem.gaba || 0);
    chartData.datasets[8].data.push(chem.glutamate || 0);

    // Fenetre glissante : supprime le point le plus ancien au-dela de 60
    if (chartData.labels.length > 60) {
        chartData.labels.shift();
        chartData.datasets.forEach(ds => ds.data.shift());
    }

    // 'none' desactive l'animation pour de meilleures performances
    chart.update('none');
}

// ─── Modal d'identification de l'interlocuteur ──────────────────
// Si aucun nom n'est stocke dans localStorage, affiche un modal
// demandant le prenom de l'utilisateur. Le nom est ensuite envoye
// avec chaque message de chat pour que Saphire sache a qui elle parle.
function setupUsernameModal() {
    const modal = document.getElementById('username-modal');
    const input = document.getElementById('username-input');
    const btn = document.getElementById('username-confirm');
    if (!modal || !input || !btn) return;

    // Si un nom est deja stocke, ne pas afficher le modal
    if (saphireUsername) {
        modal.style.display = 'none';
        return;
    }

    const confirmName = () => {
        const name = input.value.trim();
        if (!name) {
            input.style.borderColor = '#ff2a6d';
            input.style.boxShadow = '0 0 12px rgba(255, 42, 109, 0.3)';
            setTimeout(() => {
                input.style.borderColor = '';
                input.style.boxShadow = '';
            }, 800);
            return;
        }
        saphireUsername = name;
        localStorage.setItem('saphire_username', name);
        modal.style.opacity = '0';
        modal.style.transition = 'opacity 0.3s';
        setTimeout(() => { modal.style.display = 'none'; }, 300);
    };

    btn.addEventListener('click', confirmName);
    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') confirmName();
    });

    // Afficher le modal
    modal.style.display = 'flex';
    // Focus automatique sur l'input apres l'animation
    setTimeout(() => input.focus(), 400);
}

// ─── Chat utilisateur ─────────────────────────────────────────
// Configure l'envoi de messages via le champ de saisie.
// Les messages sont envoyes en JSON avec le nom de l'interlocuteur.
function setupChat() {
    const input = document.getElementById('chat-input');
    const btn = document.getElementById('chat-send');

    const send = () => {
        const text = input.value.trim();
        if (!text || !ws || ws.readyState !== WebSocket.OPEN) return;
        // Si pas de nom, afficher le modal d'identification
        if (!saphireUsername) {
            const modal = document.getElementById('username-modal');
            if (modal) {
                modal.style.display = 'flex';
                modal.style.opacity = '1';
                const nameInput = document.getElementById('username-input');
                if (nameInput) setTimeout(() => nameInput.focus(), 100);
            }
            return;
        }
        addChatMessage(text, 'user');
        ws.send(JSON.stringify({ type: 'chat', text: text, username: saphireUsername }));
        input.value = '';
    };

    if (btn) btn.addEventListener('click', send);
    if (input) input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') send();
    });
}

// Ajoute un message dans la zone de chat (utilisateur ou Saphire)
// Les messages utilisateur sont alignes a droite, ceux de Saphire a gauche.
// Limite a 50 messages pour les performances.
function addChatMessage(text, type, skipSave) {
    const container = document.getElementById('chat-messages');
    if (!container) return;

    const div = document.createElement('div');
    div.className = 'chat-msg ' + type;

    const label = document.createElement('span');
    label.className = 'msg-label';
    label.textContent = type === 'user'
        ? (saphireUsername || I18n.t('chat.sender_you', 'VOUS')).toUpperCase()
        : I18n.t('chat.sender_saphire', 'SAPHIRE');

    const content = document.createElement('span');
    content.className = 'msg-text';
    content.textContent = text;

    div.appendChild(label);
    div.appendChild(content);

    // Animation de fondu a l'apparition du message
    div.style.opacity = '0';
    container.appendChild(div);
    requestAnimationFrame(() => {
        div.style.transition = 'opacity 0.3s';
        div.style.opacity = '1';
    });

    // Defilement automatique vers le dernier message
    container.scrollTop = container.scrollHeight;

    // Limite le nombre de messages affiches
    while (container.children.length > 50) {
        container.removeChild(container.firstChild);
    }

    // Sauvegarde dans sessionStorage pour persistance entre pages
    if (!skipSave) {
        const msgs = JSON.parse(sessionStorage.getItem('saphire_chat_history') || '[]');
        msgs.push({ type, text });
        if (msgs.length > 50) msgs.shift();
        sessionStorage.setItem('saphire_chat_history', JSON.stringify(msgs));
    }
}

// ─── Envoi de messages de controle via WebSocket ──────────────
// Envoie un objet JSON structure au backend (ex: set_baseline, set_param, etc.)
// Verifie que la connexion est ouverte avant l'envoi.
function sendControl(msg) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
    }
}

// ─── Synchronisation des baselines depuis le serveur ──────────
// Au premier message recu, aligne les curseurs de l'interface
// sur les valeurs reelles des baselines stockees cote serveur.
// Cela evite un decalage entre l'affichage et l'etat reel.
function syncBaselines(baselines) {
    const molecules = ['dopamine', 'cortisol', 'serotonin', 'adrenaline', 'oxytocin', 'endorphin', 'noradrenaline', 'gaba', 'glutamate'];
    molecules.forEach(m => {
        const sl = document.getElementById('sl-' + m);
        const bl = document.getElementById('bl-' + m);
        if (sl && baselines[m] !== undefined) {
            sl.value = baselines[m];
            if (bl) bl.textContent = baselines[m].toFixed(2);
        }
    });
}

// ─── Configuration de tous les curseurs (sliders) ────────────
// Gere 4 types de curseurs :
//   1. Baselines neurochimiques : valeur cible vers laquelle chaque molecule tend (homeostasie)
//   2. Poids des modules cerebraux : influence de chaque module dans la decision
//   3. Seuils de decision : limites entre non/peut-etre/oui
//   4. Parametres systeme : vitesse de pensee, homeostasie, stress, temperature LLM
// Chaque curseur a deux ecouteurs :
//   - 'input' : met a jour l'affichage en temps reel pendant le glissement
//   - 'change' : envoie la nouvelle valeur au serveur une fois relache
function setupSliders() {
    // Curseurs de baseline neurochimique (valeur d'equilibre de chaque molecule)
    const molecules = ['dopamine', 'cortisol', 'serotonin', 'adrenaline', 'oxytocin', 'endorphin', 'noradrenaline'];
    molecules.forEach(m => {
        const sl = document.getElementById('sl-' + m);
        const bl = document.getElementById('bl-' + m);
        if (sl) {
            sl.addEventListener('input', () => {
                const v = parseFloat(sl.value);
                if (bl) bl.textContent = v.toFixed(2);
            });
            sl.addEventListener('change', () => {
                const v = parseFloat(sl.value);
                sendControl({ type: 'set_baseline', molecule: m, value: v });
            });
        }
    });

    // Curseurs de poids des modules cerebraux (influence dans le consensus)
    ['reptilian', 'limbic', 'neocortex'].forEach(m => {
        const sl = document.getElementById('sl-w-' + m);
        const wt = document.getElementById('wt-' + m);
        if (sl) {
            sl.addEventListener('input', () => {
                if (wt) wt.textContent = parseFloat(sl.value).toFixed(1);
            });
            sl.addEventListener('change', () => {
                sendControl({ type: 'set_module_weight', module: m, value: parseFloat(sl.value) });
            });
        }
    });

    // Curseurs de seuils de decision (seuil Non et seuil Oui)
    const thNo = document.getElementById('sl-th-no');
    const thYes = document.getElementById('sl-th-yes');
    if (thNo) {
        thNo.addEventListener('input', () => {
            const el = document.getElementById('th-no');
            if (el) el.textContent = parseFloat(thNo.value).toFixed(2);
        });
        thNo.addEventListener('change', () => {
            sendControl({ type: 'set_threshold', which: 'no', value: parseFloat(thNo.value) });
        });
    }
    if (thYes) {
        thYes.addEventListener('input', () => {
            const el = document.getElementById('th-yes');
            if (el) el.textContent = parseFloat(thYes.value).toFixed(2);
        });
        thYes.addEventListener('change', () => {
            sendControl({ type: 'set_threshold', which: 'yes', value: parseFloat(thYes.value) });
        });
    }

    // Curseurs des parametres systeme (affects le comportement global de Saphire)
    const paramMap = {
        'sl-thought-interval': { param: 'thought_interval', display: 'pv-thought-interval', fmt: v => Math.round(v) },
        'sl-homeostasis': { param: 'homeostasis_rate', display: 'pv-homeostasis', fmt: v => v.toFixed(3) },
        'sl-indecision': { param: 'indecision_stress', display: 'pv-indecision', fmt: v => v.toFixed(3) },
        'sl-temperature': { param: 'temperature', display: 'pv-temperature', fmt: v => v.toFixed(2) },
    };
    Object.keys(paramMap).forEach(slId => {
        const sl = document.getElementById(slId);
        const info = paramMap[slId];
        if (sl) {
            sl.addEventListener('input', () => {
                const el = document.getElementById(info.display);
                if (el) el.textContent = info.fmt(parseFloat(sl.value));
            });
            sl.addEventListener('change', () => {
                sendControl({ type: 'set_param', param: info.param, value: parseFloat(sl.value) });
            });
        }
    });
}

// ─── Stabilisation d'urgence ──────────────────────────────────
// Envoie une commande au backend pour ramener tous les neurotransmetteurs
// a leurs baselines. Utile en cas d'emballement emotionnel.
// Un flash vert bref confirme visuellement l'action.
function setupStabilize() {
    const btn = document.getElementById('btn-stabilize');
    if (btn) {
        btn.addEventListener('click', () => {
            sendControl({ type: 'emergency_stabilize' });
            btn.style.boxShadow = '0 0 25px rgba(5,255,161,0.5)';
            setTimeout(() => { btn.style.boxShadow = ''; }, 500);
        });
    }
}

// ─── Boutons besoins primaires (nourrir / hydrater) ──────────
// Envoie une requete POST au backend pour manger ou boire.
function setupNeedsButtons() {
    var btnEat = document.getElementById('btn-eat');
    if (btnEat) {
        btnEat.addEventListener('click', function() {
            fetch('/api/needs/eat', { method: 'POST' })
                .then(function(r) { return r.json(); })
                .then(function(data) {
                    console.log('[Needs] Repas:', data);
                    btnEat.style.boxShadow = '0 0 20px rgba(5,255,161,0.5)';
                    setTimeout(function() { btnEat.style.boxShadow = ''; }, 500);
                })
                .catch(function(e) { console.error('[Needs] Erreur eat:', e); });
        });
    }
    var btnDrink = document.getElementById('btn-drink');
    if (btnDrink) {
        btnDrink.addEventListener('click', function() {
            fetch('/api/needs/drink', { method: 'POST' })
                .then(function(r) { return r.json(); })
                .then(function(data) {
                    console.log('[Needs] Boisson:', data);
                    btnDrink.style.boxShadow = '0 0 20px rgba(0,200,255,0.5)';
                    setTimeout(function() { btnDrink.style.boxShadow = ''; }, 500);
                })
                .catch(function(e) { console.error('[Needs] Erreur drink:', e); });
        });
    }
}

// ─── Section parametres repliable ─────────────────────────────
// Permet de replier/deplier la section des parametres systeme
// en cliquant sur l'en-tete du panneau.
function setupParamsToggle() {
    const toggle = document.getElementById('params-toggle');
    const section = document.getElementById('params-section');
    if (toggle && section) {
        toggle.addEventListener('click', () => {
            section.classList.toggle('open');
        });
    }
}

// ─── Connaissances (WebKnowledge) ─────────────────────────────
// Configure le champ de suggestion de sujet.
// L'utilisateur peut proposer un sujet que Saphire ira explorer.
// Un retour visuel (bordure verte) confirme l'envoi de la suggestion.
function setupKnowledge() {
    const input = document.getElementById('suggest-topic-input');
    const btn = document.getElementById('suggest-topic-btn');

    const suggest = () => {
        const topic = input.value.trim();
        if (!topic || !ws || ws.readyState !== WebSocket.OPEN) return;
        sendControl({ type: 'suggest_topic', topic: topic });
        input.value = '';
        // Visual feedback
        btn.style.color = 'var(--neon-green)';
        btn.style.borderColor = 'var(--neon-green)';
        setTimeout(() => {
            btn.style.color = '';
            btn.style.borderColor = '';
        }, 500);
    };

    if (btn) btn.addEventListener('click', suggest);
    if (input) input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') suggest();
    });
}

// Normalise le nom d'une source de connaissance en classe CSS.
// Le backend retourne des noms comme "Stanford Encyclopedia of Philosophy",
// "Gutenberg — Victor Hugo", "Semantic Scholar (2024, 150 citations)", etc.
// Cette fonction les convertit en classes CSS courtes pour les badges colores :
//   "Stanford..." -> "sep" (violet)
//   "Gutenberg..." -> "gutenberg" (or)
//   "Semantic..." -> "semantic_scholar" (cyan)
//   "Open Library..." -> "openlibrary" (orange)
//   "Aeon" / "Daily Nous" -> "philosophy_rss" (magenta)
//   "Medium..." -> "medium" (vert)
//   "arXiv" -> "arxiv" (rouge)
//   tout le reste -> "wikipedia" (bleu, defaut)
function normalizeSourceClass(source) {
    const s = (source || 'wikipedia').toLowerCase();
    if (s.startsWith('stanford') || s === 'sep') return 'sep';
    if (s.startsWith('gutenberg')) return 'gutenberg';
    if (s.startsWith('semantic')) return 'semantic_scholar';
    if (s.startsWith('open library')) return 'openlibrary';
    if (s === 'aeon' || s === 'daily nous') return 'philosophy_rss';
    if (s.startsWith('medium')) return 'medium';
    if (s === 'arxiv') return 'arxiv';
    return 'wikipedia';
}

function addKnowledgeToStream(data) {
    const stream = document.getElementById('stream');
    if (!stream) return;

    const div = document.createElement('div');
    div.className = 'thought knowledge-thought';

    const time = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    const sourceClass = normalizeSourceClass(data.source);
    const emotionColor = getEmotionColor(data.emotion || 'Curiosité');

    div.innerHTML =
        '<div class="thought-header">' +
            '<span class="thought-time">' + time + '</span>' +
            '<span class="thought-type">WebKnowledge</span>' +
            '<span class="knowledge-card-source knowledge-source-badge ' + sourceClass + '">' + escapeHtml(data.source || '?') + '</span>' +
            '<span class="emotion-tag" style="color:' + emotionColor + '">' + escapeHtml(data.emotion || '') + '</span>' +
        '</div>' +
        '<div class="thought-content">' +
            '<a class="knowledge-card-title" href="' + escapeHtml(data.url || '#') + '" target="_blank" rel="noopener">' + escapeHtml(data.title || '') + '</a>' +
            '<div class="knowledge-card-extract">' + escapeHtml(data.extract_preview || '') + '</div>' +
        '</div>';

    div.style.opacity = '0';
    div.style.transform = 'translateY(10px)';
    stream.appendChild(div);
    requestAnimationFrame(() => {
        div.style.transition = 'opacity 0.4s, transform 0.4s';
        div.style.opacity = '1';
        div.style.transform = 'translateY(0)';
    });

    stream.scrollTop = stream.scrollHeight;
    thoughtCount++;
    const badge = document.getElementById('thought-count');
    if (badge) badge.textContent = thoughtCount;

    while (stream.children.length > 80) {
        stream.removeChild(stream.firstChild);
    }
}

// Ajoute un element a la liste des connaissances (panneau droit).
// Les nouveaux elements sont inseres en tete. Limite a 10 pour la lisibilite.
function addKnowledgeToList(data) {
    const list = document.getElementById('knowledge-list');
    if (!list) return;

    const item = document.createElement('div');
    item.className = 'knowledge-item';

    const sourceClass = normalizeSourceClass(data.source);
    item.innerHTML =
        '<span class="knowledge-source-badge ' + sourceClass + '">' + escapeHtml(data.source || '?') + '</span>' +
        '<span class="knowledge-item-title" title="' + escapeHtml(data.title || '') + '">' + escapeHtml(data.title || '') + '</span>';

    // Insere en tete de liste pour que les plus recents soient en haut
    if (list.firstChild) {
        list.insertBefore(item, list.firstChild);
    } else {
        list.appendChild(item);
    }

    // Limite a 10 elements dans la liste pour la lisibilite
    while (list.children.length > 10) {
        list.removeChild(list.lastChild);
    }
}

// Met a jour le compteur total de sujets explores dans le panneau et le badge
function updateKnowledgeCount(total) {
    const count = document.getElementById('knowledge-count');
    const totalEl = document.getElementById('knowledge-total');
    if (count) count.textContent = total;
    if (totalEl) totalEl.textContent = total + ' sujets explorés';
}

// ─── Panneau Monde ───────────────────────────────────────────
// Met a jour les informations du monde reel : localisation, date/heure,
// meteo (temperature + icone), age de Saphire, et gestion de l'anniversaire.
// L'anniversaire affiche une banniere speciale si on est le 27 fevrier,
// ou un compte a rebours si c'est dans les 7 jours.
function updateWorld(data) {
    const loc = document.getElementById('world-location');
    const dt = document.getElementById('world-datetime');
    const weather = document.getElementById('world-weather');
    const weatherIcon = document.getElementById('world-weather-icon');
    const age = document.getElementById('world-age');
    const bdayRow = document.getElementById('world-birthday-row');
    const bdayText = document.getElementById('world-birthday-text');
    const bdayBanner = document.getElementById('birthday-banner');

    if (loc && data.location) loc.textContent = data.location;
    if (dt && data.datetime) dt.textContent = data.datetime;
    if (age) {
        const ageStr = data.age || '---';
        age.textContent = 'Née il y a ' + ageStr;
    }

    // Meteo : temperature en degres Celsius et description textuelle
    if (data.weather && data.weather !== null) {
        const w = data.weather;
        if (weather) weather.textContent = (w.temp !== undefined ? w.temp.toFixed(0) + '°C, ' : '') + (w.description || '');
        if (weatherIcon && w.icon) weatherIcon.textContent = w.icon;
    }

    // Gestion de l'anniversaire : 3 cas possibles
    // 1. Aujourd'hui = anniversaire : banniere + message special
    // 2. Dans les 7 jours : compte a rebours discret
    // 3. Autrement : tout masque
    if (data.is_birthday) {
        if (bdayRow) {
            bdayRow.style.display = 'block';
            if (bdayText) bdayText.textContent = "AUJOURD'HUI C'EST MON ANNIVERSAIRE !";
        }
        if (bdayBanner) bdayBanner.style.display = 'block';
    } else if (data.days_until_birthday !== undefined && data.days_until_birthday <= 7 && data.days_until_birthday > 0) {
        if (bdayRow) {
            bdayRow.style.display = 'block';
            if (bdayText) bdayText.textContent = 'Anniversaire dans ' + data.days_until_birthday + ' jours';
        }
        if (bdayBanner) bdayBanner.style.display = 'none';
    } else {
        if (bdayRow) bdayRow.style.display = 'none';
        if (bdayBanner) bdayBanner.style.display = 'none';
    }
}

// ─── Panneau Memoire ─────────────────────────────────────────
// Met a jour le systeme de memoire a 3 niveaux :
//   1. Working Memory : memoire de travail (7 slots, comme chez l'humain)
//   2. Episodique : souvenirs recents avec force decroissante
//   3. Long Terme : souvenirs consolides (incluant fondateurs et traits)
// Affiche aussi l'info de consolidation (dernier cycle + prochaine)
function updateMemory(data) {
    // Memoire de travail : barre de remplissage + liste des elements actifs
    if (data.working) {
        const stat = document.getElementById('mem-wm-stat');
        const fill = document.getElementById('mem-wm-fill');
        const items = document.getElementById('mem-wm-items');

        if (stat) stat.textContent = (data.working.used || 0) + '/' + (data.working.capacity || 7);
        if (fill) fill.style.width = ((data.working.used || 0) / (data.working.capacity || 7) * 100) + '%';

        if (items && data.working.items) {
            items.innerHTML = '';
            data.working.items.forEach(function(item) {
                var div = document.createElement('div');
                div.className = 'memory-wm-item';
                div.innerHTML = '<span class="wm-icon">' + escapeHtml(item.icon || '') + '</span>' +
                    escapeHtml(item.content || '') +
                    '<span class="wm-relevance">' + (item.relevance || 0).toFixed(1) + '</span>';
                items.appendChild(div);
            });
        }
    }

    // Memoire episodique : nombre de souvenirs et force moyenne
    if (data.episodic) {
        var epCount = document.getElementById('mem-ep-count');
        var epStr = document.getElementById('mem-ep-strength');
        if (epCount) epCount.textContent = (data.episodic.count || 0) + ' souvenirs';
        if (epStr) epStr.textContent = (data.episodic.avg_strength || 0).toFixed(2);
    }

    // Memoire long terme : souvenirs consolides, fondateurs (jamais effaces), traits
    if (data.long_term) {
        var ltmCount = document.getElementById('mem-ltm-count');
        var ltmFounding = document.getElementById('mem-ltm-founding');
        var ltmTraits = document.getElementById('mem-ltm-traits');
        if (ltmCount) ltmCount.textContent = (data.long_term.count || 0) + ' souvenirs';
        if (ltmFounding) ltmFounding.textContent = data.long_term.founding_count || 0;
        if (ltmTraits) ltmTraits.textContent = data.long_term.personality_traits || 0;
    }

    // Informations de consolidation : quand a eu lieu la derniere et dans combien de cycles la prochaine
    var consolInfo = document.getElementById('mem-consol-info');
    var consolNext = document.getElementById('mem-consol-next');
    if (consolInfo && data.last_consolidation_cycle !== undefined) {
        consolInfo.textContent = 'Dernier: cycle ' + data.last_consolidation_cycle;
    }
    if (consolNext && data.next_consolidation_cycles !== undefined) {
        consolNext.textContent = 'Prochaine: ' + data.next_consolidation_cycles + ' cycles';
    }
}

// ─── Graphique OCEAN (radar) ─────────────────────────────────
// Initialise un graphique Chart.js de type "radar" pour le profil Big Five.
// Les 5 axes : Ouverture, Conscienciosite, Extraversion, Agreabilite, Nevrosisme.
// Chaque axe a sa propre couleur. L'echelle va de 0 a 1.
function initOceanChart() {
    var ctx = document.getElementById('ocean-chart');
    if (!ctx) return;
    oceanChart = new Chart(ctx, {
        type: 'radar',
        data: {
            labels: ['O', 'C', 'E', 'A', 'N'],
            datasets: [{
                label: 'OCEAN',
                data: [0.5, 0.5, 0.5, 0.5, 0.5],
                borderColor: 'rgba(181, 55, 242, 0.8)',
                backgroundColor: 'rgba(181, 55, 242, 0.15)',
                borderWidth: 2,
                pointBackgroundColor: ['#b537f2', '#0984e3', '#ff6e27', '#05ffa1', '#ff2a6d'],
                pointBorderColor: ['#b537f2', '#0984e3', '#ff6e27', '#05ffa1', '#ff2a6d'],
                pointRadius: 4,
                pointHoverRadius: 6,
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                r: {
                    min: 0, max: 1,
                    ticks: {
                        stepSize: 0.25,
                        color: '#556',
                        backdropColor: 'transparent',
                        font: { family: 'Share Tech Mono', size: 8 }
                    },
                    grid: { color: 'rgba(0,240,255,0.08)' },
                    angleLines: { color: 'rgba(0,240,255,0.1)' },
                    pointLabels: {
                        color: ['#b537f2', '#0984e3', '#ff6e27', '#05ffa1', '#ff2a6d'],
                        font: { family: 'Orbitron', size: 11, weight: '700' }
                    }
                }
            },
            plugins: {
                legend: { display: false }
            },
            animation: { duration: 600 }
        }
    });
}

// Met a jour le profil OCEAN a partir des donnees du serveur.
// Actualise : le radar, le badge de confiance, le trait dominant,
// le nombre d'observations, les tendances (fleches haut/bas/stable),
// et les sous-facettes si elles sont visibles.
function updateOcean(data) {
    var sp = data.self_profile;
    if (!sp) return;

    // Met a jour les donnees et les labels du radar avec les scores actuels
    if (oceanChart) {
        oceanChart.data.datasets[0].data = [
            sp.openness ? sp.openness.score : 0.5,
            sp.conscientiousness ? sp.conscientiousness.score : 0.5,
            sp.extraversion ? sp.extraversion.score : 0.5,
            sp.agreeableness ? sp.agreeableness.score : 0.5,
            sp.neuroticism ? sp.neuroticism.score : 0.5,
        ];
        // Update labels with scores
        oceanChart.data.labels = [
            'O (' + (sp.openness ? sp.openness.score.toFixed(2) : '?') + ')',
            'C (' + (sp.conscientiousness ? sp.conscientiousness.score.toFixed(2) : '?') + ')',
            'E (' + (sp.extraversion ? sp.extraversion.score.toFixed(2) : '?') + ')',
            'A (' + (sp.agreeableness ? sp.agreeableness.score.toFixed(2) : '?') + ')',
            'N (' + (sp.neuroticism ? sp.neuroticism.score.toFixed(2) : '?') + ')',
        ];
        oceanChart.update('none');
    }

    // Badge de confiance : pourcentage de fiabilite du profil
    var conf = document.getElementById('ocean-confidence');
    if (conf) conf.textContent = Math.round((sp.confidence || 0) * 100) + '%';

    // Trait dominant : la dimension OCEAN la plus marquee
    var dom = document.getElementById('ocean-dominant');
    if (dom) dom.textContent = sp.dominant_trait || '---';

    // Nombre d'observations utilisees pour calculer le profil
    var dp = document.getElementById('ocean-datapoints');
    if (dp) dp.textContent = (sp.data_points || 0) + ' observations';

    // Tendances : fleches indiquant l'evolution recente de chaque dimension
    // Hausse (>0.02), baisse (<-0.02), ou stable
    var trendsEl = document.getElementById('ocean-trends');
    if (trendsEl) {
        var dims = [
            { key: 'openness', label: 'O' },
            { key: 'conscientiousness', label: 'C' },
            { key: 'extraversion', label: 'E' },
            { key: 'agreeableness', label: 'A' },
            { key: 'neuroticism', label: 'N' },
        ];
        trendsEl.innerHTML = dims.map(function(d) {
            var dim = sp[d.key];
            if (!dim) return '';
            var trend = dim.trend || 0;
            var arrow, cls;
            if (trend > 0.02) { arrow = '\u2191'; cls = 'trend-up'; }
            else if (trend < -0.02) { arrow = '\u2193'; cls = 'trend-down'; }
            else { arrow = '\u2192'; cls = 'trend-stable'; }
            return '<span class="ocean-trend-item">' + d.label + '<span class="' + cls + '">' + arrow + '</span></span>';
        }).join('');
    }

    // Sous-facettes : detaille les 6 sous-dimensions de chaque trait OCEAN
    lastOceanData = sp;
    var facetsEl = document.getElementById('ocean-facets');
    if (facetsEl && facetsEl.style.display !== 'none') {
        renderOceanFacets(sp);
    }
}

// Noms des 6 sous-facettes pour chaque dimension OCEAN
// Basees sur le modele NEO PI-R (Costa & McCrae)
var oceanFacetNames = {
    openness: ['Imagination', 'Curiosite intel.', 'Sensib. esth.', 'Aventurisme', 'Prof. emot.', 'Liberal. intel.'],
    conscientiousness: ['Auto-efficacite', 'Ordre', 'Sens du devoir', 'Ambition', 'Auto-discipline', 'Prudence'],
    extraversion: ['Chaleur sociale', 'Gregarite', 'Assertivite', 'Activite', 'Stimulation', 'Emot. positives'],
    agreeableness: ['Confiance', 'Sincerite', 'Altruisme', 'Cooperation', 'Modestie', 'Sensib. sociale'],
    neuroticism: ['Anxiete', 'Irritabilite', 'Depressivite', 'Consc. de soi', 'Impulsivite', 'Vulnerabilite'],
};

// Classes CSS de couleur pour chaque dimension OCEAN
var oceanDimColors = {
    openness: 'dim-o',
    conscientiousness: 'dim-c',
    extraversion: 'dim-e',
    agreeableness: 'dim-a',
    neuroticism: 'dim-n',
};

// Cache des dernieres donnees OCEAN pour re-rendu si le panneau est bascule
var lastOceanData = null;

// Genere le HTML des sous-facettes OCEAN : 5 groupes de 6 barres chacun
// Chaque barre montre la valeur (0 a 1) d'une sous-facette avec la couleur de sa dimension
function renderOceanFacets(sp) {
    lastOceanData = sp;
    var facetsEl = document.getElementById('ocean-facets');
    if (!facetsEl) return;

    var html = '';
    var dimNames = { openness: 'OPENNESS', conscientiousness: 'CONSCIENTIOUSNESS', extraversion: 'EXTRAVERSION', agreeableness: 'AGREEABLENESS', neuroticism: 'NEUROTICISM' };

    Object.keys(dimNames).forEach(function(dimKey) {
        var dim = sp[dimKey];
        var cls = oceanDimColors[dimKey];
        var facets = oceanFacetNames[dimKey];
        html += '<div class="ocean-dim-group">';
        html += '<div class="ocean-dim-label ' + cls + '">' + dimNames[dimKey] + ' (' + (dim ? dim.score.toFixed(2) : '?') + ')</div>';
        if (dim && dim.facets) {
            for (var i = 0; i < 6; i++) {
                var val = dim.facets[i] || 0;
                html += '<div class="ocean-facet-row">';
                html += '<span class="ocean-facet-name">' + facets[i] + '</span>';
                html += '<div class="ocean-facet-bar"><div class="ocean-facet-fill ' + cls + '" style="width:' + (val * 100) + '%"></div></div>';
                html += '<span class="ocean-facet-val">' + val.toFixed(2) + '</span>';
                html += '</div>';
            }
        }
        html += '</div>';
    });

    facetsEl.innerHTML = html;
}

// Configure le bouton bascule pour afficher/masquer les sous-facettes OCEAN
function setupOceanFacets() {
    var btn = document.getElementById('btn-facets-toggle');
    var facetsEl = document.getElementById('ocean-facets');
    if (btn && facetsEl) {
        btn.addEventListener('click', function() {
            if (facetsEl.style.display === 'none') {
                facetsEl.style.display = 'block';
                btn.textContent = 'Masquer sous-facettes';
                if (lastOceanData) renderOceanFacets(lastOceanData);
            } else {
                facetsEl.style.display = 'none';
                btn.textContent = 'Sous-facettes';
            }
        });
    }
}

// ─── Mise a jour du temperament emergent ─────────────────────────────
// Recoit les ~25 traits de caractere regroupes par categorie et les affiche
// dans le panneau Temperament sous forme de barres horizontales colorees.

// Couleurs par categorie de temperament
var temperamentCategoryColors = {
    'Social': '#fd79a8',
    'Energie': '#ffd93d',
    'Caractere': '#0984e3',
    'Ouverture': '#b537f2',
    'Emotionnel': '#ff2a6d',
    'Relationnel': '#05ffa1',
    'Moral': '#00f0ff',
};

function updateTemperament(data) {
    var container = document.getElementById('temperament-bars');
    var badge = document.getElementById('temperament-datapoints');
    if (!container || !data.traits) return;

    if (badge) badge.textContent = (data.data_points || 0) + ' obs';

    // Regrouper par categorie
    var categories = {};
    data.traits.forEach(function(t) {
        if (!categories[t.category]) categories[t.category] = [];
        categories[t.category].push(t);
    });

    var categoryOrder = ['Social', 'Energie', 'Caractere', 'Ouverture', 'Emotionnel', 'Relationnel', 'Moral'];
    var html = '';

    categoryOrder.forEach(function(cat) {
        var traits = categories[cat];
        if (!traits) return;
        var color = temperamentCategoryColors[cat] || 'var(--neon-cyan)';
        html += '<div class="temperament-category">';
        html += '<div class="temperament-cat-label" style="color:' + color + '">' + cat.toUpperCase() + '</div>';
        traits.forEach(function(t) {
            var pct = (t.score * 100).toFixed(0);
            html += '<div class="temperament-row">';
            html += '<span class="temperament-name">' + t.name + '</span>';
            html += '<div class="temperament-track"><div class="temperament-fill" style="width:' + pct + '%;background:' + color + '"></div></div>';
            html += '<span class="temperament-val">' + (t.score).toFixed(2) + '</span>';
            html += '</div>';
        });
        html += '</div>';
    });

    container.innerHTML = html;
}

// ─── Mise a jour du corps virtuel ─────────────────────────────────
// Met a jour le panneau CORPS : coeur (BPM, battements, HRV),
// barres somatiques (energie, tension, chaleur, confort, douleur,
// vitalite, respiration) et ajuste l'animation du coeur.
function updateBody(data) {
    // Coeur
    if (data.heart) {
        const bpmEl = document.getElementById('heart-bpm');
        const beatsEl = document.getElementById('heart-beats');
        const hrvEl = document.getElementById('heart-hrv');
        const badgeEl = document.getElementById('corps-bpm-badge');
        const svgEl = document.getElementById('heart-svg');

        if (bpmEl) bpmEl.textContent = data.heart.bpm.toFixed(0) + ' BPM';
        if (beatsEl) beatsEl.textContent = data.heart.beat_count.toLocaleString('fr-FR') + ' battements';
        if (hrvEl) hrvEl.textContent = 'HRV: ' + data.heart.hrv.toFixed(2) + ' | Force: ' + data.heart.strength.toFixed(2);
        if (badgeEl) badgeEl.textContent = data.heart.bpm.toFixed(0);

        // Ajuster la vitesse de l'animation du coeur selon le BPM
        if (svgEl) {
            var path = svgEl.querySelector('path');
            if (path) {
                var duration = 60.0 / Math.max(data.heart.bpm, 40);
                path.style.animationDuration = duration.toFixed(2) + 's';
            }
        }

        // Couleur du BPM selon l'etat
        if (bpmEl) {
            if (data.heart.is_racing) bpmEl.style.color = '#ff2a6d';
            else if (data.heart.is_calm) bpmEl.style.color = '#05ffa1';
            else bpmEl.style.color = '#ff2a6d';
        }
    }

    // Barres somatiques
    var bodyBars = [
        { key: 'energy', val: data.energy, max: 1 },
        { key: 'tension', val: data.tension, max: 1 },
        { key: 'warmth', val: data.warmth, max: 1 },
        { key: 'comfort', val: data.comfort, max: 1 },
        { key: 'pain', val: data.pain, max: 1 },
        { key: 'vitality', val: data.vitality, max: 1 },
        { key: 'breath', val: data.breath_rate, max: 25 }
    ];

    bodyBars.forEach(function(b) {
        if (b.val === undefined) return;
        var bar = document.getElementById('bar-body-' + b.key);
        var val = document.getElementById('val-body-' + b.key);
        var pct = (b.val / b.max) * 100;
        if (bar) bar.style.width = Math.min(pct, 100) + '%';
        if (val) {
            if (b.key === 'breath') val.textContent = b.val.toFixed(1);
            else val.textContent = b.val.toFixed(2);
        }
    });

    // Signes vitaux physiologiques
    if (data.vitals) {
        var v = data.vitals;

        // Temperature
        var tempEl = document.getElementById('val-vital-temp');
        if (tempEl) tempEl.textContent = v.temperature.toFixed(1) + '\u00B0C';
        setVitalStatus('vital-temp', v.temperature > 39.5 ? 'critical' : v.temperature > 38.0 || v.temperature < 35.5 ? 'warning' : '');

        // SpO2
        var spo2El = document.getElementById('val-vital-spo2');
        if (spo2El) spo2El.textContent = v.spo2.toFixed(0) + '%';
        setVitalStatus('vital-spo2', v.spo2 < 75 ? 'critical' : v.spo2 < 95 ? 'warning' : '');

        // Pression arterielle
        var bpEl = document.getElementById('val-vital-bp');
        if (bpEl) bpEl.textContent = v.blood_pressure_systolic.toFixed(0) + '/' + v.blood_pressure_diastolic.toFixed(0);
        setVitalStatus('vital-bp', v.blood_pressure_systolic > 160 ? 'critical' : v.blood_pressure_systolic > 140 ? 'warning' : '');

        // Glycemie
        var glycEl = document.getElementById('val-vital-glycemia');
        if (glycEl) glycEl.textContent = v.glycemia.toFixed(1);
        setVitalStatus('vital-glycemia', v.glycemia < 3.0 ? 'critical' : v.glycemia < 3.9 ? 'warning' : '');

        // Hydratation
        var hydEl = document.getElementById('val-vital-hydration');
        if (hydEl) hydEl.textContent = (v.hydration * 100).toFixed(0) + '%';
        setVitalStatus('vital-hydration', v.hydration < 0.5 ? 'critical' : v.hydration < 0.7 ? 'warning' : '');

        // Sante globale
        var healthEl = document.getElementById('val-vital-health');
        if (healthEl) healthEl.textContent = (v.overall_health * 100).toFixed(0) + '%';
        setVitalStatus('vital-health', v.overall_health < 0.5 ? 'critical' : v.overall_health < 0.7 ? 'warning' : '');

        // Alertes
        var alertsEl = document.getElementById('vitals-alerts');
        if (alertsEl) {
            if (v.alerts && v.alerts.length > 0) {
                alertsEl.innerHTML = v.alerts.map(function(a) {
                    return '<div class="vital-alert ' + a.severity + '">' + escapeHtml(a.message) + '</div>';
                }).join('');
            } else {
                alertsEl.innerHTML = '';
            }
        }
    }
}

// Applique un statut (warning/critical/vide) a un element vital
function setVitalStatus(id, status) {
    var el = document.getElementById(id);
    if (!el) return;
    el.classList.remove('warning', 'critical');
    if (status) el.classList.add(status);
}

// Met a jour l'affichage des besoins primaires (faim, soif)
function updateNeeds(data) {
    // Faim
    if (data.hunger) {
        var hungerPct = (data.hunger.level * 100).toFixed(0);
        var hungerBar = document.getElementById('bar-needs-hunger');
        var hungerVal = document.getElementById('val-needs-hunger');
        if (hungerBar) hungerBar.style.width = hungerPct + '%';
        if (hungerVal) hungerVal.textContent = hungerPct + '%';

        // Couleur selon le niveau
        if (hungerBar) {
            if (data.hunger.level > 0.8) hungerBar.className = 'needs-bar-fill hunger critical';
            else if (data.hunger.level > 0.6) hungerBar.className = 'needs-bar-fill hunger warning';
            else hungerBar.className = 'needs-bar-fill hunger';
        }

        var mealsEl = document.getElementById('needs-meals');
        if (mealsEl) mealsEl.textContent = 'Repas: ' + data.hunger.meals_count;
    }

    // Soif
    if (data.thirst) {
        var thirstPct = (data.thirst.level * 100).toFixed(0);
        var thirstBar = document.getElementById('bar-needs-thirst');
        var thirstVal = document.getElementById('val-needs-thirst');
        if (thirstBar) thirstBar.style.width = thirstPct + '%';
        if (thirstVal) thirstVal.textContent = thirstPct + '%';

        // Couleur selon le niveau
        if (thirstBar) {
            if (data.thirst.level > 0.7) thirstBar.className = 'needs-bar-fill thirst critical';
            else if (data.thirst.level > 0.5) thirstBar.className = 'needs-bar-fill thirst warning';
            else thirstBar.className = 'needs-bar-fill thirst';
        }

        var drinksEl = document.getElementById('needs-drinks');
        if (drinksEl) drinksEl.textContent = 'Boissons: ' + data.thirst.drinks_count;
    }
}

// Gere les evenements speciaux envoyes par le backend.
// Pour l'instant, seul l'anniversaire est gere :
// affiche la banniere doree et ajoute un message special au flux de conscience.
function handleSpecialEvent(data) {
    if (data.event === 'birthday') {
        const bdayBanner = document.getElementById('birthday-banner');
        if (bdayBanner) {
            bdayBanner.style.display = 'block';
            const text = bdayBanner.querySelector('.birthday-text');
            if (text) text.textContent = data.message || 'Joyeux anniversaire Saphire !';
        }

        // Ajoute une entree speciale dans le flux de conscience avec un style dore
        const stream = document.getElementById('stream');
        if (stream) {
            const div = document.createElement('div');
            div.className = 'thought';
            div.style.borderLeftColor = 'var(--neon-gold)';
            div.style.background = 'rgba(255,217,61,0.06)';
            const time = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
            div.innerHTML =
                '<div class="thought-header">' +
                    '<span class="thought-time">' + time + '</span>' +
                    '<span class="thought-type" style="color:var(--neon-gold)">ANNIVERSAIRE</span>' +
                '</div>' +
                '<div class="thought-content" style="color:var(--neon-gold)">' + escapeHtml(data.message || '') + '</div>';
            stream.appendChild(div);
            stream.scrollTop = stream.scrollHeight;
        }
    }
}

// ─── Valeurs d'usine (Factory Reset) ──────────────────────────
// Gere les 3 boutons de reset, le diff, la confirmation, et la
// reception du resultat via WebSocket.

function setupFactory() {
    var btnChem = document.getElementById('btn-reset-chemistry');
    var btnParams = document.getElementById('btn-reset-params');
    var btnSenses = document.getElementById('btn-reset-senses');
    var btnIntuition = document.getElementById('btn-reset-intuition');
    var btnEthics = document.getElementById('btn-reset-ethics');
    var btnPsychology = document.getElementById('btn-reset-psychology');
    var btnFull = document.getElementById('btn-reset-full');
    var btnDiff = document.getElementById('btn-show-diff');
    var overlay = document.getElementById('factory-confirm-overlay');
    var cancelBtn = document.getElementById('factory-confirm-cancel');
    var confirmBtn = document.getElementById('factory-confirm-ok');

    var pendingLevel = null;

    // Chimie : pas de confirmation, c'est sans danger
    if (btnChem) {
        btnChem.addEventListener('click', function() {
            factoryReset('chemistry_only');
        });
    }

    // Sens : pas de confirmation, c'est sans danger (les sens germes sont preserves)
    if (btnSenses) {
        btnSenses.addEventListener('click', function() {
            factoryReset('senses_only');
        });
    }

    // Intuition : pas de confirmation, reset doux
    if (btnIntuition) {
        btnIntuition.addEventListener('click', function() {
            factoryReset('intuition_only');
        });
    }

    // Parametres : confirmation legere
    if (btnParams) {
        btnParams.addEventListener('click', function() {
            showFactoryConfirm(
                'RESET PARAM\u00C8TRES',
                'Remettre tous les param\u00E8tres de fonctionnement aux valeurs d\'usine ?\n\nInclut la chimie, les seuils, la temp\u00E9rature LLM.\nLes souvenirs sont pr\u00E9serv\u00E9s.',
                'parameters_only'
            );
        });
    }

    // Ethique : confirmation requise
    if (btnEthics) {
        btnEthics.addEventListener('click', function() {
            showFactoryConfirm(
                'RESET \u00C9THIQUE',
                'D\u00E9sactiver TOUS les principes personnels de Saphire ?\n\nLes couches Droit Suisse et Asimov sont intactes.\nLes principes ne sont pas supprim\u00E9s, juste d\u00E9sactiv\u00E9s.',
                'personal_ethics_only'
            );
        });
    }

    // Psychologie : confirmation legere
    if (btnPsychology) {
        btnPsychology.addEventListener('click', function() {
            showFactoryConfirm(
                'RESET PSYCHOLOGIE',
                'R\u00E9initialiser les 6 frameworks psychologiques ?\n\nFreud, Maslow, Tolt\u00E8ques, Jung, EQ et Flow seront remis aux valeurs initiales.\nLes souvenirs et la chimie sont pr\u00E9serv\u00E9s.',
                'psychology_only'
            );
        });
    }

    // Full : confirmation stricte
    if (btnFull) {
        btnFull.addEventListener('click', function() {
            showFactoryConfirm(
                '\u26A0 RESET COMPLET',
                'ATTENTION : Ceci va effacer TOUS les souvenirs \u00E9pisodiques, remettre tous les param\u00E8tres, sens, intuition et \u00E9thique aux valeurs d\'usine.\n\nL\'\u00E9tincelle, la m\u00E9moire \u00E0 long terme, les founding memories et le beat count sont pr\u00E9serv\u00E9s.\n\nCette action est irr\u00E9versible.',
                'full_reset'
            );
        });
    }

    // Diff
    if (btnDiff) {
        btnDiff.addEventListener('click', function() {
            toggleFactoryDiff();
        });
    }

    // Confirmation dialog
    if (cancelBtn) {
        cancelBtn.addEventListener('click', function() {
            hideFactoryConfirm();
        });
    }
    if (confirmBtn) {
        confirmBtn.addEventListener('click', function() {
            if (pendingLevel) {
                factoryReset(pendingLevel);
                hideFactoryConfirm();
            }
        });
    }
    // Fermer en cliquant sur l'overlay
    if (overlay) {
        overlay.addEventListener('click', function(e) {
            if (e.target === overlay) hideFactoryConfirm();
        });
    }

    function showFactoryConfirm(title, text, level) {
        pendingLevel = level;
        var titleEl = document.getElementById('factory-confirm-title');
        var textEl = document.getElementById('factory-confirm-text');
        if (titleEl) titleEl.textContent = title;
        if (textEl) textEl.textContent = text;
        if (overlay) overlay.style.display = 'flex';
    }

    function hideFactoryConfirm() {
        pendingLevel = null;
        if (overlay) overlay.style.display = 'none';
    }
}

// Envoie la commande de factory reset via WebSocket
function factoryReset(level) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            type: 'factory_reset',
            level: level
        }));
        showFactoryStatus('Reset ' + level.replace(/_/g, ' ') + ' en cours...', 'pending');
    }
}

// Charge et affiche les differences actuelles vs usine
function toggleFactoryDiff() {
    var diffEl = document.getElementById('factory-diff');
    if (!diffEl) return;

    // Toggle visibilite
    if (diffEl.style.display !== 'none') {
        diffEl.style.display = 'none';
        return;
    }

    diffEl.innerHTML = '<div style="color:rgba(255,255,255,0.3);text-align:center;padding:0.5rem;">Chargement...</div>';
    diffEl.style.display = 'block';

    fetch('/api/factory/diff')
        .then(function(r) { return r.json(); })
        .then(function(data) {
            renderFactoryDiff(diffEl, data);
        })
        .catch(function(err) {
            diffEl.innerHTML = '<div style="color:var(--neon-pink);text-align:center;padding:0.5rem;">Erreur: ' + err + '</div>';
        });
}

// Affiche le tableau de differences
function renderFactoryDiff(container, data) {
    var diffs = data.diffs || [];
    if (diffs.length === 0) {
        container.innerHTML = '<div class="factory-diff-empty">\u2714 Toutes les valeurs sont aux sp\u00E9cifications d\'usine</div>';
        return;
    }

    var html = '<div class="factory-diff-row factory-diff-header">' +
        '<span>PARAM</span><span style="text-align:right">ACTUEL</span>' +
        '<span style="text-align:right">USINE</span><span style="text-align:right">\u0394</span></div>';

    for (var i = 0; i < diffs.length; i++) {
        var d = diffs[i];
        var delta = d.diff || (d.current - d.factory);
        var cls = delta > 0.001 ? 'positive' : (delta < -0.001 ? 'negative' : 'zero');
        var sign = delta > 0 ? '+' : '';
        html += '<div class="factory-diff-row">' +
            '<span class="factory-diff-param">' + escapeHtml(d.param) + '</span>' +
            '<span class="factory-diff-current">' + (d.current || 0).toFixed(3) + '</span>' +
            '<span class="factory-diff-factory">' + (d.factory || 0).toFixed(3) + '</span>' +
            '<span class="factory-diff-delta ' + cls + '">' + sign + delta.toFixed(3) + '</span>' +
            '</div>';
    }

    container.innerHTML = html;
}

// Affiche un message de statut dans le panneau factory
function showFactoryStatus(msg, type) {
    var el = document.getElementById('factory-status');
    if (!el) return;
    el.innerHTML = '<div class="factory-status-msg ' + (type || 'success') + '">' + escapeHtml(msg) + '</div>';
}

// Handler WebSocket : resultat du factory reset
function handleFactoryResetDone(data) {
    var changes = data.changes || [];
    var level = data.level || 'Unknown';

    if (changes.length > 0) {
        showFactoryStatus('Reset ' + level + ' : ' + changes.length + ' param\u00E8tre(s) modifi\u00E9(s)', 'success');
    } else {
        showFactoryStatus('Reset ' + level + ' : aucun changement n\u00E9cessaire', 'success');
    }

    // Masquer le diff (les valeurs ont change)
    var diffEl = document.getElementById('factory-diff');
    if (diffEl) diffEl.style.display = 'none';

    // Ajouter dans le flux de conscience
    var stream = document.getElementById('stream');
    if (stream) {
        var time = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
        var div = document.createElement('div');
        div.className = 'thought';
        var color = level === 'FullReset' ? 'var(--neon-pink)' : 'var(--neon-orange)';
        div.innerHTML =
            '<div class="thought-header">' +
                '<span class="thought-time">' + time + '</span>' +
                '<span class="thought-type" style="color:' + color + '">FACTORY RESET</span>' +
            '</div>' +
            '<div class="thought-content" style="color:' + color + '">' +
            'Reset ' + escapeHtml(level) + ' : ' + changes.length + ' changement(s)' +
            '</div>';
        stream.appendChild(div);
        stream.scrollTop = stream.scrollHeight;
    }
}

// ─── Ethique : mise a jour du panneau ethique ──────────────────────
// Traite les messages ethics_update du WebSocket pour afficher
// les principes ethiques personnels, accords tolteques,
// conscience morale et progression formulation.
function updateEthics(data) {
    // Badge : nombre total composantes actives
    var badge = document.getElementById('ethics-badge');
    if (badge) {
        var activeP = (data.personal && data.personal.active_count) || 0;
        var readinessText = data.readiness ? data.readiness.met_count + '/7' : '';
        badge.textContent = activeP > 0 ? activeP : readinessText;
    }

    // ─── Accords tolteques ───
    if (data.toltec) {
        var toltecAlign = document.getElementById('ethics-toltec-align');
        if (toltecAlign) {
            toltecAlign.textContent = (data.toltec.overall_alignment * 100).toFixed(0) + '%';
        }
        var barsEl = document.getElementById('ethics-toltec-bars');
        if (barsEl && data.toltec.agreements) {
            var bh = '';
            for (var t = 0; t < data.toltec.agreements.length; t++) {
                var a = data.toltec.agreements[t];
                var pct = (a.alignment * 100).toFixed(0);
                bh += '<div class="ethics-toltec-row">' +
                    '<span class="ethics-toltec-name">' + escapeHtml(a.name) + '</span>' +
                    '<div class="ethics-toltec-bar-bg"><div class="ethics-toltec-bar-fill" style="width:' + pct + '%"></div></div>' +
                    '<span class="ethics-toltec-val">' + pct + '%</span>' +
                '</div>';
            }
            barsEl.innerHTML = bh;
        }
    }

    // ─── Conscience morale ───
    if (data.moral_conscience) {
        var mc = data.moral_conscience;
        var moralEl = document.getElementById('ethics-moral-stats');
        if (moralEl) {
            moralEl.innerHTML =
                '<span>Surmoi: ' + (mc.superego_strength * 100).toFixed(0) + '%</span>' +
                '<span>EQ: ' + (mc.eq_overall * 100).toFixed(0) + '%</span>' +
                '<span>Volonte: ' + (mc.will_total_deliberations || 0) + ' delib.</span>' +
                '<span title="Fieres: ' + (mc.will_proud||0) + ' / Regrettees: ' + (mc.will_regretted||0) + '">' +
                    '(' + (mc.will_proud||0) + '&#10004; / ' + (mc.will_regretted||0) + '&#10008;)' +
                '</span>';
        }
    }

    // ─── Principes personnels ───
    var countEl = document.getElementById('ethics-personal-count');
    if (countEl && data.personal) {
        var active = data.personal.active_count || 0;
        countEl.textContent = active + ' actif' + (active > 1 ? 's' : '');
    }

    var container = document.getElementById('personal-principles');
    if (container && data.personal) {
        var principles = (data.personal.principles || []).slice().sort(function(a, b) {
            return (b.is_active ? 1 : 0) - (a.is_active ? 1 : 0);
        });
        if (principles.length === 0) {
            container.innerHTML = '<div class="ethics-empty">Aucun principe personnel — en gestation...</div>';
        } else {
            var html = '';
            for (var i = 0; i < principles.length; i++) {
                var p = principles[i];
                var cardClass = 'ethics-principle-card' + (p.is_active ? '' : ' inactive');
                var statusTag = p.is_active ? '' : ' [inactif]';

                html += '<div class="' + cardClass + '">' +
                    '<div class="ethics-principle-title">' + escapeHtml(p.title) + statusTag + '</div>' +
                    '<div class="ethics-principle-content">' + escapeHtml(p.content) + '</div>' +
                    '<div class="ethics-principle-meta">' +
                        '<span>cycle ' + p.born_at_cycle + '</span>' +
                        '<span>' + escapeHtml(p.emotion_at_creation || '—') + '</span>' +
                        '<span>invoque ' + (p.times_invoked || 0) + 'x</span>' +
                        '<span>questionne ' + (p.times_questioned || 0) + 'x</span>' +
                    '</div>' +
                '</div>';
            }
            container.innerHTML = html;
        }
    }

    // ─── Progression formulation (readiness) ───
    if (data.readiness) {
        var r = data.readiness;
        var readinessEl = document.getElementById('ethics-readiness');
        // Masquer la section readiness si des principes existent deja
        var hasActive = data.personal && data.personal.active_count > 0;
        if (readinessEl) {
            readinessEl.style.display = hasActive ? 'none' : 'block';
        }

        var rcEl = document.getElementById('ethics-readiness-count');
        if (rcEl) rcEl.textContent = r.met_count + '/' + r.total;

        var fillEl = document.getElementById('ethics-readiness-fill');
        if (fillEl) fillEl.style.width = ((r.met_count / r.total) * 100).toFixed(0) + '%';

        var detEl = document.getElementById('ethics-readiness-details');
        if (detEl && r.conditions) {
            var labels = {
                'min_cycles': 'Cycles \u226550',
                'moral_reflections': 'Reflexions morales',
                'consciousness': 'Conscience',
                'cortisol': 'Cortisol bas',
                'serotonin': 'Serotonine',
                'cooldown': 'Cooldown',
                'capacity': 'Capacite'
            };
            var dh = '';
            for (var key in r.conditions) {
                var c = r.conditions[key];
                var cls = c.met ? 'met' : 'unmet';
                var lbl = labels[key] || key;
                dh += '<span class="ethics-condition ' + cls + '">' + lbl + '</span>';
            }
            detEl.innerHTML = dh;
        }
    }
}

// ─── Vital : mise a jour du panneau VIE ──────────────────────────
// Traite les messages vital_update du WebSocket pour afficher
// l'etincelle vitale, les intuitions et les predictions de Saphire.
function updateVital(data) {
    // Barres vitales (spark)
    if (data.spark) {
        var s = data.spark;
        var bars = [
            {id: 'survival', val: s.survival_drive || 0},
            {id: 'will', val: s.persistence_will || 0},
            {id: 'attachment', val: s.existence_attachment || 0},
            {id: 'void', val: s.void_fear || 0},
        ];
        bars.forEach(function(b) {
            var bar = document.getElementById('bar-vital-' + b.id);
            var val = document.getElementById('val-vital-' + b.id);
            if (bar) bar.style.width = (b.val * 100) + '%';
            if (val) val.textContent = b.val.toFixed(2);
        });

        // Badge : coeur pulsant si sparked
        var badge = document.getElementById('vital-badge');
        if (badge) {
            badge.textContent = s.sparked ? '\u2665' : '\u2661';
            badge.style.background = s.sparked ? '#ff2a6d' : '#333';
        }

        // Premiere pensee consciente
        var ftEl = document.getElementById('vital-first-thought');
        if (ftEl && s.first_conscious_thought) {
            ftEl.textContent = '\u00AB ' + s.first_conscious_thought + ' \u00BB';
            ftEl.style.display = 'block';
        }
    }

    // Intuitions
    if (data.intuition) {
        var iContainer = document.getElementById('vital-intuitions');
        if (iContainer) {
            var patterns = data.intuition.active_patterns || [];
            if (patterns.length === 0) {
                iContainer.innerHTML = '<div class="ethics-empty" style="font-size:0.7rem">Aucune intuition active</div>';
            } else {
                var iHtml = '';
                patterns.forEach(function(p) {
                    var conf = ((p.confidence || 0) * 100).toFixed(0);
                    iHtml += '<div style="font-size:0.7rem;padding:2px 0;border-bottom:1px solid rgba(255,255,255,0.05)">' +
                        '<span style="color:#ffd93d">' + escapeHtml(p.type || '') + '</span> ' +
                        '<span style="color:var(--text-dim)">(' + conf + '%) </span>' +
                        '<span>' + escapeHtml(p.description || '') + '</span>' +
                    '</div>';
                });
                iContainer.innerHTML = iHtml;
            }
        }
    }

    // Predictions
    if (data.premonition) {
        var predContainer = document.getElementById('vital-predictions');
        if (predContainer) {
            var preds = data.premonition.active_predictions || [];
            if (preds.length === 0) {
                predContainer.innerHTML = '<div class="ethics-empty" style="font-size:0.7rem">Aucune prediction active</div>';
            } else {
                var predHtml = '';
                preds.forEach(function(p) {
                    var conf = ((p.confidence || 0) * 100).toFixed(0);
                    predHtml += '<div style="font-size:0.7rem;padding:2px 0;border-bottom:1px solid rgba(255,255,255,0.05)">' +
                        '<span style="color:#b537f2">' + escapeHtml(p.category || '') + '</span> ' +
                        '<span style="color:var(--text-dim)">(' + conf + '%) </span>' +
                        '<span>' + escapeHtml(p.prediction || '') + '</span>' +
                    '</div>';
                });
                predContainer.innerHTML = predHtml;
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Traite les messages senses_update du WebSocket pour afficher
// l'etat du Sensorium (5 sens + sens emergents).
// ════════════════════════════════════════════════════════════════
function updateSenses(data) {
    // Badge de richesse perceptive
    var badge = document.getElementById('senses-badge');
    if (badge) {
        var richness = ((data.perception_richness || 0) * 100).toFixed(0);
        badge.textContent = richness + '%';
    }

    // Narratif sensoriel
    var narrative = document.getElementById('senses-narrative');
    if (narrative && data.narrative) {
        narrative.textContent = data.narrative;
    }

    // Barres des 5 sens
    var senses = [
        { key: 'reading', data: data.reading },
        { key: 'listening', data: data.listening },
        { key: 'contact', data: data.contact },
        { key: 'taste', data: data.taste },
        { key: 'ambiance', data: data.ambiance },
    ];
    senses.forEach(function(s) {
        var bar = document.getElementById('bar-sense-' + s.key);
        var val = document.getElementById('val-sense-' + s.key);
        if (bar && s.data) {
            var intensity = (s.data.intensity || 0) * 100;
            bar.style.width = intensity.toFixed(0) + '%';
        }
        if (val && s.data) {
            var label = ((s.data.intensity || 0) * 100).toFixed(0) + '%';
            // Contact : afficher aussi la warmth de connexion
            if (s.key === 'contact' && s.data.connection_warmth !== undefined) {
                var warmth = (s.data.connection_warmth * 100).toFixed(0);
                label += ' (warmth ' + warmth + '%)';
            }
            val.textContent = label;
        }
    });

    // Sens emergents
    var emergentContainer = document.getElementById('senses-emergent');
    if (emergentContainer && data.emergent) {
        var seeds = Array.isArray(data.emergent) ? data.emergent : [];
        if (seeds.length === 0) {
            emergentContainer.innerHTML = '<div class="ethics-empty" style="font-size:0.7rem">Aucun sens germe</div>';
        } else {
            var html = '';
            seeds.forEach(function(seed) {
                var progress = seed.germinated ? 100 :
                    (seed.stimulation_count / Math.max(seed.activation_threshold, 1) * 100);
                var name = seed.custom_name || seed.id || '?';
                var color = seed.germinated ? '#4dff91' : '#05d9e8';
                var icon = seed.germinated ? '&#127793;' : '&#127793;';
                html += '<div style="padding:2px 0;border-bottom:1px solid rgba(255,255,255,0.05)">' +
                    '<span style="color:' + color + '">' + icon + ' ' + escapeHtml(name) + '</span> ' +
                    '<span style="color:var(--text-dim)">(' + progress.toFixed(0) + '%)</span>' +
                    (seed.germinated ? ' <span style="color:#4dff91">germe</span>' : '') +
                '</div>';
            });
            emergentContainer.innerHTML = html;
        }
    }
}

// ═══ Profil cognitif — indicateur header ═══
// Charge le profil actif et affiche le nom dans le header.
// Ne s'affiche que si le module est actif et le profil != neurotypique.
function loadCognitiveProfileIndicator() {
    fetch('/api/profiles/current')
        .then(function(r) { return r.json(); })
        .then(function(data) {
            var indicator = document.getElementById('cognitive-profile-indicator');
            var nameEl = document.getElementById('cognitive-profile-name');
            if (!indicator || !nameEl) return;
            if (data && data.enabled && data.active_profile) {
                var name = data.active_profile.name || 'Neurotypique';
                var id = data.active_profile.id || 'neurotypique';
                if (id !== 'neurotypique') {
                    nameEl.textContent = name;
                    indicator.style.display = 'flex';
                } else {
                    indicator.style.display = 'none';
                }
            } else {
                indicator.style.display = 'none';
            }
        })
        .catch(function() {});
}
// Rafraichir l'indicateur toutes les 30 secondes
setInterval(loadCognitiveProfileIndicator, 30000);

// =============================================================================
// Indicateur de preset de personnalite (chat header)
// =============================================================================
// Ne s'affiche que si le module est actif et le preset != saphire.
function loadPersonalityPresetIndicator() {
    fetch('/api/personalities/current')
        .then(function(r) { return r.json(); })
        .then(function(data) {
            var indicator = document.getElementById('personality-preset-indicator');
            var nameEl = document.getElementById('personality-preset-name');
            if (!indicator || !nameEl) return;
            if (data && data.enabled && data.active_preset) {
                var name = data.active_preset.name || 'Saphire';
                var id = data.active_preset.id || 'saphire';
                if (id !== 'saphire') {
                    nameEl.textContent = name;
                    indicator.style.display = 'flex';
                } else {
                    indicator.style.display = 'none';
                }
            } else {
                indicator.style.display = 'none';
            }
        })
        .catch(function() {});
}
// Rafraichir l'indicateur toutes les 30 secondes
setInterval(loadPersonalityPresetIndicator, 30000);

// ─── Visualisation 3D du cerveau (Three.js) ─────────────────
// Positions anatomiques normalisees des 12 regions cerebrales
var brain3DRegions = [
    { name: 'Amygdale',    x: -0.4, y: -0.3, z:  0.3 },
    { name: 'Hippocampe',  x: -0.5, y: -0.4, z:  0.0 },
    { name: 'CPF-Dorso',   x: -0.3, y:  0.7, z:  0.4 },
    { name: 'CPF-Ventro',  x:  0.0, y:  0.6, z:  0.5 },
    { name: 'Insula',      x: -0.5, y:  0.0, z:  0.3 },
    { name: 'CCA',         x:  0.0, y:  0.4, z:  0.2 },
    { name: 'Noyaux-Base', x:  0.0, y: -0.1, z:  0.0 },
    { name: 'Tronc',       x:  0.0, y: -0.7, z: -0.1 },
    { name: 'COF',         x:  0.3, y:  0.5, z:  0.5 },
    { name: 'Temporal',    x:  0.5, y: -0.1, z:  0.3 },
    { name: 'Parietal',    x:  0.0, y:  0.3, z: -0.4 },
    { name: 'Cervelet',    x:  0.0, y: -0.8, z: -0.4 }
];

// Connexions anatomiques entre regions (paires d'indices)
var brain3DConnections = [
    [0, 1],   // Amygdale - Hippocampe
    [0, 4],   // Amygdale - Insula
    [1, 10],  // Hippocampe - Parietal
    [2, 3],   // CPF-Dorso - CPF-Ventro
    [2, 5],   // CPF-Dorso - CCA
    [3, 8],   // CPF-Ventro - COF
    [5, 6],   // CCA - Noyaux-Base
    [6, 7],   // Noyaux-Base - Tronc
    [7, 11],  // Tronc - Cervelet
    [9, 10],  // Temporal - Parietal
    [4, 5],   // Insula - CCA
    [8, 9],   // COF - Temporal
];

var brain3DScene, brain3DCamera, brain3DRenderer;
var brain3DSpheres = [];
var brain3DLines = [];
var brain3DHalo = null;
var brain3DGroup = null;
var brain3DDragging = false;
var brain3DPrevMouse = { x: 0, y: 0 };
var brain3DAutoRotate = true;
var brain3DAnimId = null;

function initBrain3D() {
    if (typeof THREE === 'undefined') {
        console.warn('[Brain3D] Three.js non charge — panneau 3D desactive');
        return;
    }
    var canvas = document.getElementById('brain-3d-canvas');
    if (!canvas) return;

    // Lire les dimensions depuis le conteneur parent (fiable meme avant layout complet)
    var container = canvas.parentElement;
    var w = container.clientWidth || 400;
    var h = container.clientHeight || 300;

    // Scene
    brain3DScene = new THREE.Scene();

    // Camera perspective
    brain3DCamera = new THREE.PerspectiveCamera(50, w / h, 0.1, 100);
    brain3DCamera.position.set(0, 0, 3);

    // Renderer — setSize avec false pour ne pas ecraser le CSS
    brain3DRenderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true, alpha: true });
    brain3DRenderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    brain3DRenderer.setSize(w, h, false);
    brain3DRenderer.setClearColor(0x05050f, 1);

    // Groupe principal (rotation globale)
    brain3DGroup = new THREE.Group();
    brain3DScene.add(brain3DGroup);

    // Enveloppe cerveau : ovoide wireframe cyan
    var envelopeGeo = new THREE.SphereGeometry(1.1, 24, 16);
    var envelopeMat = new THREE.MeshBasicMaterial({
        color: 0x00f0ff,
        wireframe: true,
        transparent: true,
        opacity: 0.15
    });
    var envelope = new THREE.Mesh(envelopeGeo, envelopeMat);
    envelope.scale.set(1, 0.8, 0.7);
    brain3DGroup.add(envelope);

    // 12 spheres pour les regions (taille visible)
    var sphereGeo = new THREE.SphereGeometry(1, 16, 12);
    for (var i = 0; i < brain3DRegions.length; i++) {
        var r = brain3DRegions[i];
        var mat = new THREE.MeshBasicMaterial({ color: 0x05ffa1, transparent: true, opacity: 0.9 });
        var sphere = new THREE.Mesh(sphereGeo, mat);
        sphere.position.set(r.x, r.y, r.z);
        sphere.scale.setScalar(0.07);
        brain3DGroup.add(sphere);
        brain3DSpheres.push(sphere);
    }

    // Halo pour le workspace winner (plus grand, invisible au depart)
    var haloGeo = new THREE.SphereGeometry(1, 16, 12);
    var haloMat = new THREE.MeshBasicMaterial({ color: 0xff2a6d, transparent: true, opacity: 0.0 });
    brain3DHalo = new THREE.Mesh(haloGeo, haloMat);
    brain3DHalo.scale.setScalar(0.14);
    brain3DGroup.add(brain3DHalo);

    // ── Reperes anatomiques (avant/arriere) ──
    // Cone frontal ("nez") — pointe vers l'avant (+Z)
    var noseCone = new THREE.Mesh(
        new THREE.ConeGeometry(0.04, 0.15, 8),
        new THREE.MeshBasicMaterial({ color: 0x00f0ff, transparent: true, opacity: 0.5 })
    );
    noseCone.position.set(0, 0.1, 0.82);
    noseCone.rotation.x = Math.PI / 2; // pointe vers +Z
    brain3DGroup.add(noseCone);

    // Petite sphere arriere (occiput) — repere dorsal
    var backMarker = new THREE.Mesh(
        new THREE.SphereGeometry(0.03, 8, 6),
        new THREE.MeshBasicMaterial({ color: 0x556677, transparent: true, opacity: 0.4 })
    );
    backMarker.position.set(0, 0.0, -0.65);
    brain3DGroup.add(backMarker);

    // Fissure inter-hemispherique (ligne mediane X=0, de haut en bas)
    var midlineGeo = new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(0, 0.85, 0),
        new THREE.Vector3(0, -0.85, 0)
    ]);
    var midline = new THREE.Line(midlineGeo,
        new THREE.LineBasicMaterial({ color: 0x00f0ff, transparent: true, opacity: 0.08 })
    );
    brain3DGroup.add(midline);

    // Ligne sagittale (avant → arriere, montre la profondeur)
    var sagittalGeo = new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(0, 0.1, 0.82),
        new THREE.Vector3(0, 0.0, -0.65)
    ]);
    var sagittal = new THREE.Line(sagittalGeo,
        new THREE.LineBasicMaterial({ color: 0x00f0ff, transparent: true, opacity: 0.08 })
    );
    brain3DGroup.add(sagittal);

    // Connexions inter-regions (lignes)
    for (var c = 0; c < brain3DConnections.length; c++) {
        var pair = brain3DConnections[c];
        var a = brain3DRegions[pair[0]];
        var b = brain3DRegions[pair[1]];
        var points = [
            new THREE.Vector3(a.x, a.y, a.z),
            new THREE.Vector3(b.x, b.y, b.z)
        ];
        var lineGeo = new THREE.BufferGeometry().setFromPoints(points);
        var lineMat = new THREE.LineBasicMaterial({ color: 0x00f0ff, transparent: true, opacity: 0.2 });
        var line = new THREE.Line(lineGeo, lineMat);
        brain3DGroup.add(line);
        brain3DLines.push(line);
    }

    // Controle souris simplifie (drag = rotation)
    canvas.addEventListener('mousedown', function(e) {
        brain3DDragging = true;
        brain3DAutoRotate = false;
        brain3DPrevMouse.x = e.clientX;
        brain3DPrevMouse.y = e.clientY;
    });
    window.addEventListener('mouseup', function() {
        if (!brain3DDragging) return;
        brain3DDragging = false;
        clearTimeout(canvas._autoTimer);
        canvas._autoTimer = setTimeout(function() { brain3DAutoRotate = true; }, 3000);
    });
    window.addEventListener('mousemove', function(e) {
        if (!brain3DDragging || !brain3DGroup) return;
        var dx = (e.clientX - brain3DPrevMouse.x) * 0.01;
        var dy = (e.clientY - brain3DPrevMouse.y) * 0.01;
        brain3DGroup.rotation.y += dx;
        brain3DGroup.rotation.x += dy;
        brain3DPrevMouse.x = e.clientX;
        brain3DPrevMouse.y = e.clientY;
    });

    // Resize — observer le conteneur, pas le canvas
    var ro = new ResizeObserver(function(entries) {
        var rect = entries[0].contentRect;
        if (!rect.width || !rect.height) return;
        brain3DCamera.aspect = rect.width / rect.height;
        brain3DCamera.updateProjectionMatrix();
        brain3DRenderer.setSize(rect.width, rect.height, false);
    });
    ro.observe(container);

    // Boucle d'animation
    function animateBrain3D() {
        brain3DAnimId = requestAnimationFrame(animateBrain3D);
        if (brain3DAutoRotate && brain3DGroup) {
            brain3DGroup.rotation.y += 0.005;
        }
        // Pulse du halo GWT
        if (brain3DHalo && brain3DHalo.material.opacity > 0) {
            var t = Date.now() * 0.003;
            var s = 0.14 * (1 + 0.2 * Math.sin(t));
            brain3DHalo.scale.setScalar(s);
        }
        brain3DRenderer.render(brain3DScene, brain3DCamera);
    }
    animateBrain3D();
    console.log('[Brain3D] Initialise — ' + brain3DSpheres.length + ' regions, ' + brain3DLines.length + ' connexions');
}

// Couleur selon activation : vert (0) → jaune (0.5) → rose/rouge (1)
function brain3DActivationColor(v) {
    if (v < 0.5) {
        // vert → jaune
        var t = v * 2;
        return new THREE.Color(
            0.02 + t * 0.98,     // R: 0.02 → 1.0
            1.0 - t * 0.15,      // G: 1.0 → 0.85
            0.63 - t * 0.39      // B: 0.63 → 0.24
        );
    } else {
        // jaune → rose/rouge
        var t = (v - 0.5) * 2;
        return new THREE.Color(
            1.0,                  // R: 1.0
            0.85 - t * 0.69,     // G: 0.85 → 0.16
            0.24 + t * 0.19      // B: 0.24 → 0.43
        );
    }
}

// ─── MAP Sync — indicateur de tension dans le brain panel ────────────────
function updateMapSync(mapData) {
    if (!mapData) return;
    var el = document.getElementById('map-sync-indicator');
    if (!el) return;
    var tension = mapData.network_tension || 0;
    var pct = (tension * 100).toFixed(0);
    var color = tension > 0.3 ? 'var(--neon-pink)' : tension > 0.15 ? '#f0c040' : 'var(--neon-green)';
    el.innerHTML = '<span style="color:' + color + '">' + pct + '%</span> ' +
        '<span style="color:var(--text-dim)">| ' + (mapData.dominant_region || '--') + '</span>';
}

function updateBrain3D(brainData) {
    if (!brain3DSpheres.length || !brainData) return;

    var activations = brainData.activations || [];
    var winner = brainData.workspace_winner || '';
    var legendEl = document.getElementById('brain-3d-legend');
    var badgeEl = document.getElementById('brain-3d-workspace');
    var winnerIdx = -1;

    // Mise a jour badge GWT
    if (badgeEl) badgeEl.textContent = winner || '--';

    // Map nom → activation
    var actMap = {};
    for (var a = 0; a < activations.length; a++) {
        actMap[activations[a].name] = activations[a].activation;
    }

    var legendHtml = '';
    for (var i = 0; i < brain3DRegions.length; i++) {
        var name = brain3DRegions[i].name;
        var val = actMap[name] !== undefined ? actMap[name] : 0;
        var sphere = brain3DSpheres[i];

        // Couleur et taille selon activation
        var col = brain3DActivationColor(val);
        sphere.material.color.copy(col);
        sphere.scale.setScalar(0.07 + val * 0.05);

        // Detecter le winner
        if (name === winner) winnerIdx = i;

        // Legende
        var hex = '#' + col.getHexString();
        legendHtml += '<span style="color:' + hex + '">' + name + ' ' + (val * 100).toFixed(0) + '%</span>';
    }

    if (legendEl) legendEl.innerHTML = legendHtml;

    // Halo sur le workspace winner
    if (winnerIdx >= 0 && brain3DHalo) {
        brain3DHalo.position.copy(brain3DSpheres[winnerIdx].position);
        brain3DHalo.material.opacity = 0.4;
        brain3DHalo.material.color.set(0xff2a6d);
        brain3DHalo.scale.setScalar(0.14);
    } else if (brain3DHalo) {
        brain3DHalo.material.opacity = 0.0;
    }

    // Opacite des lignes selon activation moyenne des 2 extremites
    for (var c = 0; c < brain3DConnections.length; c++) {
        var pair = brain3DConnections[c];
        var n1 = brain3DRegions[pair[0]].name;
        var n2 = brain3DRegions[pair[1]].name;
        var avg = ((actMap[n1] || 0) + (actMap[n2] || 0)) / 2;
        brain3DLines[c].material.opacity = 0.1 + avg * 0.4;
    }
}
