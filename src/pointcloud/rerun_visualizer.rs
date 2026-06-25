use std::cell::Cell;

use super::protocol::PointCloudFrame;

#[cfg(feature = "rerun")]
use rerun as rr;

pub struct PointCloudVisualizer {
    ground_logged: Cell<bool>,
}

impl PointCloudVisualizer {
    pub fn new() -> Self {
        Self {
            ground_logged: Cell::new(false),
        }
    }
}

#[cfg(feature = "rerun")]
impl PointCloudVisualizer {
    pub fn log_point_cloud(
        &self,
        rec: &rr::RecordingStream,
        frame: &PointCloudFrame,
    ) {
        if frame.points.is_empty() {
            return;
        }
        self.ensure_ground_reference(rec);
        let positions: Vec<[f32; 3]> = frame.points.clone();
        let colors = Self::compute_colors(&frame.colors, &positions, &frame.normals);
        let _ = rec.log(
            "world/pointcloud",
            &rr::Points3D::new(positions)
                .with_colors(colors)
                .with_radii([0.01]),
        );
    }

    fn ensure_ground_reference(&self, rec: &rr::RecordingStream) {
        if self.ground_logged.get() {
            return;
        }
        self.ground_logged.set(true);

        let _ = rec.log(
            "world/axes/x",
            &rr::Arrows3D::from_vectors([(2.0, 0.0, 0.0)])
                .with_colors([rr::Color::from_rgb(255, 60, 60)]),
        );
        let _ = rec.log(
            "world/axes/y",
            &rr::Arrows3D::from_vectors([(0.0, 2.0, 0.0)])
                .with_colors([rr::Color::from_rgb(60, 255, 60)]),
        );
        let _ = rec.log(
            "world/axes/z",
            &rr::Arrows3D::from_vectors([(0.0, 0.0, 2.0)])
                .with_colors([rr::Color::from_rgb(60, 120, 255)]),
        );

        let grid_size: i32 = 10;
        let half = grid_size as f32 / 2.0;
        let grid_color = rr::Color::from_unmultiplied_rgba(100, 100, 120, 80);
        for i in 0..=grid_size {
            let c = i as f32 - half;
            let _ = rec.log(
                "world/ground_grid",
                &rr::LineStrips3D::new([vec![[c, -half, 0.0], [c, half, 0.0]]])
                    .with_colors([grid_color]),
            );
            let _ = rec.log(
                "world/ground_grid",
                &rr::LineStrips3D::new([vec![[-half, c, 0.0], [half, c, 0.0]]])
                    .with_colors([grid_color]),
            );
        }
    }

    fn compute_colors(
        raw: &[[u8; 4]],
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
    ) -> Vec<rr::Color> {
        let all_white = raw.iter().all(|c| *c == [255, 255, 255, 255]);
        if !all_white {
            return raw
                .iter()
                .map(|c| rr::Color::from_unmultiplied_rgba(c[0], c[1], c[2], c[3]))
                .collect();
        }
        let has_normals = normals.len() == positions.len()
            && normals.iter().any(|n| *n != [0.0, 0.0, 1.0]);
        if has_normals {
            return normals
                .iter()
                .map(|n| {
                    let up = n[2].abs();
                    if up > 0.8 {
                        rr::Color::from_unmultiplied_rgba(0xE8, 0xE4, 0xE0, 255)
                    } else if up < 0.3 {
                        rr::Color::from_unmultiplied_rgba(0xD4, 0xCF, 0xC9, 255)
                    } else {
                        rr::Color::from_unmultiplied_rgba(0xC0, 0xBB, 0xB5, 255)
                    }
                })
                .collect();
        }
        let z_min = positions.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
        let z_max = positions.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max);
        let z_range = (z_max - z_min).max(0.001);
        positions
            .iter()
            .map(|p| {
                let t = ((p[2] - z_min) / z_range).clamp(0.0, 1.0);
                let gray = 0xC0 + (t * 40.0) as u8;
                rr::Color::from_unmultiplied_rgba(gray, gray, gray, 230)
            })
            .collect()
    }
}

#[cfg(not(feature = "rerun"))]
impl PointCloudVisualizer {
    pub fn log_point_cloud(&self, _rec: &(), _frame: &PointCloudFrame) {}
}

impl Default for PointCloudVisualizer {
    fn default() -> Self {
        Self::new()
    }
}
