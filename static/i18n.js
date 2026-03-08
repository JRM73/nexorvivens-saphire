// =============================================================================
// i18n.js — Module d'internationalisation de Saphire
//
// Role : Charge les fichiers de traduction JSON et applique la langue
//        selectionnee a tous les elements du DOM marques avec data-i18n.
//        La langue est persistee dans localStorage.
//
// Langues : fr (reference), en, de, es, it, pt, ru, zh, ja, ar
// =============================================================================

const I18n = {
    current: 'en',
    translations: {},
    available: ['fr', 'en', 'de', 'es', 'it', 'pt', 'ru', 'zh', 'ja', 'ar'],
    rtlLanguages: ['ar'],

    // Charge un fichier de traduction et applique
    async load(lang) {
        if (!this.available.includes(lang)) lang = 'en';
        try {
            const resp = await fetch('/i18n/' + lang + '.json');
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            this.translations = await resp.json();
            this.current = lang;
            this.apply();
            localStorage.setItem('saphire-lang', lang);
            document.documentElement.lang = lang;
            document.documentElement.dir = this.rtlLanguages.includes(lang) ? 'rtl' : 'ltr';
            // Mettre a jour le titre de la page
            if (this.translations['app.title']) {
                document.title = this.translations['app.title'];
            }
        } catch (e) {
            console.warn('[i18n] Loading error ' + lang + ':', e);
            // Fallback to English if not already the current language
            if (lang !== 'en') this.load('en');
        }
    },

    // Retourne la traduction pour une cle donnee
    t(key, fallback) {
        return this.translations[key] || fallback || key;
    },

    // Applique les traductions a tous les elements du DOM
    apply() {
        // Elements avec data-i18n : remplace le contenu texte
        document.querySelectorAll('[data-i18n]').forEach(function(el) {
            var key = el.getAttribute('data-i18n');
            var text = I18n.translations[key];
            if (text) el.textContent = text;
        });

        // Elements avec data-i18n-placeholder : remplace le placeholder
        document.querySelectorAll('[data-i18n-placeholder]').forEach(function(el) {
            var key = el.getAttribute('data-i18n-placeholder');
            var text = I18n.translations[key];
            if (text) el.placeholder = text;
        });

        // Elements avec data-i18n-title : remplace le title (tooltip)
        document.querySelectorAll('[data-i18n-title]').forEach(function(el) {
            var key = el.getAttribute('data-i18n-title');
            var text = I18n.translations[key];
            if (text) el.title = text;
        });

        // Mettre a jour le selecteur de langue si present
        var sel = document.getElementById('lang-selector');
        if (sel) sel.value = this.current;
    },

    // Initialise le systeme i18n au chargement de la page
    init() {
        var saved = localStorage.getItem('saphire-lang');
        var lang = saved || 'en';
        this.load(lang);
    }
};
