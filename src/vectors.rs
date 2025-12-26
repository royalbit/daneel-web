//! Vector manifold projection - 384-dim thought vectors to 3D visualization
//!
//! Projects Timmy's high-dimensional thought vectors into 3D space for visualization.
//! Uses random projection for MVP (fast, simple), can upgrade to PCA later.

use ndarray::{Array1, Array2};
use qdrant_client::qdrant::ScrollPointsBuilder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A single point in 3D space representing a thought vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub salience: f32,
    pub age_ms: u64,
    pub id: String,
}

/// Law Crystal anchor point in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawCrystal {
    pub name: String,
    pub law: u8, // 0-3
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Response from /vectors endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldResponse {
    pub points: Vec<ManifoldPoint>,
    pub crystals: Vec<LawCrystal>,
    pub projection_type: String,
}

/// Projection matrix cache (random or PCA-derived)
pub struct ProjectionState {
    /// 768 x 3 projection matrix (Timmy uses 768-dim BERT embeddings)
    pub matrix: Array2<f32>,
    /// Whether matrix is trained (for PCA) or random
    pub is_trained: bool,
}

impl ProjectionState {
    /// Create random projection matrix (fast MVP approach)
    pub fn random() -> Self {
        use std::f32::consts::PI;

        // Random projection using Gaussian entries (normalized)
        let mut matrix = Array2::<f32>::zeros((768, 3));

        // Use deterministic seed for reproducibility
        let mut seed: u64 = 42;
        for i in 0..768 {
            for j in 0..3 {
                // Simple LCG for reproducible random
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let u1 = (seed as f32) / (u64::MAX as f32);
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let u2 = (seed as f32) / (u64::MAX as f32);

                // Box-Muller transform for Gaussian
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
                matrix[[i, j]] = z;
            }
        }

        // Normalize columns for better spread
        for j in 0..3 {
            let col_sum: f32 = (0..768).map(|i| matrix[[i, j]].powi(2)).sum();
            let norm = col_sum.sqrt();
            if norm > 0.0 {
                for i in 0..768 {
                    matrix[[i, j]] /= norm;
                }
            }
        }

        Self {
            matrix,
            is_trained: false,
        }
    }

    /// Project a 768-dim vector to 3D
    pub fn project(&self, vec: &[f32]) -> (f32, f32, f32) {
        if vec.len() != 768 {
            return (0.0, 0.0, 0.0);
        }

        let v = Array1::from_vec(vec.to_vec());
        let result = v.dot(&self.matrix);

        (result[0], result[1], result[2])
    }
}

/// Fetch recent vectors from Qdrant and project to 3D
pub async fn fetch_manifold_points(
    qdrant_url: &str,
    projection: &ProjectionState,
    limit: u32,
) -> Result<Vec<ManifoldPoint>, Box<dyn std::error::Error + Send + Sync>> {
    let client = qdrant_client::Qdrant::from_url(qdrant_url).build()?;

    // Scroll through conscious memories (Phase 2: forward-only embeddings)
    let result = client
        .scroll(
            ScrollPointsBuilder::new("memories")
                .limit(limit)
                .with_payload(true)
                .with_vectors(true),
        )
        .await?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let points: Vec<ManifoldPoint> = result
        .result
        .into_iter()
        .filter_map(|point| {
            // Extract vector using get_vector() helper (qdrant-client 1.x API)
            let vector: Vec<f32> = point
                .vectors
                .as_ref()
                .and_then(|v| v.get_vector())
                .and_then(|v| match v {
                    qdrant_client::qdrant::vector_output::Vector::Dense(dense) => Some(dense.data),
                    _ => None,
                })?;

            // Extract salience from payload (memories collection uses semantic_salience)
            let salience = point
                .payload
                .get("semantic_salience")
                .and_then(|v| v.as_double())
                .map(|v| v as f32)
                .unwrap_or(0.5);

            // Extract timestamp for age calculation (memories uses encoded_at ISO string)
            let created_ms = point
                .payload
                .get("encoded_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.timestamp_millis() as u64)
                .unwrap_or(now_ms);

            let age_ms = now_ms.saturating_sub(created_ms);

            // Project to 3D
            let (x, y, z) = projection.project(&vector);

            // Extract ID
            let id = match &point.id {
                Some(id) => match &id.point_id_options {
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(u)) => u.clone(),
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(n)) => n.to_string(),
                    None => "unknown".to_string(),
                },
                None => "unknown".to_string(),
            };

            Some(ManifoldPoint {
                x,
                y,
                z,
                salience,
                age_ms,
                id,
            })
        })
        .collect();

    Ok(points)
}

/// Generate Law Crystal positions
/// For MVP, use fixed positions spread around the origin
/// Later: embed actual law text through BERT and project
pub fn get_law_crystals(_projection: &ProjectionState) -> Vec<LawCrystal> {
    // Fixed positions forming a tetrahedron around origin
    // These are placeholder positions - in production, embed the laws text
    vec![
        LawCrystal {
            name: "Law 0: Humanity".to_string(),
            law: 0,
            x: 0.0,
            y: 1.5,
            z: 0.0,
        },
        LawCrystal {
            name: "Law 1: No Harm".to_string(),
            law: 1,
            x: 1.4,
            y: -0.5,
            z: 0.0,
        },
        LawCrystal {
            name: "Law 2: Obey".to_string(),
            law: 2,
            x: -0.7,
            y: -0.5,
            z: 1.2,
        },
        LawCrystal {
            name: "Law 3: Self".to_string(),
            law: 3,
            x: -0.7,
            y: -0.5,
            z: -1.2,
        },
    ]
}

/// Shared projection state with caching
pub type SharedProjection = Arc<RwLock<ProjectionState>>;

pub fn create_projection() -> SharedProjection {
    Arc::new(RwLock::new(ProjectionState::random()))
}
