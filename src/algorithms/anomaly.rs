// =============================================================================
// anomaly.rs — Z-Score anomaly detection
// =============================================================================
//
// Role: Implements a Z-Score-based anomaly detector (standard score).
//  A value is considered an anomaly if its deviation from the mean
//  exceeds a threshold measured in standard deviations.
//
// Dependencies: none (pure statistical computation)
//
// Place in architecture:
//  Used by Saphire to detect abnormal changes in its neurotransmitter
//  levels, mood, or any internal metric. Enables a form of "anomaly
//  awareness" — an alert signal when something falls outside the norm.
//  Part of the algorithms/ submodule.
// =============================================================================
/// Z-Score anomaly detector — maintains a sliding window of values
/// and computes the Z-Score of each new observation.
///
/// The Z-Score measures how many standard deviations a value is from
/// the mean: z = (value - mean) / std_dev
pub struct ZScoreDetector {
    /// History of observed values (sliding window)
    history: Vec<f64>,
    /// Maximum history size (oldest values are discarded)
    max_size: usize,
    /// Z-Score threshold beyond which a value is considered an anomaly
    /// (typically 2.0 or 3.0 — respectively ~5% or ~0.3% of normal data)
    threshold: f64,
}

impl ZScoreDetector {
    /// Creates a new anomaly detector.
    ///
    /// Parameter `max_size`: sliding history window size
    /// Parameter `threshold`: Z-Score threshold for anomaly detection
    /// Returns: an empty ZScoreDetector instance
    pub fn new(max_size: usize, threshold: f64) -> Self {
        Self {
            history: Vec::new(),
            max_size,
            threshold,
        }
    }

    /// Observes a new value and determines whether it is an anomaly.
    ///
    /// Adds the value to the history, maintains the sliding window,
    /// then computes the mean, standard deviation, and Z-Score of the
    /// new value.
    ///
    /// Why 5 minimum: with fewer than 5 observations, the statistics
    /// are too unstable for reliable anomaly detection.
    ///
    /// Parameter `value`: new observed value
    /// Returns: an AnomalyResult containing the diagnosis and statistics
    pub fn observe(&mut self, value: f64) -> AnomalyResult {
        // Add the value to the history
        self.history.push(value);
        // Maintain the sliding window by removing the oldest value
        if self.history.len() > self.max_size {
            self.history.remove(0);
        }

        // Not enough data for reliable statistics
        if self.history.len() < 5 {
            return AnomalyResult {
                is_anomaly: false,
                z_score: 0.0,
                mean: value,
                std_dev: 0.0,
            };
        }

        // Compute the mean of the history
        let n = self.history.len() as f64;
        let mean = self.history.iter().sum::<f64>() / n;
        // Compute the variance (mean of squared deviations)
        let variance = self.history.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        // Standard deviation = square root of the variance
        let std_dev = variance.sqrt();

        // Compute the Z-Score: number of standard deviations between the value and the mean
        // Guard against division by zero if std_dev is near-zero
        let z_score = if std_dev > 1e-10 {
            (value - mean) / std_dev
        } else {
            0.0
        };

        AnomalyResult {
            // An anomaly is detected if the absolute Z-Score exceeds the threshold
            is_anomaly: z_score.abs() > self.threshold,
            z_score,
            mean,
            std_dev,
        }
    }

    /// Returns the current mean of the history.
    ///
    /// Returns: the mean of values in the history, or 0.0 if empty
    pub fn mean(&self) -> f64 {
        if self.history.is_empty() { return 0.0; }
        self.history.iter().sum::<f64>() / self.history.len() as f64
    }
}

/// Anomaly detection result — contains the diagnosis and the
/// statistics used for the computation.
pub struct AnomalyResult {
    /// True if the value is considered an anomaly (|z_score| > threshold)
    pub is_anomaly: bool,
    /// Computed Z-Score: (value - mean) / std_dev
    /// Positive if the value is above the mean, negative otherwise
    pub z_score: f64,
    /// History mean at the time of observation
    pub mean: f64,
    /// History standard deviation at the time of observation
    pub std_dev: f64,
}
