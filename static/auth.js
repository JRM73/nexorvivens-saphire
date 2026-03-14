/*
 * ============================================================
 * SAPHIRE — auth.js
 * ============================================================
 * Module d'authentification pour l'interface web.
 * Intercepte les appels fetch() pour ajouter le header
 * Authorization: Bearer, et fournit un helper pour les URLs
 * WebSocket avec token.
 *
 * Doit etre charge AVANT app.js, dashboard.html et brain-map.html.
 * ============================================================
 */
(function() {
    'use strict';

    const STORAGE_KEY = 'saphire_api_key';

    function getApiKey() {
        return localStorage.getItem(STORAGE_KEY);
    }

    function setApiKey(key) {
        localStorage.setItem(STORAGE_KEY, key);
    }

    // ─── Override fetch() pour ajouter le Bearer token ──────────
    const originalFetch = window.fetch;
    window.fetch = function(url, options) {
        options = options || {};
        const key = getApiKey();
        if (key && typeof url === 'string' && url.startsWith('/api/')) {
            if (!options.headers) {
                options.headers = {};
            }
            if (options.headers instanceof Headers) {
                if (!options.headers.has('Authorization')) {
                    options.headers.set('Authorization', 'Bearer ' + key);
                }
            } else if (Array.isArray(options.headers)) {
                if (!options.headers.some(h => h[0].toLowerCase() === 'authorization')) {
                    options.headers.push(['Authorization', 'Bearer ' + key]);
                }
            } else {
                if (!options.headers['Authorization']) {
                    options.headers['Authorization'] = 'Bearer ' + key;
                }
            }
        }
        return originalFetch.call(this, url, options);
    };

    // ─── Helper pour URL WebSocket avec token ───────────────────
    window.saphireWsUrl = function(path) {
        var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        var key = getApiKey();
        var tokenParam = key ? '?token=' + encodeURIComponent(key) : '';
        return protocol + '//' + window.location.host + path + tokenParam;
    };

    // ─── Modal de connexion ─────────────────────────────────────
    function showLoginModal() {
        var overlay = document.createElement('div');
        overlay.id = 'auth-overlay';
        overlay.style.cssText = 'position:fixed;top:0;left:0;width:100%;height:100%;' +
            'background:rgba(5,5,15,0.95);display:flex;align-items:center;' +
            'justify-content:center;z-index:99999;font-family:"Share Tech Mono",monospace;';

        var modal = document.createElement('div');
        modal.style.cssText = 'background:#0f0f2a;border:1px solid #00f0ff;' +
            'border-radius:8px;padding:2.5rem;max-width:380px;width:90%;text-align:center;' +
            'box-shadow:0 0 40px rgba(0,240,255,0.15);';

        var title = document.createElement('h2');
        title.textContent = 'SAPHIRE';
        title.style.cssText = 'color:#00f0ff;margin:0 0 0.5rem;font-family:Orbitron,monospace;' +
            'font-size:1.4rem;letter-spacing:3px;';

        var sub = document.createElement('p');
        sub.textContent = 'Authentification requise';
        sub.style.cssText = 'color:#6070a0;margin:0 0 1.5rem;font-size:0.85rem;';

        var input = document.createElement('input');
        input.type = 'password';
        input.placeholder = 'Clé API';
        input.autocomplete = 'off';
        input.style.cssText = 'width:100%;padding:0.75rem;background:#0a0a1a;' +
            'border:1px solid #334;color:#eee;border-radius:4px;font-family:inherit;' +
            'font-size:0.9rem;box-sizing:border-box;outline:none;';

        var btn = document.createElement('button');
        btn.textContent = 'CONNEXION';
        btn.style.cssText = 'width:100%;margin-top:1rem;padding:0.75rem;' +
            'background:linear-gradient(135deg,#00f0ff,#b537f2);color:#0a0a1a;' +
            'border:none;border-radius:4px;cursor:pointer;font-family:Orbitron,monospace;' +
            'font-size:0.8rem;font-weight:700;letter-spacing:2px;';

        var error = document.createElement('p');
        error.textContent = 'Clé invalide';
        error.style.cssText = 'color:#ff2a6d;margin:0.8rem 0 0;font-size:0.8rem;display:none;';

        modal.appendChild(title);
        modal.appendChild(sub);
        modal.appendChild(input);
        modal.appendChild(btn);
        modal.appendChild(error);
        overlay.appendChild(modal);
        document.body.appendChild(overlay);

        function tryLogin() {
            var key = input.value.trim();
            if (!key) return;
            btn.textContent = '...';
            btn.disabled = true;
            // Tester la clé sur un endpoint protégé
            originalFetch('/api/chemistry', {
                headers: { 'Authorization': 'Bearer ' + key }
            }).then(function(resp) {
                if (resp.ok) {
                    setApiKey(key);
                    overlay.remove();
                    window.location.reload();
                } else {
                    error.style.display = 'block';
                    btn.textContent = 'CONNEXION';
                    btn.disabled = false;
                    input.focus();
                }
            }).catch(function() {
                error.textContent = 'Connexion impossible';
                error.style.display = 'block';
                btn.textContent = 'CONNEXION';
                btn.disabled = false;
            });
        }

        btn.addEventListener('click', tryLogin);
        input.addEventListener('keydown', function(e) {
            if (e.key === 'Enter') tryLogin();
        });
        input.addEventListener('focus', function() {
            input.style.borderColor = '#00f0ff';
        });
        input.addEventListener('blur', function() {
            input.style.borderColor = '#334';
        });
        setTimeout(function() { input.focus(); }, 100);
    }

    // ─── Vérification au chargement ─────────────────────────────
    document.addEventListener('DOMContentLoaded', function() {
        if (!getApiKey()) {
            showLoginModal();
        }
    });
})();
