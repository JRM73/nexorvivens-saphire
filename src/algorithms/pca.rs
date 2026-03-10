// =============================================================================
// pca.rs — PCA (Principal Component Analysis = Analyse en Composantes
//          Principales) simplifiée par itération de puissance
// =============================================================================
//
// Rôle : Implémente la PCA, une technique de réduction de dimensionnalité qui
//        projette les données sur les axes de variance maximale (composantes
//        principales). Utilise l'itération de puissance pour extraire les
//        vecteurs propres dominants de la matrice de covariance.
//
// Dépendances : aucune (calcul mathématique pur)
//
// Place dans l'architecture :
//   Utilisé par Saphire pour comprendre quels axes chimiques (neurotransmetteurs)
//   portent le plus d'information, et pour réduire la dimensionnalité des
//   représentations internes avant visualisation ou analyse. Fait partie du
//   sous-module algorithms/.
// =============================================================================

/// Résultat de la PCA — contient les données projetées, les composantes
/// principales et la variance expliquée par chacune.
pub struct PcaResult {
    /// Données projetées dans l'espace réduit (n points x k composantes)
    /// Chaque ligne correspond à un point original projeté sur les k composantes
    pub projected: Vec<Vec<f64>>,
    /// Composantes principales : k vecteurs de dimension d (espace original)
    /// Ce sont les vecteurs propres de la matrice de covariance, ordonnés
    /// par variance décroissante
    pub components: Vec<Vec<f64>>,
    /// Variance expliquée par chaque composante (valeurs propres associées)
    /// Plus la valeur est élevée, plus cette composante capture de l'information
    pub explained_variance: Vec<f64>,
}

/// Effectue une PCA par itération de puissance simplifiée.
///
/// Étapes de l'algorithme :
///   1. Centrer les données (soustraire la moyenne de chaque dimension)
///   2. Calculer la matrice de covariance (d x d)
///   3. Extraire les composantes principales par itération de puissance + déflation
///   4. Projeter les données centrées sur les composantes
///
/// Paramètre `data` : matrice de données (n points x d dimensions)
/// Paramètre `n_components` : nombre de composantes principales à extraire
/// Retourne : Option<PcaResult> — None si les données sont vides ou n_components > d
pub fn pca(data: &[Vec<f64>], n_components: usize) -> Option<PcaResult> {
    if data.is_empty() { return None; }
    let n = data.len();       // Nombre de points
    let d = data[0].len();    // Nombre de dimensions
    if n_components > d { return None; }

    // 1. Centrer les données : soustraire la moyenne de chaque dimension
    //    Pourquoi centrer : la PCA cherche les directions de variance maximale,
    //    et la variance se calcule autour de la moyenne
    let means: Vec<f64> = (0..d).map(|j| {
        data.iter().map(|row| row[j]).sum::<f64>() / n as f64
    }).collect();

    let centered: Vec<Vec<f64>> = data.iter().map(|row| {
        row.iter().zip(means.iter()).map(|(x, m)| x - m).collect()
    }).collect();

    // 2. Calculer la matrice de covariance (d x d)
    //    cov[i][j] = (1/(n-1)) * sum(centered[k][i] * centered[k][j])
    //    Division par (n-1) : correction de Bessel pour un estimateur non biaisé
    let mut cov = vec![vec![0.0; d]; d];
    for row in &centered {
        for i in 0..d {
            for j in 0..d {
                cov[i][j] += row[i] * row[j];
            }
        }
    }
    let divisor = (n - 1).max(1) as f64;
    for row in cov.iter_mut().take(d) {
        for val in row.iter_mut().take(d) {
            *val /= divisor;
        }
    }

    // 3. Extraire les composantes principales par itération de puissance
    //    et déflation matricielle
    let mut components = Vec::new();
    let mut explained_variance = Vec::new();
    let mut work_cov = cov.clone();

    for _ in 0..n_components {
        // Trouver le vecteur propre dominant (plus grande valeur propre)
        let (eigvec, eigval) = power_iteration(&work_cov, 100);
        components.push(eigvec.clone());
        explained_variance.push(eigval);

        // Déflation : soustraire la contribution de cette composante
        // de la matrice de covariance pour trouver la composante suivante
        // Formule : cov = cov - eigenvalue * eigvec * eigvec^T
        for i in 0..d {
            for j in 0..d {
                work_cov[i][j] -= eigval * eigvec[i] * eigvec[j];
            }
        }
    }

    // 4. Projeter les données centrées sur les composantes principales
    //    projection[i][k] = produit scalaire de centered[i] avec components[k]
    let projected: Vec<Vec<f64>> = centered.iter().map(|row| {
        components.iter().map(|comp| {
            row.iter().zip(comp.iter()).map(|(x, c)| x * c).sum::<f64>()
        }).collect()
    }).collect();

    Some(PcaResult { projected, components, explained_variance })
}

/// Itération de puissance pour trouver le vecteur propre dominant d'une matrice.
///
/// L'algorithme multiplie itérativement un vecteur initial par la matrice
/// et normalise le résultat. Le vecteur converge vers le vecteur propre
/// associé à la plus grande valeur propre en valeur absolue.
///
/// Pourquoi cette méthode : elle est simple, efficace pour les petites matrices,
/// et ne nécessite pas de bibliothèque d'algèbre linéaire externe.
///
/// Paramètre `matrix` : matrice carrée (d x d) dont on cherche le vecteur propre dominant
/// Paramètre `max_iter` : nombre maximum d'itérations
/// Retourne : tuple (vecteur_propre, valeur_propre) du mode dominant
fn power_iteration(matrix: &[Vec<f64>], max_iter: usize) -> (Vec<f64>, f64) {
    let d = matrix.len();
    // Initialiser le vecteur avec [1, 0, 0, ...] (vecteur unitaire e1)
    let mut v: Vec<f64> = (0..d).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();

    for _ in 0..max_iter {
        // Multiplier : w = M x v (produit matrice-vecteur)
        let w: Vec<f64> = (0..d).map(|i| {
            matrix[i].iter().zip(v.iter()).map(|(m, vi)| m * vi).sum::<f64>()
        }).collect();

        // Calculer la norme euclidienne du résultat
        let norm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        // Si la norme est quasi-nulle, la matrice est dégénérée
        if norm < 1e-10 { break; }

        // Normaliser le vecteur pour le prochain tour
        v = w.iter().map(|x| x / norm).collect();
    }

    // Calculer la valeur propre : lambda = v^T * M * v (quotient de Rayleigh)
    let mv: Vec<f64> = (0..d).map(|i| {
        matrix[i].iter().zip(v.iter()).map(|(m, vi)| m * vi).sum::<f64>()
    }).collect();
    let eigenvalue = v.iter().zip(mv.iter()).map(|(a, b)| a * b).sum::<f64>();

    (v, eigenvalue)
}
