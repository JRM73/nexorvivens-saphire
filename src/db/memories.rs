// =============================================================================
// db/memories.rs — Memoires LTM, episodiques et fondatrices
// =============================================================================

use super::{SaphireDb, DbError, MemoryRecord, NewMemory};
use chrono::{DateTime, Utc};

impl SaphireDb {
    // ---------------------------------------------------------
    // MEMOIRES (tier 3 : memoire a long terme vectorielle)
    // ---------------------------------------------------------

    /// Stocke un souvenir avec son embedding vectoriel dans la table memories.
    /// L'embedding permet ensuite la recherche par similarite cosinus via pgvector.
    ///
    /// # Parametres
    /// - `memory` : le nouveau souvenir a inserer (avec embedding + metadonnees)
    ///
    /// # Retour
    /// L'identifiant (id) du souvenir insere
    pub async fn store_memory(&self, memory: &NewMemory) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        // Convertir le Vec<f32> en type pgvector::Vector compatible avec PostgreSQL
        let embedding_vec = pgvector::Vector::from(memory.embedding.clone());
        let sig_json: Option<serde_json::Value> = memory.chemical_signature
            .as_ref()
            .and_then(|s| serde_json::to_value(s).ok());
        let row = client.query_one(
            "INSERT INTO memories (embedding, text_summary, stimulus_json, decision,
                    chemistry_json, emotion, mood_valence, satisfaction, emotional_weight,
                    source_episodic_id, chemical_signature)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id",
            &[&embedding_vec, &memory.text_summary, &memory.stimulus_json,
              &memory.decision, &memory.chemistry_json, &memory.emotion,
              &memory.mood_valence, &memory.satisfaction, &memory.emotional_weight,
              &memory.source_episodic_id, &sig_json],
        ).await?;
        Ok(row.get(0))
    }

    /// Recherche vectorielle des souvenirs similaires via pgvector.
    /// Utilise l'operateur <=> (distance cosinus) pour trouver les souvenirs
    /// les plus proches du vecteur donne.
    ///
    /// # Parametres
    /// - `embedding` : vecteur de requete (representation du contexte actuel)
    /// - `limit` : nombre maximal de resultats a retourner
    /// - `threshold` : similarite minimale (0.0 a 1.0) pour filtrer les resultats
    ///
    /// # Retour
    /// Liste des souvenirs similaires, tries par proximite decroissante
    pub async fn search_similar_memories(
        &self,
        embedding: &[f32],
        limit: i64,
        threshold: f64,
    ) -> Result<Vec<MemoryRecord>, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let threshold_f32 = threshold as f32;
        // L'operateur <=> calcule la distance cosinus entre deux vecteurs.
        // 1 - distance = similarite. On filtre par seuil de similarite.
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    1 - (embedding <=> $1) as similarity, chemical_signature
             FROM memories
             WHERE 1 - (embedding <=> $1) > $3::real
             ORDER BY embedding <=> $1
             LIMIT $2",
            &[&embedding_vec, &limit, &threshold_f32],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(11).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: {
                    let sim: Option<f64> = row.get(10);
                    sim.unwrap_or(0.0)
                },
                chemical_signature,
            });
        }
        Ok(memories)
    }

    /// Compte le nombre total de souvenirs stockes dans la table memories.
    ///
    /// # Retour
    /// Le nombre total de souvenirs
    pub async fn memory_count(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memories", &[]).await?;
        Ok(row.get(0))
    }

    /// Recupere les N souvenirs les plus recents (sans filtrage par similarite).
    /// Le champ `similarity` est mis a 0.0 car ce n'est pas une recherche vectorielle.
    ///
    /// # Parametres
    /// - `n` : nombre de souvenirs a recuperer
    ///
    /// # Retour
    /// Liste des souvenirs recents, tries par date decroissante
    pub async fn recent_memories(&self, n: i64) -> Result<Vec<MemoryRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    chemical_signature
             FROM memories ORDER BY created_at DESC LIMIT $1",
            &[&n],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(10).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: 0.0,
                chemical_signature,
            });
        }
        Ok(memories)
    }

    /// Compte le nombre total de souvenirs LTM.
    pub async fn count_ltm(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memories", &[]).await?;
        Ok(row.get(0))
    }

    /// Recupere les N souvenirs LTM les plus faibles et non proteges.
    /// Un souvenir est protege si access_count >= min_access OU emotional_weight >= min_weight.
    /// Retourne les souvenirs tries par emotional_weight ASC, access_count ASC, created_at ASC.
    pub async fn fetch_ltm_weakest_unprotected(
        &self,
        count: i64,
        min_access: i32,
        min_weight: f32,
    ) -> Result<Vec<MemoryRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    0.0::float8 as similarity, chemical_signature
             FROM memories
             WHERE access_count < $2 AND emotional_weight < $3
             ORDER BY emotional_weight ASC, access_count ASC, created_at ASC
             LIMIT $1",
            &[&count, &min_access, &min_weight],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(11).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: row.get::<_, f64>(10),
                chemical_signature,
            });
        }
        Ok(memories)
    }

    /// Renforce un souvenir LTM rappele (boost d'acces).
    /// Incremente access_count et met a jour last_accessed_at.
    /// Les souvenirs frequemment rappeles deviennent proteges (access_count >= 5).
    pub async fn boost_memory_access(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE memories
             SET access_count = access_count + 1
             WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Supprime les souvenirs LTM par IDs (pour l'elagage avec archivage).
    pub async fn delete_memories_by_ids(&self, ids: &[i64]) -> Result<u64, DbError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let client = self.pool.get().await?;
        let affected = client.execute(
            "DELETE FROM memories WHERE id = ANY($1)",
            &[&ids],
        ).await?;
        Ok(affected)
    }

    // ---------------------------------------------------------
    // SOUVENIRS FONDATEURS
    // ---------------------------------------------------------

    /// Stocke un souvenir fondateur (jamais supprime, jamais oublie).
    /// Les souvenirs fondateurs sont les premiers moments de l'agent
    /// (genese, premier contact, premieres pensees) et forment le noyau
    /// permanent de sa memoire.
    ///
    /// # Parametres
    /// - `event_type` : type d'evenement (ex: "genesis", "first_contact")
    /// - `content` : contenu textuel du souvenir
    /// - `llm_response` : reponse du LLM lors de cet evenement
    /// - `chemistry_json` : etat neurochimique en JSON
    /// - `consciousness_level` : niveau de conscience au moment de l'evenement
    ///
    /// # Retour
    /// L'identifiant du souvenir fondateur insere
    pub async fn store_founding_memory(
        &self,
        event_type: &str,
        content: &str,
        llm_response: &str,
        chemistry_json: &serde_json::Value,
        consciousness_level: f32,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        // Verifier si un founding memory avec ce event_type existe deja
        // pour eviter les doublons (ex: genesis repete a chaque boot)
        let existing = client.query_opt(
            "SELECT id FROM founding_memories WHERE event_type = $1",
            &[&event_type],
        ).await?;
        if let Some(row) = existing {
            tracing::debug!("Founding memory '{}' existe deja, skip", event_type);
            return Ok(row.get(0));
        }
        let row = client.query_one(
            "INSERT INTO founding_memories (event_type, content, llm_response, chemistry_json, consciousness_level)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&event_type, &content, &llm_response, chemistry_json, &consciousness_level],
        ).await?;
        tracing::info!("Founding memory '{}' cree (id={})", event_type, row.get::<_, i64>(0));
        Ok(row.get(0))
    }

    /// Compte le nombre total de souvenirs fondateurs.
    pub async fn count_founding_memories(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM founding_memories", &[]).await?;
        Ok(row.get(0))
    }

    // ---------------------------------------------------------
    // MEMOIRE EPISODIQUE (tier 2)
    // La memoire episodique est le niveau intermediaire entre la memoire
    // immediate (RAM) et la memoire a long terme (vectorielle).
    // Les souvenirs episodiques ont une "force" qui decroit avec le temps
    // sauf s'ils sont renforces par des rappels ou consolides.
    // ---------------------------------------------------------

    /// Stocke un souvenir episodique.
    /// La force initiale est 1.0 (maximale) et decroit avec le temps.
    ///
    /// # Parametres
    /// - `content` : contenu textuel du souvenir
    /// - `source_type` : type de source (ex: "thought", "conversation", "learning")
    /// - `stimulus_json` : donnees du stimulus en JSON
    /// - `decision` : decision prise (-1, 0, 1)
    /// - `chemistry_json` : etat neurochimique en JSON
    /// - `emotion` : emotion dominante
    /// - `satisfaction` : niveau de satisfaction
    /// - `emotional_intensity` : intensite emotionnelle (protege contre l'oubli)
    /// - `conversation_id` : identifiant optionnel de la conversation d'origine
    ///
    /// # Retour
    /// L'identifiant du souvenir episodique insere
    #[allow(clippy::too_many_arguments)]
    pub async fn store_episodic(
        &self,
        content: &str,
        source_type: &str,
        stimulus_json: &serde_json::Value,
        decision: i16,
        chemistry_json: &serde_json::Value,
        emotion: &str,
        satisfaction: f32,
        emotional_intensity: f32,
        conversation_id: Option<&str>,
        chemical_signature: Option<&crate::neurochemistry::ChemicalSignature>,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let sig_json: Option<serde_json::Value> = chemical_signature
            .map(|s| serde_json::to_value(s).unwrap_or_default());
        let row = client.query_one(
            "INSERT INTO episodic_memories
             (content, source_type, stimulus_json, decision, chemistry_json,
              emotion, satisfaction, emotional_intensity, strength, conversation_id,
              chemical_signature)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1.0, $9, $10)
             RETURNING id",
            &[&content, &source_type, stimulus_json, &decision, chemistry_json,
              &emotion, &satisfaction, &emotional_intensity, &conversation_id,
              &sig_json],
        ).await?;
        Ok(row.get(0))
    }

    /// Recupere les N souvenirs episodiques les plus recents (force > 0.1).
    /// Les souvenirs trop faibles (presque oublies) sont exclus.
    ///
    /// # Parametres
    /// - `limit` : nombre maximal de resultats
    pub async fn recent_episodic(&self, limit: i64) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE strength > 0.1
             ORDER BY created_at DESC
             LIMIT $1",
            &[&limit],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Recupere les souvenirs episodiques filtres par emotion dominante.
    /// Utile pour la reflexion memorielle ou l'analyse emotionnelle.
    ///
    /// # Parametres
    /// - `emotion` : emotion a rechercher (ex: "Curiosite", "Joie")
    /// - `limit` : nombre maximal de resultats
    pub async fn episodic_by_emotion(&self, emotion: &str, limit: i64) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE emotion = $1 AND strength > 0.1
             ORDER BY strength DESC
             LIMIT $2",
            &[&emotion, &limit],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Recupere les souvenirs episodiques d'une conversation specifique.
    /// Permet de retrouver le contexte d'une discussion passee.
    ///
    /// # Parametres
    /// - `conversation_id` : identifiant de la conversation
    /// - `limit` : nombre maximal de resultats
    pub async fn episodic_by_conversation(&self, conversation_id: &str, limit: i64) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE conversation_id = $1 AND strength > 0.1
             ORDER BY created_at DESC
             LIMIT $2",
            &[&conversation_id, &limit],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Applique la decroissance naturelle aux souvenirs episodiques non consolides.
    /// La vitesse d'oubli est moderee par l'intensite emotionnelle (les souvenirs
    /// forts sont oublies plus lentement) et le nombre d'acces (les souvenirs
    /// souvent rappeles resistent mieux a l'oubli).
    /// Les souvenirs dont la force tombe a zero sont supprimes.
    ///
    /// # Parametres
    /// - `rate` : taux de decroissance (plus la valeur est elevee, plus l'oubli est rapide)
    ///
    /// # Retour
    /// Nombre total de lignes affectees (decroissance + suppression)
    pub async fn decay_episodic(&self, rate: f64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        // Formule de decroissance :
        //   force -= taux * (1 / (1 + intensite_emotionnelle)) * (1 / (1 + acces * 0.1))
        // Pourquoi : les souvenirs emotionnels et frequemment rappeles resistent a l'oubli.
        // Note : on caste $1 en real pour matcher le type de la colonne strength (real/f32).
        let affected = client.execute(
            "UPDATE episodic_memories
             SET strength = GREATEST(0.0::real,
                strength - ($1::double precision * (1.0 / (1.0 + emotional_intensity::double precision))
                             * (1.0 / (1.0 + access_count::double precision * 0.1)))::real
             )
             WHERE consolidated = FALSE AND strength > 0.0",
            &[&rate],
        ).await?;

        // Supprimer les souvenirs completement effaces (force = 0)
        let deleted = client.execute(
            "DELETE FROM episodic_memories
             WHERE strength <= 0.0 AND consolidated = FALSE",
            &[],
        ).await?;

        Ok(affected + deleted)
    }

    /// Renforce un souvenir episodique quand il est rappele (acces en lecture).
    /// Augmente la force de 0.2 (plafonnee a 1.0) et incremente le compteur d'acces.
    /// C'est un mecanisme de "reconsolidation" : plus on se souvient, plus on retient.
    ///
    /// # Parametres
    /// - `id` : identifiant du souvenir a renforcer
    pub async fn reinforce_episodic(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE episodic_memories
             SET strength = LEAST(1.0, strength + 0.2),
                 access_count = access_count + 1,
                 last_accessed_at = NOW()
             WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Marque un souvenir episodique comme consolide.
    /// Un souvenir consolide n'est plus soumis a la decroissance naturelle
    /// et ne sera jamais supprime automatiquement (il passe en memoire a long terme).
    ///
    /// # Parametres
    /// - `id` : identifiant du souvenir a consolider
    pub async fn mark_episodic_consolidated(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE episodic_memories SET consolidated = TRUE WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Trouve les souvenirs episodiques candidats a la consolidation.
    /// Un candidat est un souvenir non consolide avec une force > 0.3,
    /// trie par intensite emotionnelle et force decroissante (les plus
    /// marquants sont consolides en premier).
    ///
    /// # Retour
    /// Liste des candidats (maximum 50)
    pub async fn episodic_consolidation_candidates(&self) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE consolidated = FALSE AND strength > 0.3
             ORDER BY emotional_intensity DESC, strength DESC
             LIMIT 50",
            &[],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Compte le nombre de souvenirs episodiques actifs (non consolides).
    /// Les souvenirs consolides sont deja en LTM et ne comptent pas dans le quota.
    pub async fn count_episodic(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM episodic_memories WHERE consolidated = FALSE",
            &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Supprime les souvenirs episodiques deja consolides en LTM.
    /// Une fois transferes en memoire a long terme, les souvenirs episodiques
    /// n'ont plus besoin de rester dans la table episodique.
    pub async fn cleanup_consolidated_episodic(&self) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute(
            "DELETE FROM episodic_memories WHERE consolidated = TRUE",
            &[],
        ).await?;
        Ok(affected)
    }

    /// Supprime les N souvenirs episodiques les plus faibles (non consolides).
    /// Utilise pour le nettoyage quand la memoire est pleine.
    /// Les souvenirs sont tries par force croissante puis par date croissante
    /// (les plus faibles et les plus anciens sont supprimes en premier).
    ///
    /// # Parametres
    /// - `count` : nombre de souvenirs a supprimer
    ///
    /// # Retour
    /// Nombre de souvenirs effectivement supprimes
    pub async fn prune_episodic(&self, count: i64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute(
            "DELETE FROM episodic_memories
             WHERE id IN (
                 SELECT id FROM episodic_memories
                 WHERE consolidated = FALSE
                 ORDER BY strength ASC, created_at ASC
                 LIMIT $1
             )",
            &[&count],
        ).await?;
        Ok(affected)
    }

    /// Efface TOUS les souvenirs episodiques (pour factory reset complet).
    /// Les souvenirs deja consolides en LTM sont preserves dans la table memories.
    pub async fn clear_episodic_memories(&self) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute("DELETE FROM episodic_memories", &[]).await?;
        Ok(affected)
    }

    /// Convertit une ligne (row) PostgreSQL en structure EpisodicRecord.
    /// Fonction utilitaire interne utilisee par les methodes de lecture
    /// des souvenirs episodiques.
    ///
    /// # Parametres
    /// - `row` : ligne PostgreSQL contenant les 15 colonnes attendues
    ///
    /// # Retour
    /// La structure EpisodicRecord remplie
    fn row_to_episodic(row: &tokio_postgres::Row) -> crate::memory::EpisodicRecord {
        // Deserialiser la signature chimique depuis JSONB (None pour les anciens souvenirs)
        let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
            row.try_get::<_, Option<serde_json::Value>>(15).ok()
                .flatten()
                .and_then(|v| serde_json::from_value(v).ok());
        crate::memory::EpisodicRecord {
            id: row.get(0),
            content: row.get(1),
            source_type: row.get(2),
            stimulus_json: row.get(3),
            decision: row.get(4),
            chemistry_json: row.get(5),
            emotion: row.get(6),
            satisfaction: row.get(7),
            emotional_intensity: row.get(8),
            strength: row.get(9),
            access_count: row.get(10),
            last_accessed_at: row.get(11),
            consolidated: row.get(12),
            conversation_id: row.get(13),
            created_at: row.get(14),
            chemical_signature,
        }
    }

    // ---------------------------------------------------------
    // METHODES ADDITIONNELLES POUR LE DASHBOARD
    // ---------------------------------------------------------

    /// Liste les souvenirs episodiques avec pagination.
    pub async fn list_episodic(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, emotion, satisfaction, strength,
                    access_count, consolidated, created_at
             FROM episodic_memories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: DateTime<Utc> = row.get(8);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "content": row.get::<_, String>(1),
                "source_type": row.get::<_, String>(2),
                "emotion": row.get::<_, String>(3),
                "satisfaction": row.get::<_, f32>(4),
                "strength": row.get::<_, f32>(5),
                "access_count": row.get::<_, i32>(6),
                "consolidated": row.get::<_, bool>(7),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Recupere un souvenir episodique par ID.
    pub async fn get_episodic_by_id(&self, id: i64) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT row_to_json(episodic_memories) FROM episodic_memories WHERE id = $1",
            &[&id],
        ).await?;
        match result {
            Some(row) => Ok(Some(row.get(0))),
            None => Ok(None),
        }
    }

    /// Liste les souvenirs LTM avec pagination.
    pub async fn list_memories(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, emotion, satisfaction, emotional_weight,
                    access_count, created_at
             FROM memories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: DateTime<Utc> = row.get(6);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "text_summary": row.get::<_, String>(1),
                "emotion": row.get::<_, String>(2),
                "satisfaction": row.get::<_, f32>(3),
                "emotional_weight": row.get::<_, f32>(4),
                "access_count": row.get::<_, i32>(5),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Recupere un souvenir LTM par ID.
    pub async fn get_memory_by_id(&self, id: i64) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight,
                    access_count, created_at
             FROM memories WHERE id = $1",
            &[&id],
        ).await?;
        match result {
            Some(row) => {
                let ts: DateTime<Utc> = row.get(10);
                Ok(Some(serde_json::json!({
                    "id": row.get::<_, i64>(0),
                    "text_summary": row.get::<_, String>(1),
                    "stimulus_json": row.get::<_, serde_json::Value>(2),
                    "decision": row.get::<_, i16>(3),
                    "chemistry_json": row.get::<_, serde_json::Value>(4),
                    "emotion": row.get::<_, String>(5),
                    "mood_valence": row.get::<_, f32>(6),
                    "satisfaction": row.get::<_, f32>(7),
                    "emotional_weight": row.get::<_, f32>(8),
                    "access_count": row.get::<_, i32>(9),
                    "created_at": ts.to_rfc3339(),
                })))
            }
            None => Ok(None),
        }
    }

    /// Liste les souvenirs fondateurs.
    pub async fn list_founding_memories(&self) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, event_type, content, llm_response, consciousness_level, created_at
             FROM founding_memories ORDER BY created_at ASC",
            &[],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: DateTime<Utc> = row.get(5);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "event_type": row.get::<_, String>(1),
                "content": row.get::<_, String>(2),
                "llm_response": row.get::<_, String>(3),
                "consciousness_level": row.get::<_, f32>(4),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Statistiques de memoire pour le dashboard.
    pub async fn memory_stats(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;
        let ltm_count: i64 = client.query_one("SELECT COUNT(*) FROM memories", &[]).await?.get(0);
        let episodic_count: i64 = client.query_one("SELECT COUNT(*) FROM episodic_memories", &[]).await?.get(0);
        let founding_count: i64 = client.query_one("SELECT COUNT(*) FROM founding_memories", &[]).await?.get(0);
        let thought_count: i64 = client.query_one("SELECT COUNT(*) FROM thought_log", &[]).await?.get(0);
        let knowledge_count: i64 = client.query_one("SELECT COUNT(*) FROM knowledge_log", &[]).await?.get(0);

        // Nombre de souvenirs LTM proteges (access_count >= 5 OU emotional_weight >= 0.7)
        let ltm_protected: i64 = client.query_one(
            "SELECT COUNT(*) FROM memories WHERE access_count >= 5 OR emotional_weight >= 0.7",
            &[],
        ).await.map(|r| r.get(0)).unwrap_or(0);

        // Nombre d'archives memoire
        let archives_count: i64 = client.query_one(
            "SELECT COUNT(*) FROM memory_archives", &[]
        ).await.map(|r| r.get(0)).unwrap_or(0);

        Ok(serde_json::json!({
            "ltm": ltm_count,
            "ltm_protected": ltm_protected,
            "episodic": episodic_count,
            "founding": founding_count,
            "thoughts": thought_count,
            "knowledge": knowledge_count,
            "archives": archives_count,
        }))
    }
}
