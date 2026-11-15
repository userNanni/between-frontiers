//! Hard-edged scanline fill of polygons into a 16-bit id buffer.
//!
//! A pixel belongs to a polygon when its centre is inside it (even-odd rule over the exterior and
//! the holes). A centre lying exactly on a left edge is inside, on a right edge outside, so two
//! polygons that share an edge never both claim a pixel and never leave a gap between them.

/// Ring in texture pixels. Closing the ring is implicit; a repeated first point is harmless.
pub type Ring = Vec<(f64, f64)>;

/// Writes `value` into every pixel of `buf` (row-major, `width` x `height`) whose centre lies
/// inside the polygon made of `rings` (exterior and holes, any orientation).
pub fn fill_polygon(buf: &mut [u16], width: u32, height: u32, rings: &[Ring], value: u16) {
    debug_assert_eq!(buf.len(), width as usize * height as usize);
    let mut crossings: Vec<(u32, f64)> = Vec::new();
    for ring in rings {
        let n = ring.len();
        if n < 3 {
            continue;
        }
        for i in 0..n {
            let (x0, y0) = ring[i];
            let (x1, y1) = ring[(i + 1) % n];
            if y0 == y1 {
                continue;
            }
            let (ya, yb) = if y0 < y1 { (y0, y1) } else { (y1, y0) };
            // Rows whose centre c = j + 0.5 satisfies ya <= c < yb.
            let first = (ya - 0.5).ceil().max(0.0);
            let last = (yb - 0.5).ceil().min(f64::from(height));
            if first >= last {
                continue;
            }
            let slope = (x1 - x0) / (y1 - y0);
            for j in first as u32..last as u32 {
                let c = f64::from(j) + 0.5;
                crossings.push((j, x0 + (c - y0) * slope));
            }
        }
    }
    crossings.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));

    let w = f64::from(width);
    for row in crossings.chunk_by(|a, b| a.0 == b.0) {
        let j = row[0].0 as usize;
        let (pairs, _) = row.as_chunks::<2>();
        for [(_, xa), (_, xb)] in pairs {
            // Columns whose centre c = i + 0.5 satisfies xa <= c < xb.
            let first = (xa - 0.5).ceil().clamp(0.0, w) as usize;
            let last = (xb - 0.5).ceil().clamp(0.0, w) as usize;
            if first < last {
                let base = j * width as usize;
                buf[base + first..base + last].fill(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(width: u32, height: u32, rings: &[Ring]) -> Vec<u16> {
        let mut buf = vec![0u16; (width * height) as usize];
        fill_polygon(&mut buf, width, height, rings, 7);
        buf
    }

    #[test]
    fn square_with_hole() {
        // 8x8 square from (1,1) to (9,9) with a 2x2 hole from (4,4) to (6,6), on a 10x10 grid.
        let outer = vec![(1.0, 1.0), (9.0, 1.0), (9.0, 9.0), (1.0, 9.0)];
        let hole = vec![(4.0, 4.0), (4.0, 6.0), (6.0, 6.0), (6.0, 4.0)];
        let buf = render(10, 10, &[outer, hole]);
        let filled = buf.iter().filter(|&&v| v == 7).count();
        assert_eq!(filled, 64 - 4);
        let at = |x: usize, y: usize| buf[y * 10 + x];
        assert_eq!(at(0, 0), 0);
        assert_eq!(at(1, 1), 7);
        assert_eq!(at(8, 8), 7);
        assert_eq!(at(9, 9), 0);
        assert_eq!(at(4, 4), 0);
        assert_eq!(at(5, 5), 0);
        assert_eq!(at(3, 4), 7);
        assert_eq!(at(6, 5), 7);
        // Only the value is written: no anti-aliasing, no other values.
        assert!(buf.iter().all(|&v| v == 0 || v == 7));
    }

    #[test]
    fn shared_edge_has_no_gap_or_overlap() {
        // Two triangles splitting the square (0,0)-(6,6) along a diagonal.
        let a = vec![(0.0, 0.0), (6.0, 0.0), (6.0, 6.0)];
        let b = vec![(0.0, 0.0), (6.0, 6.0), (0.0, 6.0)];
        let mut buf = vec![0u16; 36];
        let mut count_a = vec![0u16; 36];
        fill_polygon(&mut buf, 6, 6, std::slice::from_ref(&a), 1);
        fill_polygon(&mut count_a, 6, 6, &[a], 1);
        let mut count_b = vec![0u16; 36];
        fill_polygon(&mut count_b, 6, 6, std::slice::from_ref(&b), 1);
        fill_polygon(&mut buf, 6, 6, &[b], 2);
        assert!(buf.iter().all(|&v| v != 0), "gap: {buf:?}");
        let overlap = count_a
            .iter()
            .zip(&count_b)
            .filter(|(x, y)| **x == 1 && **y == 1);
        assert_eq!(overlap.count(), 0);
    }

    #[test]
    fn clips_to_the_buffer() {
        let big = vec![(-5.0, -5.0), (20.0, -5.0), (20.0, 20.0), (-5.0, 20.0)];
        let buf = render(4, 3, &[big]);
        assert!(buf.iter().all(|&v| v == 7));
    }

    #[test]
    fn sub_pixel_polygon_gets_no_pixel() {
        let tiny = vec![(2.1, 2.1), (2.4, 2.1), (2.4, 2.4)];
        let buf = render(5, 5, &[tiny]);
        assert!(buf.iter().all(|&v| v == 0));
    }
}
