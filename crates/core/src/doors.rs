//! Door kinds: how a project door type's leaf opens. A swing door (one leaf, or two meeting in the middle) swings; a sliding
//! door (one leaf, or two parting) runs along the wall's face; a sectional door's panels roll up under the ceiling on tracks;
//! a roller shutter's curtain winds onto a coil over the opening. Each kind's parts are boxes in the opening's frame, which
//! the 3D view builds, a plan draws (cut, seen or overhead) and IFC writes, and each kind has IFC's operation type.
//!
//! What sits off the wall (a sliding leaf, a sectional door and its tracks, a roller's coil box) is on the side the door
//! faces: the side a swing door swings to, its exterior unless the door is flipped.

use serde::{Deserialize, Serialize};

/// How a door type's leaf opens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// One leaf hung on a jamb, swinging.
    #[default]
    Swing,
    /// Two leaves hung on the two jambs, meeting in the middle.
    DoubleSwing,
    /// One leaf running along the wall's face, beside the opening when open.
    Sliding,
    /// Two leaves parting from the middle, each beside the opening when open.
    DoubleSliding,
    /// An overhead sectional door: panels hinged to each other that run up and under the ceiling on tracks.
    Sectional,
    /// A roller shutter: a curtain of slats that winds onto a coil in a box over the opening.
    Roller,
}

impl Operation {
    pub const ALL: [Operation; 6] =
        [Operation::Swing, Operation::DoubleSwing, Operation::Sliding, Operation::DoubleSliding, Operation::Sectional, Operation::Roller];

    /// Its name, as a file, a command and a schedule write it.
    pub fn word(self) -> &'static str {
        match self {
            Operation::Swing => "swing",
            Operation::DoubleSwing => "double_swing",
            Operation::Sliding => "sliding",
            Operation::DoubleSliding => "double_sliding",
            Operation::Sectional => "sectional",
            Operation::Roller => "roller",
        }
    }

    /// The kind a word names, as `word` writes it, in any case, with spaces or hyphens for its underscores.
    pub fn parse(s: &str) -> Option<Operation> {
        let w = s.trim().to_lowercase().replace([' ', '-'], "_");
        Operation::ALL.into_iter().find(|o| o.word() == w)
    }

    /// Whether it is the plain swing door, which a file needn't say.
    pub fn is_swing(&self) -> bool {
        *self == Operation::Swing
    }

    /// Its IFC4 operation type (`IfcDoorTypeOperationEnum`) for a door hung on its start's jamb, or `flipped` its end's;
    /// and, where IFC has none (a sectional door), the user-defined operation it names.
    pub fn ifc(self, flipped: bool) -> (&'static str, Option<&'static str>) {
        let hand = |left, right| if flipped { right } else { left };
        match self {
            Operation::Swing => (hand(".SINGLE_SWING_LEFT.", ".SINGLE_SWING_RIGHT."), None),
            Operation::DoubleSwing => (".DOUBLE_DOOR_SINGLE_SWING.", None),
            Operation::Sliding => (hand(".SLIDING_TO_LEFT.", ".SLIDING_TO_RIGHT."), None),
            Operation::DoubleSliding => (".DOUBLE_DOOR_SLIDING.", None),
            Operation::Sectional => (".USERDEFINED.", Some("SECTIONAL")),
            Operation::Roller => (".ROLLINGUP.", None),
        }
    }

    /// The kind an IFC operation type (`op`, without its dots) and user-defined operation say, and whether it is hung on
    /// the right; none for one BIMCraft draws as a swing door.
    pub fn from_ifc(op: &str, user: Option<&str>) -> (Operation, bool) {
        let right = op.contains("RIGHT");
        let kind = match op {
            "DOUBLE_DOOR_SINGLE_SWING"
            | "DOUBLE_DOOR_DOUBLE_SWING"
            | "DOUBLE_DOOR_SINGLE_SWING_OPPOSITE_LEFT"
            | "DOUBLE_DOOR_SINGLE_SWING_OPPOSITE_RIGHT" => Operation::DoubleSwing,
            "SLIDING_TO_LEFT" | "SLIDING_TO_RIGHT" => Operation::Sliding,
            "DOUBLE_DOOR_SLIDING" => Operation::DoubleSliding,
            "ROLLINGUP" => Operation::Roller,
            "USERDEFINED" if user.is_some_and(|u| u.to_uppercase().contains("SECTIONAL")) => Operation::Sectional,
            _ => Operation::Swing,
        };
        (kind, right)
    }

    /// Its parts, closed, as boxes in the opening's frame: `along` the wall from the opening's middle, `across` it from the
    /// wall's centre line (positive to its left), up from the sill; for an opening `width` by `height` in a wall whose face
    /// on the side the door faces is `face` across, `out` (1 or -1) the way out of the wall there.
    pub fn parts(self, width: f64, height: f64, face: f64, out: f64) -> Vec<Part> {
        let (w, h) = (width / 2.0, height);
        // A band `a` to `b` out from the face.
        let off = |a: f64, b: f64| {
            let (p, q) = (face + out * a, face + out * b);
            (p.min(q), p.max(q))
        };
        let part = |along: (f64, f64), across: (f64, f64), z: (f64, f64), role: Role| Part { along, across, z, role };
        match self {
            Operation::Swing => vec![part((-w, w), (-25.0, 25.0), (0.0, h), Role::Leaf)],
            Operation::DoubleSwing => {
                vec![part((-w, -2.0), (-25.0, 25.0), (0.0, h), Role::Leaf), part((2.0, w), (-25.0, 25.0), (0.0, h), Role::Leaf)]
            }
            // Lapping the wall 50 mm at each side and over the head.
            Operation::Sliding => vec![part((-w - 50.0, w + 50.0), off(10.0, 50.0), (0.0, h + 50.0), Role::Leaf)],
            Operation::DoubleSliding => vec![
                part((-w - 50.0, 0.0), off(10.0, 50.0), (0.0, h + 50.0), Role::Leaf),
                part((0.0, w + 50.0), off(10.0, 50.0), (0.0, h + 50.0), Role::Leaf),
            ],
            // Panels about 600 high one over another, guide rails up the jambs and on, level, under the ceiling, as far in as
            // the door is high.
            Operation::Sectional => {
                let n = (h / 600.0).round().max(1.0);
                let each = h / n;
                let mut out: Vec<Part> = (0..n as usize)
                    .map(|i| {
                        let z0 = i as f64 * each;
                        part((-w - 40.0, w + 40.0), off(20.0, 65.0), (z0 + 5.0, z0 + each - 5.0), Role::Leaf)
                    })
                    .collect();
                for side in [-1.0, 1.0] {
                    let along = if side < 0.0 { (-w - 100.0, -w - 40.0) } else { (w + 40.0, w + 100.0) };
                    out.push(part(along, off(0.0, 80.0), (0.0, h + 100.0), Role::Rail));
                    out.push(part(along, off(80.0, 80.0 + h), (h + 50.0, h + 100.0), Role::Track));
                }
                out
            }
            // A thin curtain in guides at the jambs, its coil in a box 450 square over the opening.
            Operation::Roller => vec![
                part((-w - 30.0, w + 30.0), off(30.0, 45.0), (0.0, h), Role::Leaf),
                part((-w - 100.0, -w - 30.0), off(0.0, 80.0), (0.0, h), Role::Rail),
                part((w + 30.0, w + 100.0), off(0.0, 80.0), (0.0, h), Role::Rail),
                part((-w - 100.0, w + 100.0), off(0.0, 450.0), (h, h + 450.0), Role::Coil),
            ],
        }
    }
}

/// What a part of a door is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// A leaf, a panel or a curtain: what closes the opening.
    Leaf,
    /// A guide rail up a jamb.
    Rail,
    /// A sectional door's track under the ceiling, where it runs open.
    Track,
    /// A roller shutter's coil box over the opening.
    Coil,
}

/// A part of a door: a box `along` the wall from the opening's middle, `across` it from the wall's centre line, and up
/// from the sill (`z`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub along: (f64, f64),
    pub across: (f64, f64),
    pub z: (f64, f64),
    pub role: Role,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_reads_back_from_its_word_and_its_ifc_operation() {
        for o in Operation::ALL {
            assert_eq!(Operation::parse(o.word()), Some(o));
            for flipped in [false, true] {
                let (op, user) = o.ifc(flipped);
                let (back, right) = Operation::from_ifc(op.trim_matches('.'), user);
                assert_eq!(back, o);
                assert_eq!(right, flipped && matches!(o, Operation::Swing | Operation::Sliding), "{o:?}");
            }
        }
        assert_eq!(Operation::parse("Double Sliding"), Some(Operation::DoubleSliding));
        assert_eq!(Operation::parse("revolving"), None);
    }

    #[test]
    fn a_doors_parts_stand_on_the_side_it_faces() {
        // A 200 wall: its faces 100 either side of the centre line.
        for kind in [Operation::Sliding, Operation::Sectional, Operation::Roller] {
            for out in [1.0, -1.0] {
                for p in kind.parts(4000.0, 4000.0, 100.0 * out, out) {
                    assert!(p.across.0 * out >= 100.0 - 1e-9 && p.across.1 * out >= 100.0 - 1e-9, "{kind:?} {p:?}");
                }
            }
        }
        // Sectional panels about 600 high fill the opening; a roller's coil is over it.
        let panels: Vec<Part> =
            Operation::Sectional.parts(4000.0, 5000.0, 100.0, 1.0).into_iter().filter(|p| p.role == Role::Leaf).collect();
        assert_eq!(panels.len(), 8);
        assert!(panels.iter().all(|p| p.z.1 - p.z.0 > 600.0 && p.z.1 <= 5000.0));
        let coil = Operation::Roller.parts(3000.0, 4000.0, 100.0, 1.0).into_iter().find(|p| p.role == Role::Coil).unwrap();
        assert_eq!(coil.z, (4000.0, 4450.0));
    }
}
