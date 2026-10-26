//! The site: a terrain's triangles from its points (a Delaunay triangulation, written here), where a plane cuts it (its
//! contour lines in plan, its ground line in a section), its edge, and how a property line reads (its bearings).

use std::collections::HashMap;

use crate::geom::{self, P2};

/// A point of a terrain, mm: east, north, and its height.
pub type P3 = [f64; 3];

const NONE: usize = usize::MAX;

/// The Delaunay triangulation of `pts` in plan: triangles as indices into `pts`, each counter-clockwise, the circle through
/// each holding no other point. A point on one taken already is left out, and so are all of them when they lie on one line.
/// A sweep from the middle out (as S-hull sweeps): each point in turn joins the hull of those before it, and the edges it
/// makes are flipped until every triangle is Delaunay (Lawson's flips).
pub fn delaunay(pts: &[P2]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if n < 3 || pts.iter().any(|p| !p.is_finite()) {
        return Vec::new();
    }
    // About the middle of their box, so the arithmetic keeps its digits.
    let (lo, hi) = pts.iter().fold((pts[0], pts[0]), |(lo, hi), p| (P2(lo.0.min(p.0), lo.1.min(p.1)), P2(hi.0.max(p.0), hi.1.max(p.1))));
    let mid = (lo + hi) * 0.5;
    let p: Vec<P2> = pts.iter().map(|q| *q - mid).collect();
    let d2 = |a: P2, b: P2| (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2);
    // The seed triangle: the point nearest the middle, the one nearest it, and the one making the smallest circle with them.
    let Some(i0) = (0..n).min_by(|&a, &b| d2(p[a], P2(0.0, 0.0)).total_cmp(&d2(p[b], P2(0.0, 0.0)))) else { return Vec::new() };
    let Some(i1) = (0..n).filter(|&i| d2(p[i], p[i0]) > 0.0).min_by(|&a, &b| d2(p[a], p[i0]).total_cmp(&d2(p[b], p[i0]))) else {
        return Vec::new();
    };
    let Some(i2) = (0..n)
        .filter(|&i| i != i0 && i != i1)
        .filter_map(|i| circumradius2(p[i0], p[i1], p[i]).filter(|r| r.is_finite()).map(|r| (i, r)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
    else {
        return Vec::new();
    };
    let (i1, i2) = if (p[i1] - p[i0]).cross(p[i2] - p[i0]) < 0.0 { (i2, i1) } else { (i1, i2) };
    let Some(centre) = circumcentre(p[i0], p[i1], p[i2]) else { return Vec::new() };
    let mut order: Vec<usize> = (0..n).collect();
    let dist: Vec<f64> = p.iter().map(|q| d2(*q, centre)).collect();
    order.sort_by(|&a, &b| dist[a].total_cmp(&dist[b]).then(a.cmp(&b)));
    let size = (n as f64).sqrt().ceil().max(1.0) as usize;
    let mut t = Sweep {
        p: &p,
        tris: Vec::with_capacity(6 * n),
        half: Vec::with_capacity(6 * n),
        prev: vec![0; n],
        next: vec![0; n],
        hull_tri: vec![0; n],
        start: i0,
        hash: vec![NONE; size],
        centre,
        stack: Vec::new(),
    };
    (t.next[i0], t.next[i1], t.next[i2]) = (i1, i2, i0);
    (t.prev[i0], t.prev[i1], t.prev[i2]) = (i2, i0, i1);
    (t.hull_tri[i0], t.hull_tri[i1], t.hull_tri[i2]) = (0, 1, 2);
    for i in [i0, i1, i2] {
        let k = t.key(p[i]);
        t.hash[k] = i;
    }
    t.add(i0, i1, i2, NONE, NONE, NONE);
    let mut last: Option<P2> = None;
    for &k in &order {
        let q = p[k];
        if last.is_some_and(|l| d2(l, q) <= 1e-12) || k == i0 || k == i1 || k == i2 {
            continue;
        }
        last = Some(q);
        t.insert(k);
    }
    t.tris.as_chunks::<3>().0.iter().filter(|&&[a, b, c]| (pts[b] - pts[a]).cross(pts[c] - pts[a]) > 0.0).copied().collect()
}

fn circumradius2(a: P2, b: P2, c: P2) -> Option<f64> {
    let o = circumcentre(a, b, c)?;
    Some((o.0 - a.0).powi(2) + (o.1 - a.1).powi(2))
}

fn circumcentre(a: P2, b: P2, c: P2) -> Option<P2> {
    let (b, c) = (b - a, c - a);
    let d = 2.0 * b.cross(c);
    if d.abs() < 1e-12 {
        return None;
    }
    let (bl, cl) = (b.dot(b), c.dot(c));
    let o = P2(a.0 + (c.1 * bl - b.1 * cl) / d, a.1 + (b.0 * cl - c.0 * bl) / d);
    o.is_finite().then_some(o)
}

/// Whether `d` lies inside the circle through `a`, `b`, `c` (counter-clockwise), past rounding.
fn in_circle(a: P2, b: P2, c: P2, d: P2) -> bool {
    let (a, b, c) = (a - d, b - d, c - d);
    let (la, lb, lc) = (a.dot(a), b.dot(b), c.dot(c));
    let (x, y, z) = (b.cross(c), c.cross(a), a.cross(b));
    let det = la * x + lb * y + lc * z;
    det > 1e-12 * (la * x.abs() + lb * y.abs() + lc * z.abs())
}

/// The triangulation being swept: its triangles (three corners each, their edges `3t`, `3t + 1`, `3t + 2` from each corner to
/// the next) and each edge's twin in the triangle across it; the hull so far, round its points both ways, each with its edge
/// on the hull; and the hull's points hashed by their angle about the seed's centre, to find where a new point sees it.
struct Sweep<'a> {
    p: &'a [P2],
    tris: Vec<usize>,
    half: Vec<usize>,
    prev: Vec<usize>,
    next: Vec<usize>,
    hull_tri: Vec<usize>,
    start: usize,
    hash: Vec<usize>,
    centre: P2,
    stack: Vec<usize>,
}

impl Sweep<'_> {
    fn key(&self, q: P2) -> usize {
        let d = q - self.centre;
        let s = d.0.abs() + d.1.abs();
        let k = if s > 0.0 { d.0 / s } else { 0.0 };
        let angle = (if d.1 > 0.0 { 3.0 - k } else { 1.0 + k }) / 4.0;
        let n = self.hash.len();
        ((angle * n as f64).floor().max(0.0) as usize) % n
    }

    fn at(&self, i: usize) -> P2 {
        self.p.get(i).copied().unwrap_or_default()
    }

    /// Whether `q` sees the hull's edge from `a` to `b`: lies to its right, outside.
    fn sees(&self, q: P2, a: usize, b: usize) -> bool {
        let (a, b) = (self.at(a), self.at(b));
        (b - a).cross(q - a) < 0.0
    }

    fn link(&mut self, a: usize, b: usize) {
        if let Some(x) = self.half.get_mut(a) {
            *x = b;
        }
        if let Some(x) = self.half.get_mut(b) {
            *x = a;
        }
    }

    fn add(&mut self, a: usize, b: usize, c: usize, ab: usize, bc: usize, ca: usize) -> usize {
        let t = self.tris.len();
        self.tris.extend([a, b, c]);
        self.half.extend([NONE; 3]);
        self.link(t, ab);
        self.link(t + 1, bc);
        self.link(t + 2, ca);
        t
    }

    fn corner(&self, e: usize) -> usize {
        self.tris.get(e).copied().unwrap_or(0)
    }

    fn twin(&self, e: usize) -> usize {
        self.half.get(e).copied().unwrap_or(NONE)
    }

    /// Point `k` joined to the hull: a triangle with each edge it sees, then the hull goes round it.
    fn insert(&mut self, k: usize) {
        let q = self.at(k);
        let key = self.key(q);
        let n = self.hash.len();
        let mut start = NONE;
        for j in 0..n {
            let s = self.hash.get((key + j) % n).copied().unwrap_or(NONE);
            if s != NONE && self.next.get(s) != Some(&s) {
                start = s;
                break;
            }
        }
        if start == NONE {
            return;
        }
        start = self.prev[start];
        let mut e = start;
        loop {
            let b = self.next[e];
            if self.sees(q, e, b) {
                break;
            }
            e = b;
            if e == start {
                // Inside the hull, or on it: a point on one taken already.
                return;
            }
        }
        let t = self.add(e, k, self.next[e], NONE, NONE, self.hull_tri[e]);
        self.hull_tri[k] = self.legalize(t + 2);
        self.hull_tri[e] = t;
        // On round the hull one way while it sees the edges...
        let mut nn = self.next[e];
        loop {
            let b = self.next[nn];
            if !self.sees(q, nn, b) {
                break;
            }
            let t = self.add(nn, k, b, self.hull_tri[k], NONE, self.hull_tri[nn]);
            self.hull_tri[k] = self.legalize(t + 2);
            self.next[nn] = nn;
            nn = b;
        }
        // ...and the other.
        if e == start {
            loop {
                let a = self.prev[e];
                if !self.sees(q, a, e) {
                    break;
                }
                let t = self.add(a, k, e, NONE, self.hull_tri[e], self.hull_tri[a]);
                self.legalize(t + 2);
                self.hull_tri[a] = t;
                self.next[e] = e;
                e = a;
            }
        }
        self.start = e;
        self.prev[k] = e;
        self.next[e] = k;
        self.prev[nn] = k;
        self.next[k] = nn;
        let (kk, ke) = (self.key(q), self.key(self.at(e)));
        self.hash[kk] = k;
        self.hash[ke] = e;
    }

    /// Edge `a` flipped, and the edges round it after it, while the triangle across one holds the far corner of its own in
    /// its circle. Returns the edge that ends up where `a`'s triangle's edge before `a` was: the hull's, for the caller.
    fn legalize(&mut self, mut a: usize) -> usize {
        let mut ar;
        loop {
            let b = self.twin(a);
            let a0 = a - a % 3;
            ar = a0 + (a + 2) % 3;
            if b == NONE {
                match self.stack.pop() {
                    Some(x) => {
                        a = x;
                        continue;
                    }
                    None => break,
                }
            }
            let b0 = b - b % 3;
            let al = a0 + (a + 1) % 3;
            let bl = b0 + (b + 2) % 3;
            let (p0, pr, pl, p1) = (self.corner(ar), self.corner(a), self.corner(al), self.corner(bl));
            if in_circle(self.at(p0), self.at(pr), self.at(pl), self.at(p1)) {
                if let Some(x) = self.tris.get_mut(a) {
                    *x = p1;
                }
                if let Some(x) = self.tris.get_mut(b) {
                    *x = p0;
                }
                let hbl = self.twin(bl);
                // The edge across the hull changed triangles: the hull points at its new one.
                if hbl == NONE {
                    let mut e = self.start;
                    for _ in 0..self.p.len() {
                        if self.hull_tri.get(e) == Some(&bl) {
                            self.hull_tri[e] = a;
                            break;
                        }
                        e = self.prev.get(e).copied().unwrap_or(self.start);
                        if e == self.start {
                            break;
                        }
                    }
                }
                self.link(a, hbl);
                let har = self.twin(ar);
                self.link(b, har);
                self.link(ar, bl);
                let br = b0 + (b + 1) % 3;
                self.stack.push(br);
            } else {
                match self.stack.pop() {
                    Some(x) => a = x,
                    None => break,
                }
            }
        }
        ar
    }
}

/// Where the plane `f = 0` cuts a surface of triangles (`f` of each corner, by its index): each run of its crossings with the
/// triangles' edges, joined across the triangles that share an edge, as the edge's corners and how far from the first to the
/// second. A corner on the plane counts as above it, so each triangle the plane crosses is crossed through two edges.
pub fn slice(tris: &[[usize; 3]], f: &dyn Fn(usize) -> f64) -> Vec<Vec<(usize, usize, f64)>> {
    type Edge = (usize, usize);
    let mut segs: Vec<[Edge; 2]> = Vec::new();
    for &[a, b, c] in tris {
        let up = [a, b, c].map(|i| f(i) >= 0.0);
        let crossed: Vec<Edge> = [(a, b, up[0], up[1]), (b, c, up[1], up[2]), (c, a, up[2], up[0])]
            .into_iter()
            .filter(|(.., x, y)| x != y)
            .map(|(i, j, ..)| (i.min(j), i.max(j)))
            .collect();
        if let [x, y] = crossed[..] {
            segs.push([x, y]);
        }
    }
    let mut by_edge: HashMap<Edge, Vec<usize>> = HashMap::new();
    for (k, s) in segs.iter().enumerate() {
        for e in s {
            by_edge.entry(*e).or_default().push(k);
        }
    }
    let mut used = vec![false; segs.len()];
    let mut runs = Vec::new();
    for first in 0..segs.len() {
        if used[first] {
            continue;
        }
        used[first] = true;
        let [a, b] = segs[first];
        let mut run: std::collections::VecDeque<Edge> = [a, b].into();
        // On from each end while another segment shares its edge.
        for forward in [true, false] {
            let mut end = if forward { b } else { a };
            while let Some(&k) = by_edge.get(&end).and_then(|v| v.iter().find(|k| !used[**k])) {
                used[k] = true;
                let [x, y] = segs[k];
                end = if x == end { y } else { x };
                if forward {
                    run.push_back(end);
                } else {
                    run.push_front(end);
                }
            }
        }
        let at = |(i, j): Edge| {
            let (fi, fj) = (f(i), f(j));
            let t = if (fi - fj).abs() > 0.0 { fi / (fi - fj) } else { 0.5 };
            (i, j, t.clamp(0.0, 1.0))
        };
        runs.push(run.into_iter().map(at).collect());
    }
    runs
}

/// A terrain's contour lines every `step` mm of height: each height with the lines at it, a closed one ending where it began.
pub fn contours(pts: &[P3], tris: &[[usize; 3]], step: f64) -> Vec<(f64, Vec<Vec<P2>>)> {
    let Some((lo, hi)) =
        pts.iter().map(|p| p[2]).fold(None, |r: Option<(f64, f64)>, z| Some(r.map_or((z, z), |(a, b)| (a.min(z), b.max(z)))))
    else {
        return Vec::new();
    };
    if !(step.is_finite() && step > 0.0) {
        return Vec::new();
    }
    let (first, last) = ((lo / step).ceil() as i64, (hi / step).floor() as i64);
    // Each height's triangles: those it passes through.
    let mut at: std::collections::BTreeMap<i64, Vec<[usize; 3]>> = std::collections::BTreeMap::new();
    for t in tris {
        let zs = t.map(|i| pts.get(i).map_or(0.0, |p| p[2]));
        let (a, b) = (zs.iter().copied().fold(f64::INFINITY, f64::min), zs.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        for k in ((a / step).ceil() as i64).max(first)..=((b / step).floor() as i64).min(last) {
            at.entry(k).or_default().push(*t);
        }
    }
    let point = |i: usize| pts.get(i).copied().unwrap_or_default();
    at.into_iter()
        .map(|(k, tris)| {
            let z = k as f64 * step;
            let lines = slice(&tris, &|i| point(i)[2] - z)
                .into_iter()
                .map(|run| {
                    run.into_iter()
                        .map(|(i, j, t)| {
                            let (a, b) = (point(i), point(j));
                            P2(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
                        })
                        .collect()
                })
                .collect();
            (z, lines)
        })
        .collect()
}

/// Where a section along unit `u` from `p`, `len` long, cuts a terrain: its ground lines, as (along, up), each from where it
/// begins along the line to where it ends.
pub fn profile(pts: &[P3], tris: &[[usize; 3]], p: P2, u: P2, len: f64) -> Vec<Vec<P2>> {
    let point = |i: usize| pts.get(i).copied().unwrap_or_default();
    let side = |i: usize| (P2(point(i)[0], point(i)[1]) - p).dot(u.perp());
    // Only the triangles near the line.
    let near: Vec<[usize; 3]> = tris
        .iter()
        .filter(|t| {
            let s = t.map(side);
            s.iter().any(|x| *x >= 0.0) && s.iter().any(|x| *x < 0.0)
        })
        .copied()
        .collect();
    slice(&near, &side)
        .into_iter()
        .flat_map(|run| {
            let mut line: Vec<P2> = run
                .into_iter()
                .map(|(i, j, t)| {
                    let (a, b) = (point(i), point(j));
                    let q = P2(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
                    P2((q - p).dot(u), a[2] + (b[2] - a[2]) * t)
                })
                .collect();
            if line.first().zip(line.last()).is_some_and(|(a, b)| a.0 > b.0) {
                line.reverse();
            }
            clip_along(&line, 0.0, len)
        })
        .collect()
}

/// The pieces of a line, as (along, up), between `from` and `to` along.
fn clip_along(line: &[P2], from: f64, to: f64) -> Vec<Vec<P2>> {
    let mut out: Vec<Vec<P2>> = Vec::new();
    let mut run: Vec<P2> = Vec::new();
    for w in line.windows(2) {
        let (Some(&a), Some(&b)) = (w.first(), w.get(1)) else { continue };
        let (lo, hi) = (a.0.min(b.0), a.0.max(b.0));
        if hi < from || lo > to || hi - lo <= 0.0 {
            if hi - lo > 0.0 && !run.is_empty() {
                out.push(std::mem::take(&mut run));
            }
            continue;
        }
        let at = |x: f64| a + (b - a) * ((x - a.0) / (b.0 - a.0));
        let (s, e) = (if a.0 < from { at(from) } else { a }, if b.0 > to { at(to) } else { b });
        if run.last() != Some(&s) {
            if !run.is_empty() {
                out.push(std::mem::take(&mut run));
            }
            run.push(s);
        }
        run.push(e);
    }
    if run.len() >= 2 {
        out.push(run);
    }
    out.retain(|r| r.len() >= 2);
    out
}

/// The edge of a surface of triangles: its loops, round the edges only one triangle has.
pub fn edge(pts: &[P3], tris: &[[usize; 3]]) -> Vec<Vec<P2>> {
    let mut count: HashMap<(usize, usize), (usize, usize, usize)> = HashMap::new();
    for &[a, b, c] in tris {
        for (i, j) in [(a, b), (b, c), (c, a)] {
            let e = count.entry((i.min(j), i.max(j))).or_insert((i, j, 0));
            e.2 += 1;
        }
    }
    let mut from: HashMap<usize, usize> = count.values().filter(|e| e.2 == 1).map(|&(i, j, _)| (i, j)).collect();
    let point = |i: usize| pts.get(i).map_or(P2(0.0, 0.0), |p| P2(p[0], p[1]));
    let mut loops = Vec::new();
    while let Some((&s, _)) = from.iter().next() {
        let mut ring = vec![point(s)];
        let mut at = s;
        while let Some(next) = from.remove(&at) {
            if next == s {
                break;
            }
            ring.push(point(next));
            at = next;
        }
        if ring.len() >= 3 {
            loops.push(ring);
        }
    }
    loops
}

/// The bearing of a line from `a` to `b` as a survey writes it, from north or south toward east or west, in degrees,
/// minutes and seconds (`N 45°30'00" E`); and its azimuth, degrees clockwise from north.
pub fn bearing(a: P2, b: P2) -> (String, f64) {
    let d = b - a;
    let azimuth = (d.0.atan2(d.1).to_degrees() + 360.0) % 360.0;
    let seconds = (azimuth * 3600.0).round() as i64 % (360 * 3600);
    let az = seconds as f64 / 3600.0;
    let (ns, angle, ew) = match seconds {
        s if s <= 90 * 3600 => ("N", s, "E"),
        s if s <= 180 * 3600 => ("S", 180 * 3600 - s, "E"),
        s if s <= 270 * 3600 => ("S", s - 180 * 3600, "W"),
        s => ("N", 360 * 3600 - s, "W"),
    };
    (format!("{ns} {}°{:02}'{:02}\" {ew}", angle / 3600, angle / 60 % 60, angle % 60), az)
}

/// The middle of a ring's area, where its label goes; its first point where it has none.
pub fn centroid(ring: &[P2]) -> P2 {
    let a = geom::area(ring);
    if a.abs() < 1e-9 {
        return ring.first().copied().unwrap_or_default();
    }
    let (mut x, mut y) = (0.0, 0.0);
    for (p, q) in geom::ring_edges(ring) {
        let c = p.cross(q);
        x += (p.0 + q.0) * c;
        y += (p.1 + q.1) * c;
    }
    P2(x / (6.0 * a), y / (6.0 * a))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(nx: usize, ny: usize, step: f64) -> Vec<P2> {
        (0..ny).flat_map(|j| (0..nx).map(move |i| P2(i as f64 * step, j as f64 * step))).collect()
    }

    /// Whether every triangle's circle is clear of every point, but for those on it.
    fn delaunay_ok(p: &[P2], tris: &[[usize; 3]]) -> bool {
        tris.iter().all(|&[a, b, c]| (0..p.len()).filter(|i| ![a, b, c].contains(i)).all(|d| !in_circle(p[a], p[b], p[c], p[d])))
    }

    #[test]
    fn a_grid_of_points_is_cut_into_two_triangles_a_cell_covering_it_once() {
        let p = grid(5, 4, 1000.0);
        let t = delaunay(&p);
        assert_eq!(t.len(), 2 * 4 * 3);
        let area: f64 = t.iter().map(|&[a, b, c]| (p[b] - p[a]).cross(p[c] - p[a]) / 2.0).sum();
        assert!((area - 4000.0 * 3000.0).abs() < 1e-6, "{area}");
        assert!(t.iter().all(|&[a, b, c]| (p[b] - p[a]).cross(p[c] - p[a]) > 0.0), "counter-clockwise");
        assert!(delaunay_ok(&p, &t));
    }

    #[test]
    fn scattered_points_make_a_delaunay_triangulation_of_their_hull() {
        // Points of a fixed scatter, the same every run.
        let mut seed = 12345_u64;
        let mut next = || {
            seed = crate::model::mix(seed);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let p: Vec<P2> = (0..400).map(|_| P2(next() * 50_000.0, next() * 30_000.0)).collect();
        let t = delaunay(&p);
        let hull = geom::hull(p.clone());
        // Euler: 2n - 2 - h triangles for n points, h of them on the hull.
        assert_eq!(t.len(), 2 * p.len() - 2 - hull.len());
        assert!(delaunay_ok(&p, &t));
        let area: f64 = t.iter().map(|&[a, b, c]| (p[b] - p[a]).cross(p[c] - p[a]) / 2.0).sum();
        assert!((area - geom::area(&hull).abs()).abs() < 1e-3 * area, "{area}");
    }

    #[test]
    fn too_few_points_a_line_of_them_or_twins_make_no_triangles_and_twins_one_set() {
        assert!(delaunay(&[]).is_empty());
        assert!(delaunay(&[P2(0.0, 0.0), P2(1.0, 0.0)]).is_empty());
        assert!(delaunay(&[P2(0.0, 0.0), P2(1.0, 0.0), P2(2.0, 0.0), P2(3.0, 0.0)]).is_empty());
        assert!(delaunay(&[P2(0.0, 0.0), P2(f64::NAN, 0.0), P2(0.0, 1.0)]).is_empty());
        let twins = [P2(0.0, 0.0), P2(1000.0, 0.0), P2(0.0, 1000.0), P2(1000.0, 0.0), P2(1000.0, 1000.0)];
        assert_eq!(delaunay(&twins).len(), 2);
    }

    #[test]
    fn contours_cross_a_slope_at_each_height_and_a_section_cuts_its_ground_line() {
        // A slope rising 1 m every 10 m to the east, 40 m by 20 m.
        let p = grid(5, 3, 10_000.0);
        let pts: Vec<P3> = p.iter().map(|q| [q.0, q.1, q.0 / 10.0]).collect();
        let t = delaunay(&p);
        let c = contours(&pts, &t, 1000.0);
        assert_eq!(c.iter().map(|(z, _)| *z).collect::<Vec<_>>(), [0.0, 1000.0, 2000.0, 3000.0, 4000.0]);
        let (_, at2) = &c[2];
        assert_eq!(at2.len(), 1, "one line at 2 m");
        assert!(at2[0].iter().all(|q| (q.0 - 20_000.0).abs() < 1e-6), "{at2:?}");
        // A section east along y = 5 m from x = 5 m, 30 m long: its ground rises from 0.5 m to 3.5 m.
        let g = profile(&pts, &t, P2(5000.0, 5000.0), P2(1.0, 0.0), 30_000.0);
        assert_eq!(g.len(), 1, "{g:?}");
        let (a, b) = (g[0][0], *g[0].last().unwrap());
        assert!((a.0 - 0.0).abs() < 1e-6 && (a.1 - 500.0).abs() < 1e-6, "{a:?}");
        assert!((b.0 - 30_000.0).abs() < 1e-6 && (b.1 - 3500.0).abs() < 1e-6, "{b:?}");
        // Its edge is the rectangle round it.
        let e = edge(&pts, &t);
        assert_eq!(e.len(), 1);
        assert!((geom::area(&e[0]).abs() - 40_000.0 * 20_000.0).abs() < 1e-3);
    }

    #[test]
    fn bearings_read_from_north_or_south_toward_east_or_west() {
        let o = P2(0.0, 0.0);
        assert_eq!(bearing(o, P2(1.0, 1.0)), ("N 45°00'00\" E".to_string(), 45.0));
        assert_eq!(bearing(o, P2(1.0, -1.0)).0, "S 45°00'00\" E");
        assert_eq!(bearing(o, P2(-1.0, -2.0)).0, "S 26°33'54\" W");
        assert_eq!(bearing(o, P2(-1.0, 1.0)), ("N 45°00'00\" W".to_string(), 315.0));
        assert_eq!(bearing(o, P2(0.0, -5.0)).0, "S 0°00'00\" E");
        assert_eq!(bearing(o, P2(-5.0, 0.0)).0, "S 90°00'00\" W");
    }
}
