// =============================================================================
// hardware/mod.rs — Detection materielle automatique
//
// Role : Au demarrage, detecte GPU, CPU, RAM, disque et Ollama pour ajuster
// les parametres (modele LLM, max_tokens, etc.). Fonctionne sans
// dependance supplementaire (Linux /proc, nvidia-smi, HTTP check).
//
// Place dans l'architecture :
//   Appele dans main.rs apres le chargement de la config, avant le boot de
//   l'agent. Le profil est stocke dans SaphireAgent et expose via API.
// =============================================================================

use std::process::Command;
use serde::{Deserialize, Serialize};

// =============================================================================
// Structures de detection
// =============================================================================

/// Profil materiel complet detecte au demarrage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub gpu: Option<GpuInfo>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
    pub ollama: Option<OllamaInfo>,
    pub detected_at: String,
}

/// Informations GPU (NVIDIA via nvidia-smi).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub driver_version: String,
}

/// Informations CPU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
}

/// Informations memoire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
}

/// Informations disque.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub total_gb: u64,
    pub available_gb: u64,
}

/// Informations Ollama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaInfo {
    pub version: String,
    pub models: Vec<String>,
}

/// Recommandations basees sur le profil materiel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRecommendations {
    pub llm_model: String,
    pub llm_max_tokens: u32,
    pub tokio_workers: usize,
    pub warnings: Vec<String>,
}

// =============================================================================
// Detection
// =============================================================================

impl HardwareProfile {
    /// Detecte le profil materiel complet.
    /// Appele une seule fois au demarrage — les commandes systeme sont synchrones.
    pub fn detect(ollama_url: &str) -> Self {
        Self {
            gpu: detect_gpu(),
            cpu: detect_cpu(),
            memory: detect_memory(),
            disk: detect_disk("/"),
            ollama: detect_ollama(ollama_url),
            detected_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Genere des recommandations basees sur le profil.
    pub fn recommend(&self, current_model: &str) -> HardwareRecommendations {
        let vram = self.gpu.as_ref().map(|g| g.vram_mb).unwrap_or(0);
        let ram = self.memory.total_mb;
        let mut warnings = Vec::new();

        // Recommandation modele LLM basee sur la VRAM
        let (recommended_model, max_tokens) = match vram {
            0 => {
                // Pas de GPU — recommander un petit modele ou CPU inference
                if ram > 32_000 {
                    ("qwen3:7b".to_string(), 4096u32)
                } else {
                    warnings.push("Pas de GPU detecte — performance LLM limitee".into());
                    ("qwen3:1.5b".to_string(), 2048)
                }
            }
            1..=4096 => {
                warnings.push(format!("VRAM limitee ({} MB) — modele reduit recommande", vram));
                ("qwen3:1.5b".to_string(), 2048)
            }
            4097..=8192 => ("qwen3:7b".to_string(), 4096),
            8193..=16384 => ("qwen3:14b".to_string(), 8192),
            _ => ("qwen3:32b".to_string(), 8192),
        };

        // Verifier si le modele actuel est disponible dans Ollama
        if let Some(ref ollama) = self.ollama {
            if !ollama.models.iter().any(|m| m.starts_with(current_model.split(':').next().unwrap_or(""))) {
                warnings.push(format!(
                    "Modele configure '{}' non trouve dans Ollama. Disponibles : {}",
                    current_model,
                    ollama.models.join(", "),
                ));
            }
        }

        // Avertissements RAM
        if ram < 8_000 {
            warnings.push(format!("RAM faible ({} MB) — risque d'OOM sous charge", ram));
        }

        let tokio_workers = (self.cpu.threads / 2).max(2);

        HardwareRecommendations {
            llm_model: recommended_model,
            llm_max_tokens: max_tokens,
            tokio_workers,
            warnings,
        }
    }

    /// Affiche le profil dans les logs du demarrage.
    pub fn log_summary(&self) {
        println!("  ─── Profil materiel ───");
        println!("  🖥️  CPU : {} ({} coeurs, {} threads)",
            self.cpu.model, self.cpu.cores, self.cpu.threads);
        println!("  🧮 RAM : {} MB total, {} MB disponible",
            self.memory.total_mb, self.memory.available_mb);
        if let Some(ref gpu) = self.gpu {
            println!("  🎮 GPU : {} ({} MB VRAM, driver {})",
                gpu.name, gpu.vram_mb, gpu.driver_version);
        } else {
            println!("  🎮 GPU : non detecte");
        }
        println!("  💾 Disque : {} — {} GB / {} GB",
            self.disk.path, self.disk.available_gb, self.disk.total_gb);
        if let Some(ref ollama) = self.ollama {
            println!("  🦙 Ollama : v{} — {} modele(s) : {}",
                ollama.version, ollama.models.len(),
                ollama.models.join(", "));
        } else {
            println!("  🦙 Ollama : non disponible");
        }
    }

    /// Retourne un JSON serialisable pour l'API.
    /// Champs aplatis pour le frontend (cpu_model, ram_total_gb, gpu_model, etc.)
    pub fn to_json(&self) -> serde_json::Value {
        let mut j = serde_json::json!({
            "cpu_model": self.cpu.model,
            "cpu_cores": self.cpu.cores,
            "cpu_threads": self.cpu.threads,
            "ram_total_gb": self.memory.total_mb / 1024,
            "ram_available_gb": self.memory.available_mb / 1024,
            "disk_path": self.disk.path,
            "disk_total_gb": self.disk.total_gb,
            "disk_available_gb": self.disk.available_gb,
            "os": "Linux",
            "detected_at": self.detected_at,
        });
        if let Some(ref gpu) = self.gpu {
            j["gpu_model"] = serde_json::json!(gpu.name);
            j["gpu"] = serde_json::json!(gpu.name);
            j["gpu_vram_gb"] = serde_json::json!(gpu.vram_mb / 1024);
            j["gpu_driver"] = serde_json::json!(gpu.driver_version);
        }
        if let Some(ref ollama) = self.ollama {
            j["ollama_version"] = serde_json::json!(ollama.version);
            j["ollama_models"] = serde_json::json!(ollama.models);
        }
        j
    }
}

// =============================================================================
// Fonctions de detection
// =============================================================================

/// Detecte le GPU NVIDIA via nvidia-smi.
fn detect_gpu() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,driver_version", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    let parts: Vec<&str> = line.split(", ").collect();
    if parts.len() < 3 {
        return None;
    }

    Some(GpuInfo {
        name: parts[0].trim().to_string(),
        vram_mb: parts[1].trim().parse().unwrap_or(0),
        driver_version: parts[2].trim().to_string(),
    })
}

/// Detecte le CPU via /proc/cpuinfo.
fn detect_cpu() -> CpuInfo {
    let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    let model = content.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    // Nombre de threads (lignes "processor")
    let threads = content.lines()
        .filter(|l| l.starts_with("processor"))
        .count()
        .max(1);

    // Nombre de coeurs physiques (cpu cores)
    let cores = content.lines()
        .find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(threads);

    CpuInfo { model, cores, threads }
}

/// Detecte la memoire via /proc/meminfo.
fn detect_memory() -> MemoryInfo {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();

    let parse_kb = |prefix: &str| -> u64 {
        content.lines()
            .find(|l| l.starts_with(prefix))
            .and_then(|l| {
                l.split_whitespace().nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(0)
    };

    let total_kb = parse_kb("MemTotal:");
    let available_kb = parse_kb("MemAvailable:");

    MemoryInfo {
        total_mb: total_kb / 1024,
        available_mb: available_kb / 1024,
    }
}

/// Detecte l'espace disque via la commande df.
fn detect_disk(path: &str) -> DiskInfo {
    let output = Command::new("df")
        .args(["--output=size,avail", "-BG", path])
        .output();

    let (total_gb, available_gb) = match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let line = stdout.lines().nth(1).unwrap_or("");
            let parts: Vec<&str> = line.split_whitespace().collect();
            let total = parts.first()
                .and_then(|s| s.trim_end_matches('G').parse::<u64>().ok())
                .unwrap_or(0);
            let avail = parts.get(1)
                .and_then(|s| s.trim_end_matches('G').parse::<u64>().ok())
                .unwrap_or(0);
            (total, avail)
        }
        _ => (0, 0),
    };

    DiskInfo {
        path: path.to_string(),
        total_gb,
        available_gb,
    }
}

/// Detecte Ollama via HTTP (version + modeles disponibles).
fn detect_ollama(base_url: &str) -> Option<OllamaInfo> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    // Detecter la version
    let version_url = format!("{}/api/version", base_url);
    let version = agent.get(&version_url)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(|s| s.to_string()))?;

    // Lister les modeles
    let tags_url = format!("{}/api/tags", base_url);
    let models = agent.get(&tags_url)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .and_then(|v| {
            v.get("models")
                .and_then(|arr| arr.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
        })
        .unwrap_or_default();

    Some(OllamaInfo { version, models })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_detection() {
        let cpu = detect_cpu();
        assert!(cpu.threads >= 1);
        assert!(cpu.cores >= 1);
        assert!(!cpu.model.is_empty());
    }

    #[test]
    fn test_memory_detection() {
        let mem = detect_memory();
        assert!(mem.total_mb > 0);
    }

    #[test]
    fn test_recommendations_no_gpu() {
        let profile = HardwareProfile {
            gpu: None,
            cpu: CpuInfo { model: "test".into(), cores: 4, threads: 8 },
            memory: MemoryInfo { total_mb: 16_000, available_mb: 8_000 },
            disk: DiskInfo { path: "/".into(), total_gb: 500, available_gb: 200 },
            ollama: None,
            detected_at: "2026-01-01".into(),
        };
        let rec = profile.recommend("qwen3:14b");
        assert!(!rec.warnings.is_empty());
        assert_eq!(rec.tokio_workers, 4);
    }

    #[test]
    fn test_recommendations_16gb_gpu() {
        let profile = HardwareProfile {
            gpu: Some(GpuInfo {
                name: "RTX 4060 Ti".into(),
                vram_mb: 16384,
                driver_version: "555.42".into(),
            }),
            cpu: CpuInfo { model: "test".into(), cores: 8, threads: 16 },
            memory: MemoryInfo { total_mb: 64_000, available_mb: 32_000 },
            disk: DiskInfo { path: "/".into(), total_gb: 1000, available_gb: 500 },
            ollama: Some(OllamaInfo {
                version: "0.5.0".into(),
                models: vec!["qwen3:14b".into(), "qwen3:7b".into()],
            }),
            detected_at: "2026-01-01".into(),
        };
        let rec = profile.recommend("qwen3:14b");
        assert_eq!(rec.llm_model, "qwen3:14b");
        assert_eq!(rec.llm_max_tokens, 8192);
        assert!(rec.warnings.is_empty());
    }
}
