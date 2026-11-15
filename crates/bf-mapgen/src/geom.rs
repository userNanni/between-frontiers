//! Planar helpers in texture pixels: ring area and the label anchor.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::raster::Ring;

/// Unsigned shoelace area of one ring.
pub fn ring_area(ring: &[(f64, f64)]) -> f64 {
    let n = ring.len();
    let mut twice = 0.0;
    for i in 0..n {
        let (x0, y0) = ring[i];
        let (x1, y1) = ring[(i + 1) % n];
        twice += x0 * y1 - x1 * y0;
    }
    (twice / 2.0).abs()
}

/// Area of a polygon given as exterior followed by holes.
pub fn polygon_area(rings: &[Ring]) -> f64 {
    let mut area = 0.0;
    for (i, ring) in rings.iter().enumerate() {
        if i == 0 {
            area += ring_area(ring);
        } else {
            area -= ring_area(ring);
        }
    }
    area.max(0.0)
}

/// Even-odd point-in-polygon test over all rings.
fn contains(rings: &[Ring], x: f64, y: f64) -> bool {
    let mut inside = false;
    for ring in rings {
        let n = ring.len();
        for i in 0..n {
            let (x0, y0) = ring[i];
            let (x1, y1) = ring[(i + 1) % n];
            if (y0 <= y) != (y1 <= y) && x < x0 + (y - y0) * (x1 - x0) / (y1 - y0) {
                inside = !inside;
            }
        }
    }
    inside
}

fn segment_distance_sq(px: f64, py: f64, (ax, ay): (f64, f64), (bx, by): (f64, f64)) -> f64 {
    let (dx, dy) = (bx - ax, by - ay);
    let len_sq = dx * dx + dy * dy;
    let t = if len_sq > 0.0 {
        (((px - ax) * dx + (py - ay) * dy) / len_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (cx, cy) = (ax + t * dx - px, ay + t * dy - py);
    cx * cx + cy * cy
}

/// Distance from a point to the polygon outline: positive inside, negative outside.
fn signed_distance(rings: &[Ring], x: f64, y: f64) -> f64 {
    let mut best = f64::INFINITY;
    for ring in rings {
        let n = ring.len();
        for i in 0..n {
            best = best.min(segment_distance_sq(x, y, ring[i], ring[(i + 1) % n]));
        }
    }
    let d = best.sqrt();
    if contains(rings, x, y) { d } else { -d }
}

#[derive(Clone, Copy)]
struct Cell {
    x: f64,
    y: f64,
    half: f64,
    distance: f64,
    potential: f64,
}

impl Cell {
    fn new(rings: &[Ring], x: f64, y: f64, half: f64) -> Self {
        let distance = signed_distance(rings, x, y);
        Self {
            x,
            y,
            half,
            distance,
            potential: distance + half * std::f64::consts::SQRT_2,
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Cell {}
impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        // Ties broken by position so the result never depends on heap internals.
        self.potential
            .total_cmp(&other.potential)
            .then(other.y.total_cmp(&self.y))
            .then(other.x.total_cmp(&self.x))
    }
}

const MAX_SPLITS: usize = 5_000;

/// Pole of inaccessibility (Mapbox "polylabel"): the point inside the polygon farthest from its
/// outline, to within `precision` pixels. Always inside for a non-degenerate polygon, unlike the
/// centroid of a crescent or a ring-shaped province.
pub fn label_point(rings: &[Ring], precision: f64) -> (f64, f64) {
    let exterior = &rings[0];
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for &(x, y) in exterior {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let (w, h) = (max_x - min_x, max_y - min_y);
    let cell_size = w.min(h);
    if cell_size <= 0.0 {
        return (min_x, min_y);
    }
    let half = cell_size / 2.0;

    let mut queue = BinaryHeap::new();
    let mut x = min_x;
    while x < max_x {
        let mut y = min_y;
        while y < max_y {
            queue.push(Cell::new(rings, x + half, y + half, half));
            y += cell_size;
        }
        x += cell_size;
    }

    // Seed with the bounding-box centre.
    let mut best = Cell::new(rings, min_x + w / 2.0, min_y + h / 2.0, 0.0);
    // Cap the work for huge, intricate outlines; precision only matters for small shapes.
    let mut splits = 0usize;
    while let Some(cell) = queue.pop() {
        if cell.distance > best.distance {
            best = cell;
        }
        // The queue is ordered by potential, so no remaining cell can do better.
        if cell.potential - best.distance <= precision || splits >= MAX_SPLITS {
            break;
        }
        splits += 1;
        let q = cell.half / 2.0;
        for (dx, dy) in [(-q, -q), (q, -q), (-q, q), (q, q)] {
            queue.push(Cell::new(rings, cell.x + dx, cell.y + dy, q));
        }
    }
    if best.distance < 0.0 {
        // Degenerate sliver: fall back to the midpoint of the first exterior edge.
        let (a, b) = (exterior[0], exterior[1 % exterior.len()]);
        return ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    }
    (best.x, best.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_with_hole() {
        let outer = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let hole = vec![(2.0, 2.0), (4.0, 2.0), (4.0, 4.0), (2.0, 4.0)];
        assert_eq!(polygon_area(&[outer, hole]), 96.0);
    }

    #[test]
    fn label_inside_a_crescent() {
        // A "C": the centroid of this shape falls in the open part, outside the polygon.
        let c = vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 2.0),
            (2.0, 2.0),
            (2.0, 8.0),
            (10.0, 8.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ];
        let rings = vec![c];
        let (x, y) = label_point(&rings, 0.1);
        assert!(contains(&rings, x, y), "({x}, {y}) is outside");
        assert!(x < 2.0, "label should sit in the spine: ({x}, {y})");
    }

    #[test]
    fn label_avoids_holes() {
        let outer = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let hole = vec![(3.0, 3.0), (7.0, 3.0), (7.0, 7.0), (3.0, 7.0)];
        let rings = vec![outer, hole];
        let (x, y) = label_point(&rings, 0.1);
        assert!(contains(&rings, x, y), "({x}, {y}) is outside");
    }
}
