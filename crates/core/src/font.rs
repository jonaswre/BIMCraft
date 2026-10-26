//! Our own single-stroke lettering for plan annotation: capitals, digits, the marks of ASCII and
//! the typographic ones drawings use (primes, dashes, the ellipsis, degrees, powers, quotes, the
//! euro), German's ß and the accents of Latin languages, drawn on an 8 x 12 grid (y up). Lower case
//! is drawn as capitals, as architectural lettering is. A character it has no glyph for is a box
//! crossed corner to corner, which reads as no letter; [`missing`] names them, so a command that
//! stores text drawings show can warn.

type Glyph = &'static [&'static [(i8, i8)]];

const O: &[(i8, i8)] = &[(2, 0), (0, 2), (0, 10), (2, 12), (6, 12), (8, 10), (8, 2), (6, 0), (2, 0)];
const P: &[(i8, i8)] = &[(0, 0), (0, 12), (6, 12), (8, 10), (8, 8), (6, 6), (0, 6)];
const S: &[(i8, i8)] = &[(8, 10), (6, 12), (2, 12), (0, 10), (0, 8), (2, 6), (6, 6), (8, 4), (8, 2), (6, 0), (2, 0), (0, 2)];
/// What a character with no glyph is drawn as: a box crossed corner to corner, unlike any letter (never a `?`, which
/// small reads as a 7).
const MISSING: Glyph = &[&[(0, 0), (8, 0), (8, 12), (0, 12), (0, 0)], &[(0, 0), (8, 12)], &[(0, 12), (8, 0)]];

fn glyph(c: char) -> Option<Glyph> {
    Some(match c {
        'A' => &[&[(0, 0), (4, 12), (8, 0)], &[(1, 3), (7, 3)]],
        'B' => &[&[(0, 0), (0, 12), (6, 12), (8, 10), (8, 8), (6, 6), (0, 6)], &[(6, 6), (8, 4), (8, 2), (6, 0), (0, 0)]],
        'C' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2)]],
        'D' => &[&[(0, 0), (0, 12), (5, 12), (8, 9), (8, 3), (5, 0), (0, 0)]],
        'E' => &[&[(8, 12), (0, 12), (0, 0), (8, 0)], &[(0, 6), (6, 6)]],
        'F' => &[&[(8, 12), (0, 12), (0, 0)], &[(0, 6), (6, 6)]],
        'G' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2), (8, 5), (5, 5)]],
        'H' => &[&[(0, 0), (0, 12)], &[(8, 0), (8, 12)], &[(0, 6), (8, 6)]],
        'I' => &[&[(2, 12), (6, 12)], &[(4, 12), (4, 0)], &[(2, 0), (6, 0)]],
        'J' => &[&[(8, 12), (8, 2), (6, 0), (2, 0), (0, 2)]],
        'K' => &[&[(0, 0), (0, 12)], &[(8, 12), (0, 4)], &[(3, 7), (8, 0)]],
        'L' => &[&[(0, 12), (0, 0), (8, 0)]],
        'M' => &[&[(0, 0), (0, 12), (4, 6), (8, 12), (8, 0)]],
        'N' => &[&[(0, 0), (0, 12), (8, 0), (8, 12)]],
        'O' => &[O],
        'P' => &[P],
        'Q' => &[O, &[(5, 3), (8, 0)]],
        'R' => &[P, &[(4, 6), (8, 0)]],
        'S' => &[S],
        'T' => &[&[(0, 12), (8, 12)], &[(4, 12), (4, 0)]],
        'U' => &[&[(0, 12), (0, 2), (2, 0), (6, 0), (8, 2), (8, 12)]],
        'V' => &[&[(0, 12), (4, 0), (8, 12)]],
        'W' => &[&[(0, 12), (2, 0), (4, 6), (6, 0), (8, 12)]],
        'X' => &[&[(0, 0), (8, 12)], &[(0, 12), (8, 0)]],
        'Y' => &[&[(0, 12), (4, 6), (8, 12)], &[(4, 6), (4, 0)]],
        'Z' => &[&[(0, 12), (8, 12), (0, 0), (8, 0)]],
        '0' => &[&[(2, 0), (1, 2), (1, 10), (2, 12), (6, 12), (7, 10), (7, 2), (6, 0), (2, 0)]],
        '1' => &[&[(2, 10), (4, 12), (4, 0)], &[(2, 0), (6, 0)]],
        '2' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (0, 0), (8, 0)]],
        '3' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (6, 6), (8, 4), (8, 2), (6, 0), (2, 0), (0, 2)], &[(3, 6), (6, 6)]],
        '4' => &[&[(6, 0), (6, 12), (0, 4), (8, 4)]],
        '5' => &[&[(8, 12), (0, 12), (0, 7), (6, 7), (8, 5), (8, 2), (6, 0), (2, 0), (0, 2)]],
        '6' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2), (8, 5), (6, 7), (2, 7), (0, 5)]],
        '7' => &[&[(0, 12), (8, 12), (3, 0)]],
        '8' => &[&[
            (2, 6),
            (0, 8),
            (0, 10),
            (2, 12),
            (6, 12),
            (8, 10),
            (8, 8),
            (6, 6),
            (2, 6),
            (0, 4),
            (0, 2),
            (2, 0),
            (6, 0),
            (8, 2),
            (8, 4),
            (6, 6),
        ]],
        '9' => &[&[(0, 2), (2, 0), (6, 0), (8, 2), (8, 10), (6, 12), (2, 12), (0, 10), (0, 7), (2, 5), (6, 5), (8, 7)]],
        ' ' => &[],
        '.' => &[&[(3, 0), (4, 0), (4, 1), (3, 1), (3, 0)]],
        ',' => &[&[(4, 1), (4, 0), (3, -2)]],
        '-' => &[&[(1, 6), (7, 6)]],
        '+' => &[&[(4, 2), (4, 10)], &[(0, 6), (8, 6)]],
        '=' => &[&[(1, 4), (7, 4)], &[(1, 8), (7, 8)]],
        '_' => &[&[(0, -1), (8, -1)]],
        '/' => &[&[(0, 0), (8, 12)]],
        '(' => &[&[(6, 12), (4, 9), (4, 3), (6, 0)]],
        '{' => &[&[(6, 12), (4, 11), (4, 7), (2, 6), (4, 5), (4, 1), (6, 0)]],
        '}' => &[&[(2, 12), (4, 11), (4, 7), (6, 6), (4, 5), (4, 1), (2, 0)]],
        ')' => &[&[(2, 12), (4, 9), (4, 3), (2, 0)]],
        ':' => &[&[(4, 2), (4, 3)], &[(4, 8), (4, 9)]],
        '\'' => &[&[(4, 12), (4, 9)]],
        '"' => &[&[(3, 12), (3, 9)], &[(5, 12), (5, 9)]],
        '#' => &[&[(2, 0), (3, 12)], &[(5, 0), (6, 12)], &[(0, 4), (8, 4)], &[(0, 8), (8, 8)]],
        '×' => &[&[(1, 2), (7, 10)], &[(1, 10), (7, 2)]],
        '²' => &[&[(2, 11), (3, 12), (5, 12), (6, 11), (6, 10), (2, 7), (6, 7)]],
        '°' => &[&[(3, 12), (5, 12), (6, 11), (6, 9), (5, 8), (3, 8), (2, 9), (2, 11), (3, 12)]],
        '±' => &[&[(4, 5), (4, 11)], &[(1, 8), (7, 8)], &[(1, 2), (7, 2)]],
        '?' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (4, 5), (4, 3)], &[(4, 1), (4, 0)]],
        '!' => &[&[(4, 12), (4, 4)], &[(4, 1), (4, 0)]],
        '%' => &[&[(0, 0), (8, 12)], &[(0, 12), (2, 12), (2, 9), (0, 9), (0, 12)], &[(6, 3), (8, 3), (8, 0), (6, 0), (6, 3)]],
        '&' => &[&[(8, 0), (1, 9), (2, 12), (4, 12), (5, 10), (0, 4), (0, 2), (2, 0), (5, 0), (8, 4)]],
        '*' => &[&[(4, 3), (4, 11)], &[(1, 5), (7, 9)], &[(1, 9), (7, 5)]],
        '<' => &[&[(7, 10), (1, 6), (7, 2)]],
        '>' => &[&[(1, 10), (7, 6), (1, 2)]],
        '[' => &[&[(6, 12), (3, 12), (3, 0), (6, 0)]],
        ']' => &[&[(2, 12), (5, 12), (5, 0), (2, 0)]],
        ';' => &[&[(4, 8), (4, 9)], &[(4, 2), (4, 1), (3, -2)]],
        '@' => {
            &[&[(6, 4), (6, 8), (3, 8), (2, 7), (2, 5), (3, 4), (6, 4), (8, 6), (8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (7, 0)]]
        }
        '\\' => &[&[(0, 12), (8, 0)]],
        '|' => &[&[(4, -2), (4, 14)]],
        '~' => &[&[(0, 5), (2, 7), (6, 5), (8, 7)]],
        '^' => &[&[(2, 9), (4, 12), (6, 9)]],
        '`' => &[&[(3, 12), (5, 10)]],
        '$' => &[S, &[(4, 14), (4, -2)]],
        // Primes lean, so a secondary grid's 1′ is neither an apostrophe's 1' nor a 17.
        '′' => &[&[(5, 12), (3, 8)]],
        '″' => &[&[(3, 12), (1, 8)], &[(7, 12), (5, 8)]],
        '‘' => &[&[(5, 8), (4, 10), (4, 12)]],
        '’' => &[&[(4, 12), (4, 10), (3, 8)]],
        '‚' => &[&[(4, 1), (4, 0), (3, -2)]],
        '“' => &[&[(3, 8), (2, 10), (2, 12)], &[(7, 8), (6, 10), (6, 12)]],
        '”' => &[&[(2, 12), (2, 10), (1, 8)], &[(6, 12), (6, 10), (5, 8)]],
        '„' => &[&[(2, 1), (2, 0), (1, -2)], &[(6, 1), (6, 0), (5, -2)]],
        '«' => &[&[(4, 10), (1, 6), (4, 2)], &[(8, 10), (5, 6), (8, 2)]],
        '»' => &[&[(0, 10), (3, 6), (0, 2)], &[(4, 10), (7, 6), (4, 2)]],
        '–' => &[&[(0, 6), (8, 6)]],
        '—' => &[&[(-1, 6), (10, 6)]],
        '…' => &[
            &[(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)],
            &[(3, 0), (4, 0), (4, 1), (3, 1), (3, 0)],
            &[(7, 0), (8, 0), (8, 1), (7, 1), (7, 0)],
        ],
        '·' => &[&[(3, 6), (4, 6), (4, 7), (3, 7), (3, 6)]],
        '¹' => &[&[(3, 11), (4, 12), (4, 7)], &[(3, 7), (5, 7)]],
        '³' => &[&[(2, 12), (6, 12), (4, 10), (6, 9), (6, 8), (5, 7), (2, 7)]],
        'ß' | 'ẞ' => &[&[(0, 0), (0, 10), (2, 12), (5, 12), (7, 10), (7, 8), (5, 6), (8, 4), (8, 2), (6, 0), (3, 0)]],
        'Ø' => &[O, &[(0, 0), (8, 12)]],
        '€' => &[&[(8, 10), (6, 12), (3, 12), (1, 10), (1, 2), (3, 0), (6, 0), (8, 2)], &[(0, 7), (5, 7)], &[(0, 5), (5, 5)]],
        _ => return None,
    })
}

/// An accented capital as its base letter and accent.
fn accent(c: char) -> Option<(char, Glyph)> {
    const ACUTE: Glyph = &[&[(3, 13), (5, 15)]];
    const GRAVE: Glyph = &[&[(3, 15), (5, 13)]];
    const CIRCUMFLEX: Glyph = &[&[(2, 13), (4, 15), (6, 13)]];
    const TILDE: Glyph = &[&[(1, 13), (3, 15), (5, 13), (7, 15)]];
    const DIAERESIS: Glyph = &[&[(2, 14), (2, 15)], &[(6, 14), (6, 15)]];
    const CEDILLA: Glyph = &[&[(4, 0), (5, -1), (3, -3)]];
    const RING: Glyph = &[&[(3, 14), (4, 13), (5, 14), (4, 15), (3, 14)]];
    const CARON: Glyph = &[&[(2, 15), (4, 13), (6, 15)]];
    Some(match c {
        'Á' | 'É' | 'Í' | 'Ó' | 'Ú' | 'Ý' => (base(c)?, ACUTE),
        'À' | 'È' | 'Ì' | 'Ò' | 'Ù' => (base(c)?, GRAVE),
        'Â' | 'Ê' | 'Î' | 'Ô' | 'Û' => (base(c)?, CIRCUMFLEX),
        'Ã' | 'Õ' | 'Ñ' => (base(c)?, TILDE),
        'Ä' | 'Ë' | 'Ï' | 'Ö' | 'Ü' | 'Ÿ' => (base(c)?, DIAERESIS),
        'Ç' => ('C', CEDILLA),
        'Å' => ('A', RING),
        'Č' | 'Ď' | 'Ě' | 'Ň' | 'Ř' | 'Š' | 'Ť' | 'Ž' => (base(c)?, CARON),
        _ => return None,
    })
}

fn base(c: char) -> Option<char> {
    Some(match c {
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'Č' => 'C',
        'Ď' => 'D',
        'É' | 'È' | 'Ê' | 'Ë' | 'Ě' => 'E',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'Ñ' | 'Ň' => 'N',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'Ř' => 'R',
        'Š' => 'S',
        'Ť' => 'T',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'Ý' | 'Ÿ' => 'Y',
        'Ž' => 'Z',
        _ => return None,
    })
}

/// The capital `c` is drawn as: itself when it has no single capital (ß, whose capital is SS), so each character is one glyph.
fn capital(c: char) -> char {
    let mut up = c.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

/// Whether `c` is layout, not lettering: a space, a line break, a tab.
fn blank(c: char) -> bool {
    c.is_whitespace() || c.is_control()
}

/// The characters of `text` the lettering cannot draw, each once, in the order they come: drawings show each as a crossed box.
pub fn missing(text: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for c in text.chars() {
        let up = capital(c);
        if !blank(up) && accent(up).is_none() && glyph(up).is_none() && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// Grid units from one character to the next.
const ADVANCE: f32 = 11.0;

/// How wide `text` is at `height` px.
pub fn width(text: &str, height: f32) -> f32 {
    let n = text.chars().count() as f32;
    if n == 0.0 { 0.0 } else { (n * ADVANCE - 3.0) * height / 12.0 }
}

/// Polylines drawing `text` with capitals `height` px tall, from `(x, y)` on the baseline, y down.
pub fn strokes(text: &str, x: f32, y: f32, height: f32) -> Vec<Vec<(f32, f32)>> {
    let k = height / 12.0;
    let mut out = Vec::new();
    for (i, c) in text.chars().map(capital).enumerate() {
        let x0 = x + i as f32 * ADVANCE * k;
        let (letter, mark): (Glyph, Glyph) = match accent(c) {
            Some((b, m)) => (glyph(b).unwrap_or(MISSING), m),
            None if blank(c) => (&[], &[]),
            None => (glyph(c).unwrap_or(MISSING), &[]),
        };
        for line in letter.iter().chain(mark) {
            out.push(line.iter().map(|&(gx, gy)| (x0 + f32::from(gx) * k, y - f32::from(gy) * k)).collect());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_case_is_drawn_as_capitals_and_unknown_characters_as_a_crossed_box() {
        assert_eq!(strokes("living", 0.0, 0.0, 12.0), strokes("LIVING", 0.0, 0.0, 12.0));
        assert!(strokes(" ", 0.0, 0.0, 12.0).is_empty());
        // A character the lettering has no glyph for is a box crossed corner to corner: never a question mark, nor a 7.
        let unknown = strokes("☃", 0.0, 0.0, 12.0);
        assert_ne!(unknown, strokes("?", 0.0, 0.0, 12.0));
        assert_ne!(unknown, strokes("7", 0.0, 0.0, 12.0));
        assert_eq!(unknown, strokes("\u{1F600}", 0.0, 0.0, 12.0), "every unknown character draws the same box");
        assert_eq!(unknown.len(), 3, "its outline and two diagonals");
    }

    #[test]
    fn the_typographic_marks_of_drawings_are_drawn_each_its_own() {
        // Primes for secondary grids, dashes, the ellipsis, German letters, degrees, powers, the euro and the rest of ASCII.
        let set = "′″–—…·ÄÖÜäöüßẞ°±²³¹×ØøÅåŠšČčŽžŸÿ€?!%&*<>[];@\\|~^`$‘’‚“”„«»";
        let unknown = strokes("☃", 0.0, 0.0, 12.0);
        for c in set.chars() {
            let s = c.to_string();
            assert_ne!(strokes(&s, 0.0, 0.0, 12.0), unknown, "{c} (U+{:04X}) has a glyph of its own", u32::from(c));
            assert!(missing(&s).is_empty(), "{c} is drawable");
        }
        // A prime is not an apostrophe nor a 7, so grid 1′ reads 1′.
        assert_ne!(strokes("′", 0.0, 0.0, 12.0), strokes("'", 0.0, 0.0, 12.0));
        assert_ne!(strokes("1′", 0.0, 0.0, 12.0), strokes("17", 0.0, 0.0, 12.0));
        // ß stays one letter, as wide as the text says it is.
        assert_eq!(strokes("ß", 0.0, 0.0, 12.0).iter().flatten().map(|p| p.0).fold(f32::MIN, f32::max), 8.0);
    }

    #[test]
    fn missing_names_each_character_the_lettering_cannot_draw_once() {
        assert!(missing("Straße 1′–3″, 12 m², ±0.00").is_empty());
        assert!(missing("line one\nline two\t").is_empty(), "line breaks and tabs aren't drawn, they're layout");
        assert_eq!(missing("1☃ 2☃ 3✓"), vec!['☃', '✓']);
    }

    #[test]
    fn accented_letters_are_their_base_letter_plus_an_accent() {
        let plain = strokes("A", 0.0, 0.0, 12.0);
        let acute = strokes("á", 0.0, 0.0, 12.0);
        assert_eq!(acute.len(), plain.len() + 1);
        assert_eq!(acute[..plain.len()], plain[..]);
    }

    #[test]
    fn text_is_scaled_to_its_height_from_the_baseline() {
        let s = strokes("I", 10.0, 100.0, 24.0);
        let ys: Vec<f32> = s.iter().flatten().map(|p| p.1).collect();
        assert_eq!(ys.iter().copied().fold(f32::MAX, f32::min), 76.0);
        assert_eq!(ys.iter().copied().fold(f32::MIN, f32::max), 100.0);
        assert_eq!(width("II", 12.0), 19.0);
    }
}
