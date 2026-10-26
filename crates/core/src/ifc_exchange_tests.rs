//! What crosses to and from IFC beside the elements themselves: the grids, the base quantities, and where the project
//! stands on the earth. Driven as a front end drives it: commands through `Session::execute`, `save` to an `.ifc`, read
//! back by `open`; the file is looked at through its entities and their references.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::engine::Session;

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap_or_else(|e| panic!("{id} {p}: {e}"))
}

/// A path in the temporary folder for this test.
fn temp(name: &str) -> String {
    std::env::temp_dir().join(format!("bimcraft-ifcx-{}-{name}", std::process::id())).to_string_lossy().into_owned()
}

/// The project saved as IFC, and the file's text.
fn save_ifc(s: &mut Session, name: &str) -> (String, String) {
    let path = temp(name);
    run(s, "save", json!({"path": path}));
    let text = std::fs::read_to_string(&path).unwrap();
    (path, text)
}

/// The IFC file at `path` opened as a new project.
fn open_ifc(path: &str) -> (Session, Value) {
    let mut s = Session::new();
    let reply = run(&mut s, "open", json!({"path": path}));
    (s, reply)
}

/// A STEP file's entities: `#n` to `(KIND, its arguments as written)`.
struct Step(BTreeMap<u64, (String, Vec<String>)>);

impl Step {
    fn of(text: &str) -> Step {
        let mut all = BTreeMap::new();
        for line in text.lines().filter(|l| l.starts_with('#')) {
            let (head, body) = line.split_once('=').unwrap();
            let body = body.trim_end_matches(';');
            let (kind, rest) = body.split_once('(').unwrap();
            all.insert(head[1..].parse().unwrap(), (kind.to_string(), split(&rest[..rest.len() - 1])));
        }
        Step(all)
    }

    /// The entities of kind `kind`, in the order written.
    fn all(&self, kind: &str) -> Vec<u64> {
        self.0.iter().filter(|(_, (k, _))| k == kind).map(|(n, _)| *n).collect()
    }

    fn kind(&self, n: u64) -> &str {
        &self.0[&n].0
    }

    /// Argument `i` of `#n`, as written.
    fn arg(&self, n: u64, i: usize) -> &str {
        &self.0[&n].1[i]
    }

    /// The entity argument `i` of `#n` points at.
    fn at(&self, n: u64, i: usize) -> u64 {
        reference(self.arg(n, i))
    }

    /// The entities a list argument points at.
    fn list(&self, n: u64, i: usize) -> Vec<u64> {
        let a = self.arg(n, i);
        if a == "$" {
            return Vec::new();
        }
        split(&a[1..a.len() - 1]).iter().map(|r| reference(r)).collect()
    }

    /// A real argument.
    fn real(&self, n: u64, i: usize) -> f64 {
        number(self.arg(n, i))
    }

    /// A point's coordinates.
    fn point(&self, n: u64) -> Vec<f64> {
        assert_eq!(self.kind(n), "IFCCARTESIANPOINT");
        let a = self.arg(n, 0);
        split(&a[1..a.len() - 1]).iter().map(|x| number(x)).collect()
    }

    /// The text of a string argument, without its quotes.
    fn text(&self, n: u64, i: usize) -> String {
        self.arg(n, i).trim_matches('\'').to_string()
    }

    /// The entities that point at `#n`.
    fn users(&self, n: u64) -> Vec<u64> {
        let r = format!("#{n}");
        self.0
            .iter()
            .filter(|(_, (_, args))| args.iter().any(|a| a == &r || a.starts_with('(') && split(&a[1..a.len() - 1]).contains(&r)))
            .map(|(k, _)| *k)
            .collect()
    }
}

fn reference(a: &str) -> u64 {
    a.strip_prefix('#').unwrap_or_else(|| panic!("not a reference: {a}")).parse().unwrap()
}

/// A real, typed (`IFCLENGTHMEASURE(2.)`) or not.
fn number(a: &str) -> f64 {
    let a = a.rsplit_once('(').map_or(a, |(_, x)| x.trim_end_matches(')'));
    a.parse().unwrap_or_else(|_| panic!("not a number: {a}"))
}

/// Arguments separated by commas outside brackets and strings.
fn split(s: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth, mut quoted) = (Vec::new(), String::new(), 0, false);
    for c in s.chars() {
        match c {
            '\'' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.is_empty() || !out.is_empty() {
        out.push(cur);
    }
    out
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

mod grids {
    use super::*;

    /// The grids of the sample's kind: 1 to 3 running north, A and B running east.
    fn gridded() -> Session {
        let mut s = Session::new();
        for p in [
            json!({"start": [0, -1500], "end": [0, 8500]}),
            json!({"start": [6000, -1500], "end": [6000, 8500]}),
            json!({"start": [10000, -1500], "end": [10000, 8500]}),
            json!({"start": [-1500, 0], "end": [11500, 0], "name": "A"}),
            json!({"start": [-1500, 7000], "end": [11500, 7000]}),
        ] {
            run(&mut s, "grid", p);
        }
        s
    }

    /// Each axis of grid `#g` in its list `i` (7 U, 8 V): its tag and its line's two ends in plan.
    fn axes(f: &Step, g: u64, i: usize) -> Vec<(String, [f64; 4])> {
        f.list(g, i)
            .into_iter()
            .map(|a| {
                assert_eq!(f.kind(a), "IFCGRIDAXIS");
                let line = f.at(a, 1);
                assert_eq!(f.kind(line), "IFCPOLYLINE");
                let pts = f.list(line, 0);
                assert_eq!(pts.len(), 2, "a grid line is a straight line from its start to its end");
                let (p, q) = (f.point(pts[0]), f.point(pts[1]));
                assert_eq!((p.len(), q.len()), (2, 2), "a grid axis's curve is in plan");
                (f.text(a, 0), [p[0], p[1], q[0], q[1]])
            })
            .collect()
    }

    #[test]
    fn grids_go_out_as_one_ifc_grid_with_its_axes_by_direction() {
        let mut s = gridded();
        let (_, text) = save_ifc(&mut s, "grids.ifc");
        let f = Step::of(&text);
        let grids = f.all("IFCGRID");
        assert_eq!(grids.len(), 1, "the project's grids are one IfcGrid");
        let g = grids[0];
        assert_eq!(
            axes(&f, g, 7),
            [
                ("1".to_string(), [0.0, -1500.0, 0.0, 8500.0]),
                ("2".to_string(), [6000.0, -1500.0, 6000.0, 8500.0]),
                ("3".to_string(), [10000.0, -1500.0, 10000.0, 8500.0])
            ],
            "the first grid's direction is U, each named by its tag, its line where it is"
        );
        assert_eq!(
            axes(&f, g, 8),
            [("A".to_string(), [-1500.0, 0.0, 11500.0, 0.0]), ("B".to_string(), [-1500.0, 7000.0, 11500.0, 7000.0])],
            "the grids across it are V"
        );
        assert_eq!(f.arg(g, 9), "$", "no W axes");
        assert_eq!(f.all("IFCGRIDAXIS").len(), 5);
        // Placed at the building's origin, and contained in the building, since a grid runs through every storey.
        let placement = f.at(g, 5);
        assert_eq!(f.kind(placement), "IFCLOCALPLACEMENT");
        let building = f.all("IFCBUILDING")[0];
        assert_eq!(f.at(placement, 0), f.at(building, 5), "relative to the building's placement");
        let rel = f.users(g).into_iter().find(|r| f.kind(*r) == "IFCRELCONTAINEDINSPATIALSTRUCTURE").expect("the grid is contained");
        assert_eq!(f.at(rel, 5), building);
    }

    #[test]
    fn grids_come_back_from_ifc_with_their_names_and_lines() {
        let mut s = gridded();
        let before: Vec<Value> = run(&mut s, "elements", json!({"category": "grid"}))["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| json!([g["name"], g["start"], g["end"]]))
            .collect();
        let (path, _) = save_ifc(&mut s, "grids-back.ifc");
        let (mut t, reply) = open_ifc(&path);
        let after: Vec<Value> = run(&mut t, "elements", json!({"category": "grid"}))["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| json!([g["name"], g["start"], g["end"]]))
            .collect();
        assert_eq!(after, before, "{reply}");
    }

    #[test]
    fn parallel_grids_and_a_lone_grid_still_make_a_valid_grid_and_come_back_once_each() {
        // An IfcGrid needs at least one U and one V axis.
        for lines in [vec![[[0, 0], [0, 9000]], [[5000, 0], [5000, 9000]], [[9000, 0], [9000, 9000]]], vec![[[0, 0], [0, 9000]]]] {
            let mut s = Session::new();
            for l in &lines {
                run(&mut s, "grid", json!({"start": l[0], "end": l[1]}));
            }
            let (path, text) = save_ifc(&mut s, &format!("parallel-{}.ifc", lines.len()));
            let f = Step::of(&text);
            let g = f.all("IFCGRID")[0];
            assert!(!f.list(g, 7).is_empty() && !f.list(g, 8).is_empty(), "U and V each hold an axis: {text}");
            let (mut t, _) = open_ifc(&path);
            let back = run(&mut t, "elements", json!({"category": "grid"}));
            assert_eq!(back["count"], lines.len(), "{back}");
            let names: Vec<&str> = back["elements"].as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap()).collect();
            let want: Vec<String> = (1..=lines.len()).map(|i| i.to_string()).collect();
            assert_eq!(names, want);
        }
    }

    #[test]
    fn a_project_without_grids_writes_no_grid() {
        let mut s = Session::new();
        run(&mut s, "wall", json!({"points": [[0, 0], [5000, 0]]}));
        let (_, text) = save_ifc(&mut s, "no-grids.ifc");
        assert!(!text.contains("IFCGRID"));
    }
}

mod quantities {
    use super::*;

    /// The base quantity set of entity `#ent`: its name and each quantity's value, as the file's units give them.
    fn qto(f: &Step, ent: u64) -> (String, BTreeMap<String, f64>) {
        let sets: Vec<u64> = f
            .all("IFCRELDEFINESBYPROPERTIES")
            .into_iter()
            .filter(|r| f.list(*r, 4).contains(&ent) && f.kind(f.at(*r, 5)) == "IFCELEMENTQUANTITY")
            .map(|r| f.at(r, 5))
            .collect();
        assert_eq!(sets.len(), 1, "one base quantity set for #{ent}");
        let set = sets[0];
        let values = f
            .list(set, 5)
            .into_iter()
            .map(|q| {
                assert!(["IFCQUANTITYLENGTH", "IFCQUANTITYAREA", "IFCQUANTITYVOLUME"].contains(&f.kind(q)), "{}", f.kind(q));
                (f.text(q, 0), f.real(q, 3))
            })
            .collect();
        (f.text(set, 2), values)
    }

    /// The entity of kind `kind` written for element `id` (its Tag).
    fn of(f: &Step, kind: &str, id: u64) -> u64 {
        let tag = format!("'{id}'");
        *f.all(kind).iter().find(|e| f.0[e].1.contains(&tag)).unwrap_or_else(|| panic!("no {kind} for {id}"))
    }

    fn check(q: &BTreeMap<String, f64>, name: &str, want: f64, tol: f64) {
        let got = q.get(name).copied().unwrap_or_else(|| panic!("no {name} in {q:?}"));
        assert!(near(got, want, tol), "{name}: {got}, not {want} ({q:?})");
    }

    fn id(v: &Value) -> u64 {
        v["id"].as_u64().or_else(|| v["ids"][0].as_u64()).unwrap()
    }

    fn element(s: &mut Session, id: u64) -> Value {
        run(s, "elements", json!({}))["elements"].as_array().unwrap().iter().find(|e| e["id"] == id).unwrap().clone()
    }

    #[test]
    fn walls_slabs_spaces_and_openings_carry_their_base_quantities() {
        // Given a 6 x 4 m box of walls up to Level 2, a door and a window in it, a floor under it and a room in it.
        let mut s = Session::new();
        let walls = run(&mut s, "wall", json!({"points": [[0, 0], [6000, 0], [6000, 4000], [0, 4000]], "closed": true}));
        let south = walls["ids"][0].as_u64().unwrap();
        let door = id(&run(&mut s, "door", json!({"at": [3000, 0]})));
        let window = id(&run(&mut s, "window", json!({"at": [6000, 2000]})));
        let floor = id(&run(&mut s, "floor", json!({"boundary": [[0, 0], [6000, 0], [6000, 4000], [0, 4000]]})));
        let room = id(&run(&mut s, "room", json!({"point": [3000, 2000], "name": "Hall"})));
        let room = element(&mut s, room);
        let (wall, d, wn, fl) = (element(&mut s, south), element(&mut s, door), element(&mut s, window), element(&mut s, floor));
        let (t, h) = (wall["thickness"].as_f64().unwrap(), wall["height"].as_f64().unwrap());
        let (dw, dh) = (d["width"].as_f64().unwrap(), d["height"].as_f64().unwrap());
        let (ww, wh) = (wn["width"].as_f64().unwrap(), wn["height"].as_f64().unwrap());
        let ft = fl["thickness"].as_f64().unwrap();

        // When it is saved as IFC.
        let (_, text) = save_ifc(&mut s, "qto.ifc");
        let f = Step::of(&text);

        // Then the south wall, 6 m along its centre line, says how long, thick and tall it is, and its areas and volumes
        // (m², m³) with and without the door's hole.
        let (name, q) = qto(&f, of(&f, "IFCWALL", south));
        assert_eq!(name, "Qto_WallBaseQuantities");
        check(&q, "Length", 6000.0, 1e-6);
        check(&q, "Width", t, 1e-6);
        check(&q, "Height", h, 1e-6);
        let side = 6.0 * h / 1000.0;
        check(&q, "GrossSideArea", side, 1e-4);
        check(&q, "NetSideArea", side - dw * dh / 1e6, 1e-4);
        check(&q, "GrossFootprintArea", 6.0 * t / 1000.0, 1e-4);
        check(&q, "GrossVolume", side * t / 1000.0, 1e-4);
        check(&q, "NetVolume", (side - dw * dh / 1e6) * t / 1000.0, 1e-4);

        // The door and the window (tagged with their marks, the only ones): their sizes and the area of their openings.
        let (name, q) = qto(&f, f.all("IFCDOOR")[0]);
        assert_eq!(name, "Qto_DoorBaseQuantities");
        check(&q, "Width", dw, 1e-6);
        check(&q, "Height", dh, 1e-6);
        check(&q, "Area", dw * dh / 1e6, 1e-6);
        let (name, q) = qto(&f, f.all("IFCWINDOW")[0]);
        assert_eq!(name, "Qto_WindowBaseQuantities");
        check(&q, "Width", ww, 1e-6);
        check(&q, "Height", wh, 1e-6);
        check(&q, "Area", ww * wh / 1e6, 1e-6);

        // The floor: its thickness, its perimeter, and its areas and volumes.
        let (name, q) = qto(&f, of(&f, "IFCSLAB", floor));
        assert_eq!(name, "Qto_SlabBaseQuantities");
        check(&q, "Width", ft, 1e-6);
        check(&q, "Perimeter", 20000.0, 1e-6);
        check(&q, "GrossArea", 24.0, 1e-6);
        check(&q, "NetArea", 24.0, 1e-6);
        check(&q, "GrossVolume", 24.0 * ft / 1000.0, 1e-6);
        check(&q, "NetVolume", 24.0 * ft / 1000.0, 1e-6);

        // The room: its area and perimeter as BIMCraft measures them (at the walls' centre lines, so here the box's 6 x 4
        // m), as tall as the storey.
        let space = f.all("IFCSPACE")[0];
        let (name, q) = qto(&f, space);
        assert_eq!(name, "Qto_SpaceBaseQuantities");
        check(&q, "NetFloorArea", room["area_m2"].as_f64().unwrap(), 0.005);
        check(&q, "GrossFloorArea", 24.0, 1e-6);
        check(&q, "NetPerimeter", 20000.0, 1e-6);
        check(&q, "GrossPerimeter", 20000.0, 1e-6);
        check(&q, "Height", 3000.0, 1e-6);
        check(&q, "GrossVolume", 72.0, 1e-6);
    }

    #[test]
    fn columns_beams_and_roofs_carry_their_base_quantities() {
        let mut s = Session::new();
        let column = id(&run(&mut s, "structural_column", json!({"at": [0, 0]})));
        let beam = id(&run(&mut s, "beam", json!({"points": [[0, 0], [6000, 0]], "offset": 3000})));
        let roof = id(&run(
            &mut s,
            "roof",
            json!({"boundary": [[0, 0], [8000, 0], [8000, 6000], [0, 6000]], "slope": 30, "overhang": 0, "level": "Level 2"}),
        ));
        let (c, b, r) = (element(&mut s, column), element(&mut s, beam), element(&mut s, roof));
        let (_, text) = save_ifc(&mut s, "qto-structure.ifc");
        let f = Step::of(&text);

        // A column: how long, its section, and its volume.
        let (cw, cd, ch) = (c["width"].as_f64().unwrap(), c["depth"].as_f64().unwrap(), c["height"].as_f64().unwrap());
        let (name, q) = qto(&f, of(&f, "IFCCOLUMN", column));
        assert_eq!(name, "Qto_ColumnBaseQuantities");
        check(&q, "Length", ch, 1e-6);
        check(&q, "CrossSectionArea", cw * cd / 1e6, 1e-6);
        check(&q, "GrossVolume", cw * cd * ch / 1e9, 1e-6);
        check(&q, "NetVolume", cw * cd * ch / 1e9, 1e-6);

        // A beam: how long along its axis, its section, and its volume.
        let (bw, bd) = (b["width"].as_f64().unwrap(), b["depth"].as_f64().unwrap());
        let (name, q) = qto(&f, of(&f, "IFCBEAM", beam));
        assert_eq!(name, "Qto_BeamBaseQuantities");
        check(&q, "Length", 6000.0, 1e-6);
        check(&q, "CrossSectionArea", bw * bd / 1e6, 1e-6);
        check(&q, "GrossVolume", bw * bd * 6000.0 / 1e9, 1e-6);

        // A hipped roof, every slope at 30°: its area on the slope, and the area it covers in plan.
        let projected = r["area_m2"].as_f64().unwrap();
        let (name, q) = qto(&f, of(&f, "IFCROOF", roof));
        assert_eq!(name, "Qto_RoofBaseQuantities");
        check(&q, "ProjectedArea", projected, 0.01);
        check(&q, "GrossArea", projected / 30f64.to_radians().cos(), 0.02);
        check(&q, "NetArea", projected / 30f64.to_radians().cos(), 0.02);
    }

    #[test]
    fn a_room_that_isnt_enclosed_has_no_quantities_to_give() {
        let mut s = Session::new();
        let walls = run(&mut s, "wall", json!({"points": [[0, 0], [6000, 0], [6000, 4000], [0, 4000]], "closed": true}));
        run(&mut s, "room", json!({"point": [3000, 2000]}));
        run(&mut s, "delete", json!({"ids": [walls["ids"][3]]}));
        let (_, text) = save_ifc(&mut s, "qto-open-room.ifc");
        let f = Step::of(&text);
        let space = f.all("IFCSPACE")[0];
        assert!(
            f.users(space)
                .into_iter()
                .filter(|r| f.kind(*r) == "IFCRELDEFINESBYPROPERTIES")
                .all(|r| f.kind(f.at(r, 5)) != "IFCELEMENTQUANTITY"),
            "no area to measure"
        );
    }
}
