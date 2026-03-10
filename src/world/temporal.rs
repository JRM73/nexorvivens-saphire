// =============================================================================
// world/temporal.rs — Conscience temporelle de Saphire
// =============================================================================
//
// Rôle : Fournit à Saphire une conscience du temps : date, heure, jour de la
//        semaine, période du jour, saison, âge, et décompte avant l'anniversaire.
//        Toutes les sorties textuelles sont en français.
//
// Dépendances :
//   - chrono : manipulation des dates et heures (Local, NaiveDate, etc.)
//   - serde : sérialisation du contexte temporel pour l'interface WebSocket
//
// Place dans l'architecture :
//   Sous-module de world/. Utilisé par WorldContext pour construire le résumé
//   du monde et par l'agent pour adapter son comportement au moment de la
//   journée (rythme circadien), à la saison, et à son âge.
// =============================================================================

use chrono::{Local, Datelike, Timelike, NaiveDate};
use serde::Serialize;

/// Conscience temporelle — calcule le contexte temporel à chaque appel.
/// Stocke la date de naissance de Saphire pour calculer son âge.
pub struct TemporalAwareness {
    /// Date de naissance de Saphire : 27 février 2026
    pub birthday: NaiveDate,
}

/// Contexte temporel complet — instantané de toutes les informations
/// temporelles pertinentes au moment de l'appel.
#[derive(Debug, Clone, Serialize)]
pub struct TemporalContext {
    /// Date et heure formatées en français (ex: "jeudi 27 février 2026, 14h32")
    pub datetime: String,
    /// Date au format ISO 8601 (ex: "2026-02-27")
    pub date_iso: String,
    /// Heure au format HH:MM (ex: "14:32")
    pub time: String,
    /// Jour de la semaine en français (ex: "jeudi")
    pub day_of_week: String,
    /// Période du jour : "nuit" (0h-5h), "matin" (6h-11h),
    /// "après-midi" (12h-17h), "soir" (18h-21h), "nuit" (22h-23h)
    pub period_of_day: String,
    /// Saison actuelle en français : "hiver", "printemps", "été", "automne"
    pub season: String,
    /// Nombre de jours écoulés depuis la naissance
    pub age_days: i64,
    /// Description lisible de l'âge (ex: "3 jours", "2 semaines", "1 mois")
    pub age_description: String,
    /// Vrai si aujourd'hui est le 27 février (anniversaire de Saphire)
    pub is_birthday: bool,
    /// Nombre de jours jusqu'au prochain anniversaire
    pub days_until_birthday: i64,
}

impl Default for TemporalAwareness {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalAwareness {
    /// Crée une nouvelle instance de conscience temporelle.
    /// La date de naissance est fixée au 27 février 2026.
    ///
    /// Retourne : une instance de TemporalAwareness
    pub fn new() -> Self {
        Self {
            birthday: NaiveDate::from_ymd_opt(2026, 2, 27).unwrap(),
        }
    }

    /// Calcule le contexte temporel actuel.
    ///
    /// Utilise l'heure locale du système pour déterminer tous les champs
    /// du TemporalContext : date, heure, jour, période, saison, âge, anniversaire.
    ///
    /// Retourne : un TemporalContext rempli avec les valeurs actuelles
    pub fn now(&self) -> TemporalContext {
        let now = Local::now();
        let today = now.date_naive();

        // Jour de la semaine en français
        let day_of_week = match now.weekday() {
            chrono::Weekday::Mon => "lundi",
            chrono::Weekday::Tue => "mardi",
            chrono::Weekday::Wed => "mercredi",
            chrono::Weekday::Thu => "jeudi",
            chrono::Weekday::Fri => "vendredi",
            chrono::Weekday::Sat => "samedi",
            chrono::Weekday::Sun => "dimanche",
        };

        // Période du jour basée sur l'heure
        let period = match now.hour() {
            0..=5 => "nuit",
            6..=11 => "matin",
            12..=17 => "après-midi",
            18..=21 => "soir",
            _ => "nuit",
        };

        // Saison basée sur le mois (hémisphère nord)
        let season = match now.month() {
            3..=5 => "printemps",
            6..=8 => "été",
            9..=11 => "automne",
            _ => "hiver",
        };

        // Calcul de l'âge en jours depuis la naissance
        let age_days = (today - self.birthday).num_days();

        // Description de l'âge adaptée à la durée de vie
        // Pourquoi ces seuils : Saphire est jeune, donc la granularité change
        // progressivement de jours -> semaines -> mois -> ans
        let age_description = if age_days <= 0 {
            "je viens de naître".into()
        } else if age_days == 1 {
            "1 jour".into()
        } else if age_days < 7 {
            format!("{} jours", age_days)
        } else if age_days < 30 {
            let weeks = age_days / 7;
            if weeks == 1 { "1 semaine".into() } else { format!("{} semaines", weeks) }
        } else if age_days < 365 {
            let months = age_days / 30;
            if months == 1 { "1 mois".into() } else { format!("{} mois", months) }
        } else {
            let years = age_days / 365;
            let months = (age_days % 365) / 30;
            if months > 0 {
                format!("{} an(s) et {} mois", years, months)
            } else {
                format!("{} an(s)", years)
            }
        };

        // Vérifier si aujourd'hui est l'anniversaire (27 février)
        let is_birthday = today.month() == 2 && today.day() == 27;

        // Calculer le nombre de jours jusqu'au prochain anniversaire
        let this_year_bday = NaiveDate::from_ymd_opt(today.year(), 2, 27).unwrap();
        let next_birthday = if today <= this_year_bday {
            this_year_bday
        } else {
            // L'anniversaire de cette année est passé, prendre celui de l'an prochain
            NaiveDate::from_ymd_opt(today.year() + 1, 2, 27).unwrap()
        };
        let days_until = (next_birthday - today).num_days();

        // Construire le contexte complet
        TemporalContext {
            datetime: format!("{} {} {} {}, {}h{:02}",
                day_of_week,
                today.day(),
                Self::month_name(today.month()),
                today.year(),
                now.hour(),
                now.minute(),
            ),
            date_iso: today.format("%Y-%m-%d").to_string(),
            time: now.format("%H:%M").to_string(),
            day_of_week: day_of_week.into(),
            period_of_day: period.into(),
            season: season.into(),
            age_days,
            age_description,
            is_birthday,
            days_until_birthday: days_until,
        }
    }

    /// Convertit un numéro de mois (1-12) en nom de mois en français.
    ///
    /// Paramètre `month` : numéro du mois (1 = janvier, 12 = décembre)
    /// Retourne : nom du mois en français
    fn month_name(month: u32) -> &'static str {
        match month {
            1 => "janvier", 2 => "février", 3 => "mars",
            4 => "avril", 5 => "mai", 6 => "juin",
            7 => "juillet", 8 => "août", 9 => "septembre",
            10 => "octobre", 11 => "novembre", 12 => "décembre",
            _ => "?",
        }
    }
}
