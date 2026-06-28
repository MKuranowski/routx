// (c) Copyright 2026 Mikołaj Kuranowski
// SPDX-License-Identifier: MIT

//! Routines for simplifying polylines.

/// Simplifies a line using the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm),
/// and assuming an Euclidean geometry.
///
/// Simplification is done in-place, by overwriting discarded elements. Returns a sub-slice
/// of `points` representing the simplified line. Any other elements are undefined.
pub fn simplify<T: Vec2>(points: &mut [T], epsilon: f32) -> &mut [T] {
    simplify_generic(points, epsilon, |pt| pt)
}

/// Simplifies a line using the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm),
/// and assuming an Euclidean geometry.
///
/// Operates on a flat slice of `f32`, encoding `[x0 y0 x1 y1 x2 y2]`. Any trailing odd elements
/// are ignored.
///
/// Simplification is done in-place, by overwriting discarded elements. Returns a sub-slice
/// of `points` representing the simplified line. Any other elements are undefined.
pub fn simplify_flat(flat_points: &mut [f32], epsilon: f32) -> &mut [f32] {
    let pairs_len = flat_points.len() / 2;
    let pairs: &mut [[f32; 2]] =
        unsafe { std::slice::from_raw_parts_mut(flat_points.as_mut_ptr().cast(), pairs_len) };
    let new_pairs_len = simplify(pairs, epsilon).len();
    &mut flat_points[0..2 * new_pairs_len]
}

/// Simplifies a line using the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm),
/// and assuming an Euclidean geometry.
///
/// Operates on an arbitrary slice of elements, which are used to indirectly identify points.
///
/// Simplification is done in-place, by overwriting discarded elements. Returns a sub-slice
/// of `points` representing the simplified line. Any other elements are undefined.
pub fn simplify_generic<Elem: Copy, Pos: Vec2, ElemToPos: Fn(Elem) -> Pos>(
    points: &mut [Elem],
    epsilon: f32,
    get_position: ElemToPos,
) -> &mut [Elem] {
    // Don't do anything if there are only at most 2 points
    if points.len() <= 2 {
        return points;
    }

    // We'll compute distances squared, so the epsilon must be squared as well
    let epsilon_sq = epsilon * epsilon;

    // Use a small heap-allocated stack instead of recursion
    let mut stack = Vec::with_capacity(32);
    stack.push((0_usize, points.len() - 1));

    // Re-write into the `points` buffer instead of allocating a new vector.
    // This means that all segments (in `stack`) need to be processed left-to-right.
    let mut write_idx = 0_usize;

    // Run the RDP algorithm, rejecting points which are too co-linear
    while let Some((start_idx, end_idx)) = stack.pop() {
        let mut furthest_idx = start_idx;
        let mut furthest_dist = 0.0;

        // Segment with more than 2 elements - find the most protruding point
        if end_idx > start_idx + 1 {
            let start = get_position(points[start_idx]);
            let end = get_position(points[end_idx]);
            let line = end.sub(start);
            let line_len = line.mag();

            for (offset, &elem) in points[start_idx + 1..end_idx].iter().enumerate() {
                // Calculate the distance to this point
                let pt = get_position(elem);
                let to_pt = pt.sub(start);
                let dist = if line_len < 1e-8 {
                    // start ≈ end - segment is a ring - compute the distance to the start
                    to_pt.mag()
                } else {
                    let projection = (to_pt.dot(line) / line_len).clamp(0.0, 1.0);
                    let projected = start.add(line.scale(projection));
                    pt.sub(projected).mag()
                };

                if dist > furthest_dist {
                    furthest_idx = start_idx + 1 + offset;
                    furthest_dist = dist;
                }
            }
        }

        if furthest_dist > epsilon_sq {
            // Furthest point is significant. Keep it, and simplify the segments on both sides.
            // Note that the stack needs to strictly operate on increasing indices.
            stack.push((furthest_idx, end_idx));
            stack.push((start_idx, furthest_idx));
        } else {
            // Furthest point does not protrude enough - just write the start of the segment
            points[write_idx] = points[start_idx];
            write_idx += 1;
        }
    }

    // Write the very last point, as it's never copied over
    points[write_idx] = points[points.len() - 1];
    write_idx += 1;

    // Return a reference to the flat array
    &mut points[0..write_idx]
}

/// Vector or a point in two-dimensional euclidean space.
///
/// Only provides operations necessary for running the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm),
/// so for computing distances between two points and a point and a segment (line between two other points).
pub trait Vec2: Sized + Copy {
    fn new(a: f32, b: f32) -> Self;
    fn x(self) -> f32;
    fn y(self) -> f32;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y())
    }

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y())
    }

    #[inline(always)]
    fn mag(self) -> f32 {
        self.x() * self.x() + self.y() * self.y()
    }

    #[inline(always)]
    fn dot(self, rhs: Self) -> f32 {
        self.x() * rhs.x() + self.y() * rhs.y()
    }

    #[inline(always)]
    fn scale(self, scalar: f32) -> Self {
        Self::new(self.x() * scalar, self.y() * scalar)
    }
}

impl Vec2 for (f32, f32) {
    #[inline(always)]
    fn new(a: f32, b: f32) -> Self {
        (a, b)
    }

    #[inline(always)]
    fn x(self) -> f32 {
        self.0
    }

    #[inline(always)]
    fn y(self) -> f32 {
        self.1
    }
}

impl Vec2 for [f32; 2] {
    #[inline(always)]
    fn new(a: f32, b: f32) -> Self {
        [a, b]
    }

    #[inline(always)]
    fn x(self) -> f32 {
        self[0]
    }

    #[inline(always)]
    fn y(self) -> f32 {
        self[1]
    }
}

#[cfg(test)]

mod tests {
    use super::simplify_flat;

    #[test]

    fn test_simplify() {
        let line = &mut [
            0.0_f32, 0.0, 0.55, 0.5, 1.0, 1.0, 0.7, 1.3, 0.2, 2.0, 0.25, 2.1, 0.6, 3.0, -0.1, 4.0,
        ];

        let simplified = simplify_flat(line, 0.1);

        assert_eq!(
            simplified,
            &[0.0_f32, 0.0, 1.0, 1.0, 0.2, 2.0, 0.6, 3.0, -0.1, 4.0],
        );
    }
}
