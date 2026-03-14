// =============================================================================
// pca.rs — PCA (Principal Component Analysis) simplified via power iteration
// =============================================================================
//
// Role: Implements PCA, a dimensionality reduction technique that
//  projects data onto the axes of maximum variance (principal
//  components). Uses power iteration to extract the dominant
//  eigenvectors of the covariance matrix.
//
// Dependencies: none (pure mathematical computation)
//
// Place in architecture:
//  Used by Saphire to understand which chemical axes (neurotransmitters)
//  carry the most information, and to reduce the dimensionality of
//  internal representations before visualization or analysis. Part of the
//  algorithms/ submodule.
// =============================================================================
/// PCA result — contains the projected data, the principal
/// components and the variance explained by each.
pub struct PcaResult {
    /// Projected data in reduced space (n points x k components)
    /// Each row corresponds to an original point projected onto the k components
    pub projected: Vec<Vec<f64>>,
    /// Principal components: k vectors of dimension d (original space)
    /// These are the eigenvectors of the covariance matrix, ordered
    /// by decreasing variance
    pub components: Vec<Vec<f64>>,
    /// Variance explained by each component (associated eigenvalues)
    /// The higher the value, the more information this component captures
    pub explained_variance: Vec<f64>,
}

/// Performs PCA via simplified power iteration.
///
/// Algorithm steps:
///  1. Center the data (subtract the mean of each dimension)
///  2. Compute the covariance matrix (d x d)
///  3. Extract principal components via power iteration + deflation
///  4. Project centered data onto the components
///
/// Parameter `data`: data matrix (n points x d dimensions)
/// Parameter `n_components`: number of principal components to extract
/// Returns: Option<PcaResult> — None if data is empty or n_components > d
pub fn pca(data: &[Vec<f64>], n_components: usize) -> Option<PcaResult> {
    if data.is_empty() { return None; }
    let n = data.len(); // Number of points    let d = data[0].len(); // Number of dimensions    if n_components > d { return None; }

    // 1. Center the data: subtract the mean of each dimension
    //  Why center: PCA seeks the directions of maximum variance,
    //  and variance is computed around the mean
    let means: Vec<f64> = (0..d).map(|j| {
        data.iter().map(|row| row[j]).sum::<f64>() / n as f64
    }).collect();

    let centered: Vec<Vec<f64>> = data.iter().map(|row| {
        row.iter().zip(means.iter()).map(|(x, m)| x - m).collect()
    }).collect();

    // 2. Compute the covariance matrix (d x d)
    //    cov[i][j] = (1/(n-1)) * sum(centered[k][i] * centered[k][j])
    //  Division by (n-1): Bessel's correction for an unbiased estimator
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

    // 3. Extract principal components via power iteration
    //  and matrix deflation
    let mut components = Vec::new();
    let mut explained_variance = Vec::new();
    let mut work_cov = cov.clone();

    for _ in 0..n_components {
        // Find the dominant eigenvector (largest eigenvalue)
        let (eigvec, eigval) = power_iteration(&work_cov, 100);
        components.push(eigvec.clone());
        explained_variance.push(eigval);

        // Deflation: subtract this component's contribution from the
        // covariance matrix to find the next component
        // Formula: cov = cov - eigenvalue * eigvec * eigvec^T
        for i in 0..d {
            for j in 0..d {
                work_cov[i][j] -= eigval * eigvec[i] * eigvec[j];
            }
        }
    }

    // 4. Project centered data onto the principal components
    //  projection[i][k] = dot product of centered[i] with components[k]
    let projected: Vec<Vec<f64>> = centered.iter().map(|row| {
        components.iter().map(|comp| {
            row.iter().zip(comp.iter()).map(|(x, c)| x * c).sum::<f64>()
        }).collect()
    }).collect();

    Some(PcaResult { projected, components, explained_variance })
}

/// Power iteration to find the dominant eigenvector of a matrix.
///
/// The algorithm iteratively multiplies an initial vector by the matrix
/// and normalizes the result. The vector converges to the eigenvector
/// associated with the largest eigenvalue in absolute value.
///
/// Why this method: it is simple, efficient for small matrices,
/// and requires no external linear algebra library.
///
/// Parameter `matrix`: square matrix (d x d) whose dominant eigenvector we seek
/// Parameter `max_iter`: maximum number of iterations
/// Returns: tuple (eigenvector, eigenvalue) of the dominant mode
fn power_iteration(matrix: &[Vec<f64>], max_iter: usize) -> (Vec<f64>, f64) {
    let d = matrix.len();
    // Initialize the vector with [1, 0, 0, ...] (unit vector e1)
    let mut v: Vec<f64> = (0..d).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();

    for _ in 0..max_iter {
        // Multiply: w = M x v (matrix-vector product)
        let w: Vec<f64> = (0..d).map(|i| {
            matrix[i].iter().zip(v.iter()).map(|(m, vi)| m * vi).sum::<f64>()
        }).collect();

        // Compute the Euclidean norm of the result
        let norm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        // If the norm is near-zero, the matrix is degenerate
        if norm < 1e-10 { break; }

        // Normalize the vector for the next iteration
        v = w.iter().map(|x| x / norm).collect();
    }

    // Compute the eigenvalue: lambda = v^T * M * v (Rayleigh quotient)
    let mv: Vec<f64> = (0..d).map(|i| {
        matrix[i].iter().zip(v.iter()).map(|(m, vi)| m * vi).sum::<f64>()
    }).collect();
    let eigenvalue = v.iter().zip(mv.iter()).map(|(a, b)| a * b).sum::<f64>();

    (v, eigenvalue)
}
