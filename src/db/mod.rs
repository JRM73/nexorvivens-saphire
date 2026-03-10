// =============================================================================
// db/mod.rs — SaphireDb : pool de connexions PostgreSQL + pgvector
//
// Role : Ce module gere toute la couche de persistance de Saphire.
// Il fournit un pool de connexions PostgreSQL (via deadpool) et toutes
// les operations CRUD (Create, Read, Update, Delete) pour les differentes
// tables : souvenirs, identite, pensees, sessions, poids neuronaux,
// parametres de tuning, connaissances, profils de personnalite, etc.
//
// Dependances :
//   - deadpool_postgres : pool de connexions asynchrones PostgreSQL
//   - tokio_postgres : client PostgreSQL asynchrone (sans TLS ici)
//   - pgvector : extension PostgreSQL pour la recherche vectorielle
//   - serde / serde_json : serialisation des donnees JSON
//   - chrono : gestion des dates et heures
//
// Place dans l'architecture :
//   SaphireDb est possede par l'agent (SaphireAgent) et utilise pour
//   persister les souvenirs, l'identite, les poids du reseau de neurones,
//   les parametres d'auto-tuning et le journal des pensees.
//   La recherche vectorielle (pgvector) permet de retrouver des souvenirs
//   similaires a un nouvel evenement via la distance cosinus.
// =============================================================================

mod identity;
mod memories;
mod thoughts;
mod knowledge;
mod tuning;
mod profiling;
mod ethics;
mod orchestrators;
pub mod learnings;
mod connections;
pub mod vectors;
pub mod archives;
pub mod lora;

use deadpool_postgres::{Pool, Manager, ManagerConfig, RecyclingMethod};
use tokio_postgres::NoTls;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::neurochemistry::ChemicalSignature;

/// Erreurs de la couche base de donnees.
/// Trois categories d'erreurs sont distinguees pour faciliter le diagnostic.
#[derive(Debug)]
pub enum DbError {
    /// Erreur liee au pool de connexions (pas de connexion disponible, timeout, etc.)
    Pool(String),
    /// Erreur liee a l'execution d'une requete SQL
    Query(String),
    /// Erreur lors de l'execution des migrations de schema
    Migration(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Pool(e) => write!(f, "Pool error: {}", e),
            DbError::Query(e) => write!(f, "Query error: {}", e),
            DbError::Migration(e) => write!(f, "Migration error: {}", e),
        }
    }
}

// Conversion automatique des erreurs du pool deadpool vers DbError
impl From<deadpool_postgres::PoolError> for DbError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        DbError::Pool(e.to_string())
    }
}

// Conversion automatique des erreurs tokio_postgres vers DbError
impl From<tokio_postgres::Error> for DbError {
    fn from(e: tokio_postgres::Error) -> Self {
        DbError::Query(e.to_string())
    }
}

/// Configuration de la connexion a la base de donnees PostgreSQL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// Adresse du serveur PostgreSQL (ex: "localhost", "postgres")
    pub host: String,
    /// Port du serveur PostgreSQL (par defaut : 5432)
    pub port: u16,
    /// Nom d'utilisateur pour la connexion
    pub user: String,
    /// Mot de passe pour la connexion
    pub password: String,
    /// Nom de la base de donnees
    pub dbname: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 5432,
            user: "saphire".into(),
            password: "saphire_soul".into(),
            dbname: "saphire_soul".into(),
        }
    }
}

/// Un souvenir stocke en base de donnees.
/// Represente un evenement vecu par l'agent avec toutes ses metadonnees
/// (emotion, chimie, decision, satisfaction, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// Identifiant unique du souvenir en base
    pub id: i64,
    /// Resume textuel du souvenir (genere par le LLM ou extrait du stimulus)
    pub text_summary: String,
    /// Donnees du stimulus d'origine en JSON (danger, recompense, etc.)
    pub stimulus_json: serde_json::Value,
    /// Decision prise : -1 (Non), 0 (Peut-etre), 1 (Oui)
    pub decision: i16,
    /// Etat neurochimique au moment du souvenir (7 neurotransmetteurs en JSON)
    pub chemistry_json: serde_json::Value,
    /// Emotion dominante au moment du souvenir (ex: "Curiosite", "Peur")
    pub emotion: String,
    /// Valence de l'humeur : -1.0 (tres negative) a +1.0 (tres positive)
    pub mood_valence: f32,
    /// Niveau de satisfaction ressentie [0.0 - 1.0]
    pub satisfaction: f32,
    /// Poids emotionnel du souvenir (les souvenirs forts sont mieux retenus)
    pub emotional_weight: f32,
    /// Date et heure de creation du souvenir (UTC)
    pub created_at: DateTime<Utc>,
    /// Score de similarite avec un vecteur de requete (rempli lors d'une recherche)
    pub similarity: f64,
    /// Signature chimique au moment de l'encodage (None pour les anciens souvenirs)
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Nouveau souvenir a inserer en base de donnees.
/// Contient l'embedding vectoriel et toutes les metadonnees associees.
pub struct NewMemory {
    /// Vecteur d'embedding (representation vectorielle du contenu pour pgvector)
    pub embedding: Vec<f32>,
    /// Resume textuel du souvenir
    pub text_summary: String,
    /// Donnees du stimulus d'origine en JSON
    pub stimulus_json: serde_json::Value,
    /// Decision prise : -1 (Non), 0 (Peut-etre), 1 (Oui)
    pub decision: i16,
    /// Etat neurochimique en JSON
    pub chemistry_json: serde_json::Value,
    /// Emotion dominante
    pub emotion: String,
    /// Valence de l'humeur
    pub mood_valence: f32,
    /// Satisfaction ressentie
    pub satisfaction: f32,
    /// Poids emotionnel
    pub emotional_weight: f32,
    /// Identifiant optionnel du souvenir episodique d'origine (lien tier 1 -> tier 2)
    pub source_episodic_id: Option<i64>,
    /// Signature chimique au moment de l'encodage
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Pool de connexions PostgreSQL pour Saphire.
/// Encapsule le pool deadpool et fournit toutes les methodes de persistance.
pub struct SaphireDb {
    /// Le pool de connexions sous-jacent (max 8 connexions simultanees)
    pub(crate) pool: Pool,
}

impl SaphireDb {
    /// Connecte au serveur PostgreSQL, cree le pool de connexions et execute
    /// les migrations de schema (creation des tables si elles n'existent pas).
    ///
    /// # Parametres
    /// - `config` : configuration de connexion (host, port, user, password, dbname)
    ///
    /// # Retour
    /// - `Ok(SaphireDb)` : le pool de connexions pret a l'emploi
    /// - `Err(DbError)` : erreur de connexion ou de migration
    pub async fn connect(config: &DbConfig) -> Result<Self, DbError> {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config.host(&config.host);
        pg_config.port(config.port);
        pg_config.user(&config.user);
        pg_config.password(&config.password);
        pg_config.dbname(&config.dbname);

        // Configuration du gestionnaire de recyclage des connexions.
        // RecyclingMethod::Fast : reutilise les connexions sans verification couteuse.
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr)
            .max_size(8) // Maximum 8 connexions simultanees dans le pool
            .build()
            .map_err(|e| DbError::Pool(e.to_string()))?;

        let db = Self { pool };
        // Executer les migrations pour creer/mettre a jour le schema
        db.run_migrations().await?;
        Ok(db)
    }

    /// Execute les migrations SQL depuis le fichier schema.sql embarque.
    /// Utilise IF NOT EXISTS pour etre idempotent (peut etre execute plusieurs fois).
    async fn run_migrations(&self) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.batch_execute(include_str!("../../sql/schema.sql")).await
            .map_err(|e| DbError::Migration(e.to_string()))?;
        Ok(())
    }

    /// Verifie que la connexion a la base de donnees est fonctionnelle.
    /// Utile pour les health checks et le monitoring.
    ///
    /// # Retour
    /// true si la connexion est operationnelle, false sinon
    pub async fn health_check(&self) -> bool {
        match self.pool.get().await {
            Ok(client) => client.query_one("SELECT 1", &[]).await.is_ok(),
            Err(_) => false,
        }
    }

    /// Statistiques des tables de la base principale.
    pub async fn table_stats(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;

        let tables = [
            "memories", "self_identity", "personality_traits",
            "thought_log", "founding_memories", "session_log",
            "tuning_params", "bandit_arms", "knowledge_log", "episodic_memories",
            "ocean_self_profile", "human_profiles",
            "personal_ethics", "personal_ethics_history",
            "dream_journal", "desires", "lessons", "wounds", "nn_learnings",
            "neural_connections", "sleep_history", "memory_vectors",
            "memory_archives", "lora_training_data",
        ];

        let mut stats = serde_json::Map::new();
        for table in &tables {
            let query = format!("SELECT COUNT(*) FROM {}", table);
            match client.query_one(&query, &[]).await {
                Ok(row) => {
                    let count: i64 = row.get(0);
                    stats.insert(table.to_string(), serde_json::json!(count));
                }
                Err(_) => {
                    stats.insert(table.to_string(), serde_json::json!(-1));
                }
            }
        }
        Ok(serde_json::Value::Object(stats))
    }
}
