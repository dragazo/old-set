use std::collections::{BTreeSet, BTreeMap, HashMap, HashSet};
use std::fmt;
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

mod util;
mod adj;
mod codesets;

use adj::AdjacentIterator;

enum Goal {
    MeetOrBeat(f64),
    Exactly(usize),
}
impl Goal {
    fn get_value(&self, total_size: usize) -> usize {
        match self {
            Goal::MeetOrBeat(v) => (total_size as f64 * v).floor() as usize,
            Goal::Exactly(v) => *v,
        }
    }
}

trait Solver {
    fn get_locating_code<Adj>(&self, pos: (isize, isize)) -> Vec<(isize, isize)> where Adj: adj::AdjacentIterator;
    fn is_old<Adj>(&mut self) -> bool where Adj: adj::AdjacentIterator;
    fn try_satisfy<Adj>(&mut self, goal: Goal) -> Option<usize> where Adj: adj::AdjacentIterator;
}

struct RectSolverBase<'a, Codes> {
    src: &'a mut RectTessellation,
    codes: Codes,
    interior_codes: Codes,
    needed: usize,
    prevs: Vec<(isize, isize)>,
}
struct RectSolver<'a, Codes> {
    base: RectSolverBase<'a, Codes>,
    phases: Vec<(isize, isize)>,
}
impl<Codes> RectSolverBase<'_, Codes> where Codes: codesets::Set<(isize, isize)> {
    fn id_to_inside(&self, row: isize, col: isize) -> usize {
        let col_fix = if row < 0 { col - self.src.phase.0 } else if row >= self.src.rows as isize { col + self.src.phase.0 } else { col };
        let row_fix = if col < 0 { row - self.src.phase.1 } else if col >= self.src.cols as isize { row + self.src.phase.1 } else { row };

        let r = (row_fix + 2 * self.src.rows as isize) as usize % self.src.rows;
        let c = (col_fix + 2 * self.src.cols as isize) as usize % self.src.cols;
        r * self.src.cols + c
    }
    fn get_locating_code<Adj: adj::AdjacentIterator>(&self, pos: (isize, isize)) -> Vec<(isize, isize)> {
        let mut v = Vec::with_capacity(9);
        for x in Adj::new(pos.0, pos.1) {
            if self.src.old_set[self.id_to_inside(x.0, x.1)] {
                v.push(x)
            }
        }
        v
    }
    fn is_old_interior_up_to<Adj: adj::AdjacentIterator>(&mut self, row: isize) -> bool {
        self.interior_codes.clear();
        for r in 1..row - 1 {
            for c in 1..self.src.cols as isize - 1 {
                let is_detector = self.src.old_set[self.id_to_inside(r, c)];
                let code = self.get_locating_code::<Adj>((r, c));
                if !self.interior_codes.add(is_detector, code) {
                    return false;
                }
            }
        }
        true
    }
    fn is_old_with_current_phase<Adj: adj::AdjacentIterator>(&mut self) -> bool {
        self.codes.clear();
        for &r in &[-1, 0, self.src.rows as isize - 1, self.src.rows as isize] {
            for c in -1 ..= self.src.cols as isize {
                let is_detector = self.src.old_set[self.id_to_inside(r, c)];
                let code = self.get_locating_code::<Adj>((r, c));
                if !self.interior_codes.can_add(is_detector, &code) || !self.codes.add(is_detector, code) {
                    return false;
                }
            }
        }
        for r in 1 ..= self.src.rows as isize - 2 {
            for &c in &[-1, 0, self.src.cols as isize - 1, self.src.cols as isize] {
                let is_detector = self.src.old_set[self.id_to_inside(r, c)];
                let code = self.get_locating_code::<Adj>((r, c));
                if !self.interior_codes.can_add(is_detector, &code) || !self.codes.add(is_detector, code) {
                    return false;
                }
            }
        }
        true
    }
}
impl<Codes> RectSolver<'_, Codes>
where Codes: codesets::Set<(isize, isize)>
{
    fn calc_old_min_interior<Adj: adj::AdjacentIterator>(&mut self, pos: usize) -> bool {
        if self.base.needed == self.base.prevs.len() {
            if self.is_old::<Adj>() {
                return true;
            }
        } else if pos + (self.base.needed - self.base.prevs.len()) <= self.base.src.old_set.len() { //pos < self.base.src.old_set.len() {
            let p = ((pos / self.base.src.cols) as isize, (pos % self.base.src.cols) as isize);

            let good_so_far = p.1 != 0 || self.base.is_old_interior_up_to::<Adj>(p.0);

            if good_so_far {
                self.base.src.old_set[pos] = true;
                self.base.prevs.push(p);
                if self.calc_old_min_interior::<Adj>(pos + 1) {
                    return true;
                }
                self.base.prevs.pop();
                self.base.src.old_set[pos] = false;

                return self.calc_old_min_interior::<Adj>(pos + 1);
            }
        }

        false
    }
}
impl<Codes> Solver for RectSolver<'_, Codes>
where Codes: codesets::Set<(isize, isize)>
{
    fn get_locating_code<Adj: adj::AdjacentIterator>(&self, pos: (isize, isize)) -> Vec<(isize, isize)> {
        self.base.get_locating_code::<Adj>(pos)
    }
    fn is_old<Adj: adj::AdjacentIterator>(&mut self) -> bool {
        if self.base.is_old_interior_up_to::<Adj>(self.base.src.rows as isize) {
            for phase in &self.phases {
                self.base.src.phase = *phase;
                if self.base.is_old_with_current_phase::<Adj>() {
                    return true;
                }
            }
        }
        false
    }
    fn try_satisfy<Adj: adj::AdjacentIterator>(&mut self, goal: Goal) -> Option<usize> {
        for x in &mut self.base.src.old_set { *x = false; }
        self.base.prevs.clear();
        self.base.needed = goal.get_value(self.base.src.old_set.len());

        if self.calc_old_min_interior::<Adj>(0) { Some(self.base.needed) } else { None }
    }
}

trait Tessellation: fmt::Display {
    fn size(&self) -> usize;
    fn try_satisfy<Codes, Adj>(&mut self, goal: Goal) -> Option<usize>
    where Codes: codesets::Set<(isize, isize)>, Adj: adj::AdjacentIterator;
}

struct RectTessellation {
    rows: usize,
    cols: usize,
    old_set: Vec<bool>,
    phase: (isize, isize),
}
impl fmt::Display for RectTessellation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for r in 0..self.rows {
            for c in 0..self.cols {
                write!(f, "{} ", if self.old_set[r * self.cols + c] { 1 } else { 0 })?;
            }
            writeln!(f)?;
        }
        writeln!(f, "phase: {:?}", self.phase)?;
        Ok(())
    }
}
impl RectTessellation {
    fn new(rows: usize, cols: usize) -> Self {
        assert!(rows >= 2);
        assert!(cols >= 2);

        RectTessellation {
            rows, cols,
            old_set: vec![false; rows * cols],
            phase: (0, 0),
        }
    }
    fn solver<Codes>(&mut self) -> RectSolver<'_, Codes>
    where Codes: codesets::Set<(isize, isize)>
    {
        let (r, c) = (self.rows, self.cols);
        RectSolver::<Codes> {
            base: RectSolverBase::<Codes> {
                src: self,
                codes: Default::default(),
                interior_codes: Default::default(),
                needed: 0,
                prevs: vec![],
            },
            phases: {
                let max_phase_x = (r as isize + 1) / 2;
                let max_phase_y = (c as isize + 1) / 2;
                std::iter::once((0, 0)).chain((1..=max_phase_x).map(|x| (x, 0))).chain((1..=max_phase_y).map(|y| (0, y))).collect()
            },
        }
    }
}
impl Tessellation for RectTessellation {
    fn size(&self) -> usize {
        self.old_set.len()
    }
    fn try_satisfy<Codes, Adj>(&mut self, goal: Goal) -> Option<usize>
    where Codes: codesets::Set<(isize, isize)>, Adj: adj::AdjacentIterator
    {
        self.solver::<Codes>().try_satisfy::<Adj>(goal)
    }
}

struct Geometry {
    shape: BTreeSet<(isize, isize)>,
    detectors: BTreeSet<(isize, isize)>,
}
impl Geometry {
    fn for_printing(shape: &BTreeSet<(isize, isize)>, detectors: &BTreeSet<(isize, isize)>) -> Self {
        let mut min = (isize::MAX, isize::MAX);
        for p in shape {
            if p.0 < min.0 {
                min.0 = p.0;
            }
            if p.1 < min.1 {
                min.1 = p.1;
            }
        }
        Self {
            shape: shape.iter().map(|p| (p.0 - min.0, p.1 - min.1)).collect(),
            detectors: detectors.iter().map(|p| (p.0 - min.0, p.1 - min.1)).collect(),
        }
    }
}
impl fmt::Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut working_row = !0;
        let mut working_col = 0;
        for x in &self.shape {
            if x.0 != working_row {
                if working_row != !0 {
                    for _ in 0..(x.0 - working_row) { writeln!(f)?; }
                }
                working_row = x.0;
                working_col = 0;
            }
            for _ in 0..(x.1 - working_col) { write!(f, "  ")?; }
            working_col = x.1 + 1;
            write!(f, "{} ", if self.detectors.contains(x) { 1 } else { 0 })?;
        }
        writeln!(f)?;
        Ok(())
    }
}

enum GeometryLoadResult {
    FileOpenFailure,
    InvalidFormat(&'static str),
    TessellationFailure(&'static str),
}
struct GeometrySolver<'a, Codes> {
    shape: &'a BTreeSet<(isize, isize)>,
    interior: &'a BTreeSet<(isize, isize)>,
    shape_with_padding: &'a BTreeSet<(isize, isize)>,
    tessellation_map: &'a HashMap<(isize, isize), (isize, isize)>,
    first_per_row: &'a HashSet<(isize, isize)>,
    old_set: &'a mut BTreeSet<(isize, isize)>,

    codes: Codes,
    needed: usize,
}
impl<'a, Codes> GeometrySolver<'a, Codes>
where Codes: codesets::Set<(isize, isize)>
{
    fn is_old_interior_up_to<Adj: adj::AdjacentIterator>(&mut self, row: isize) -> bool {
        self.codes.clear();
        for p in self.interior {
            if p.0 >= row - 1 { break; }
            let is_detector = self.old_set.contains(self.tessellation_map.get(p).unwrap());
            let code = self.get_locating_code::<Adj>(*p);
            if !self.codes.add(is_detector, code) {
                return false;
            }
        }
        true
    }
    fn calc_old_min_interior<'b, Adj, P>(&mut self, mut pos: P) -> bool
    where Adj: adj::AdjacentIterator, P: Iterator<Item = (usize, &'b (isize, isize))> + Clone
    {
        if self.needed == self.old_set.len() {
            if self.is_old::<Adj>() {
                return true;
            }
        } else if let Some((i, &p)) = pos.next() {
            if i + (self.needed - self.old_set.len()) > self.shape.len() {
                return false;
            }

            let good_so_far = !self.first_per_row.contains(&p) || self.is_old_interior_up_to::<Adj>(p.0);

            if good_so_far {
                self.old_set.insert(p);
                if self.calc_old_min_interior::<Adj, _>(pos.clone()) {
                    return true;
                }
                self.old_set.remove(&p);

                return self.calc_old_min_interior::<Adj, _>(pos);
            }
        }

        false
    }
}
impl<Codes> Solver for GeometrySolver<'_, Codes>
where Codes: codesets::Set<(isize, isize)>
{
    fn get_locating_code<Adj: adj::AdjacentIterator>(&self, pos: (isize, isize)) -> Vec<(isize, isize)> {
        let mut v = Vec::with_capacity(9);
        for x in Adj::new(pos.0, pos.1) {
            if self.old_set.contains(self.tessellation_map.get(&x).unwrap()) {
                v.push(x);
            }
        }
        v
    }
    fn is_old<Adj: adj::AdjacentIterator>(&mut self) -> bool {
        self.codes.clear();
        for pos in self.shape_with_padding {
            let is_detector = self.old_set.contains(self.tessellation_map.get(pos).unwrap());
            let code = self.get_locating_code::<Adj>(*pos);
            if !self.codes.add(is_detector, code) {
                return false;
            }
        }
        true
    }
    fn try_satisfy<Adj: adj::AdjacentIterator>(&mut self, goal: Goal) -> Option<usize> {
        self.old_set.clear();
        self.needed = goal.get_value(self.shape.len());

        if self.calc_old_min_interior::<Adj, _>(self.shape.iter().enumerate()) { Some(self.needed) } else { None }
    }
}

struct GeometryTessellation {
    geo: Geometry,
    interior: BTreeSet<(isize, isize)>,
    shape_with_padding: BTreeSet<(isize, isize)>,
    tessellation_map: HashMap<(isize, isize), (isize, isize)>,
    first_per_row: HashSet<(isize, isize)>,
    basis_a: (isize, isize),
    basis_b: (isize, isize),
}
impl fmt::Display for GeometryTessellation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.geo)?;
        writeln!(f, "basis: {:?} {:?}", self.basis_a, self.basis_b)?;
        writeln!(f, "size: {}", self.size())?;
        Ok(())
    }
}
impl GeometryTessellation {
    fn with_shape<P: AsRef<Path>>(path: P) -> Result<Self, GeometryLoadResult> {
        let mut f = BufReader::new(match File::open(path) {
            Ok(x) => x,
            Err(_) => return Err(GeometryLoadResult::FileOpenFailure),
        });

        let (basis_a, basis_b) = {
            let mut line = String::new();
            f.read_line(&mut line).unwrap();

            let args: Vec<&str> = line.split_whitespace().collect();
            if args.len() != 4 {
                return Err(GeometryLoadResult::InvalidFormat("expected 4 tessellation arguments as top line of file"));
            }
            let mut parsed: Vec<isize> = vec![];
            for arg in args {
                parsed.push(match arg.parse::<isize>() {
                    Ok(x) => x,
                    Err(_) => return Err(GeometryLoadResult::InvalidFormat("failed to parse a tessellation arg as integer")),
                });
            }
            ((parsed[0], parsed[1]), (parsed[2], parsed[3]))
        };

        let geo = {
            let mut shape: BTreeSet<(isize, isize)> = Default::default();
            for (row, line) in f.lines().map(|x| x.unwrap()).enumerate() {
                for (col, item) in line.split_whitespace().enumerate() {
                    match item {
                        x if x.len() != 1 => return Err(GeometryLoadResult::InvalidFormat("expected geometry element to be length 1")),
                        "." => (),
                        _ => { shape.insert((row as isize, col as isize)); },
                    };
                }
            }
            if shape.is_empty() {
                return Err(GeometryLoadResult::InvalidFormat("shape is empty"));
            }
            Geometry::for_printing(&shape, &Default::default())
        };
        let interior = {
            let boundary: BTreeSet<_> = geo.shape.iter().filter(|x| adj::OpenKing::new(x.0, x.1).any(|y| !geo.shape.contains(&y))).cloned().collect();
            &geo.shape - &boundary
        };
        let first_per_row = {
            let mut s: HashSet<(isize, isize)> = Default::default();
            let mut r = !0;
            for p in &geo.shape {
                if p.0 != r {
                    r = p.0;
                    s.insert(*p);
                }
            }
            s
        };

        let shape_with_padding: BTreeSet<_> = {
            let mut t = geo.shape.clone();
            t.extend(geo.shape.iter().flat_map(|x| adj::OpenKing::new(x.0, x.1)));
            t
        };
        let shape_with_extra_padding: BTreeSet<_> = {
            let mut t = shape_with_padding.clone();
            t.extend(shape_with_padding.iter().flat_map(|x| adj::OpenKing::new(x.0, x.1)));
            t
        };

        let tessellation_map = {
            let mut p: HashSet<(isize, isize)> = HashSet::with_capacity(geo.shape.len() * 25);
            let mut m: HashMap<(isize, isize), (isize, isize)> = HashMap::with_capacity(geo.shape.len() * 9);

            for &to in &geo.shape {
                for i in -2..=2 {
                    for j in -2..=2 {
                        let from = (to.0 + basis_a.0 * i + basis_b.0 * j, to.1 + basis_a.1 * i + basis_b.1 * j);
                        if !p.insert(from) {
                            return Err(GeometryLoadResult::TessellationFailure("tessellation resulted in overlap"));
                        }
                        if shape_with_extra_padding.contains(&from) {
                            m.insert(from, to);
                        }
                    }
                }
            }
            for pos in &shape_with_extra_padding {
                if !m.contains_key(pos) {
                    return Err(GeometryLoadResult::TessellationFailure("tessellation is not dense"));
                }
            }

            m
        };

        Ok(Self {
            geo, interior, shape_with_padding, tessellation_map, first_per_row,
            basis_a, basis_b,
        })
    }
    fn solver<Codes>(&mut self) -> GeometrySolver<'_, Codes>
    where Codes: codesets::Set<(isize, isize)>
    {
        GeometrySolver::<Codes> {
            shape: &self.geo.shape,
            interior: &self.interior,
            shape_with_padding: &self.shape_with_padding,
            tessellation_map: &self.tessellation_map,
            first_per_row: &self.first_per_row,
            old_set: &mut self.geo.detectors,

            codes: Default::default(),
            needed: 0,
        }
    }
}
impl Tessellation for GeometryTessellation {
    fn size(&self) -> usize {
        self.geo.shape.len()
    }
    fn try_satisfy<Codes, Adj>(&mut self, goal: Goal) -> Option<usize>
    where Codes: codesets::Set<(isize, isize)>, Adj: adj::AdjacentIterator
    {
        self.solver::<Codes>().try_satisfy::<Adj>(goal)
    }
}

#[derive(Default)]
struct Lands {
    closelands: Vec<(isize, isize)>,
    farlands: Vec<(isize, isize)>,
}

#[derive(Default)]
struct LowerBoundSearcher<'a, Codes>
where Codes: Default
{
    center: (isize, isize),
    closed_interior: Rc<RefCell<BTreeSet<(isize, isize)>>>, // everything up to radius 2
    open_interior: Rc<RefCell<BTreeSet<(isize, isize)>>>,   // everything up to radius 2 except center
    exterior: Rc<RefCell<BTreeSet<(isize, isize)>>>,        // everything at exactly radius 3
    detectors: BTreeSet<(isize, isize)>,

    boundary_map: Rc<RefCell<BTreeMap<(isize, isize), Lands>>>, // maps a boundary point (exactly radius 2) to radius 2 around itself at radius 4 from center

    averagees: RefCell<BTreeMap<(isize, isize), f64>>,
    averagee_drops: Vec<(isize, isize)>,

    codes: Codes,

    highest: f64,
    thresh: f64,
    pipe: Option<&'a mut dyn io::Write>,
}
impl<'a, Codes> LowerBoundSearcher<'a, Codes>
where Codes: codesets::Set<(isize, isize)>
{
    fn get_locating_code<Adj>(&self, pos: (isize, isize)) -> Vec<(isize, isize)>
    where Adj: AdjacentIterator
    {
        let mut v = Vec::with_capacity(9);
        for p in Adj::new(pos.0, pos.1) {
            if self.detectors.contains(&p) {
                v.push(p);
            }
        }
        v
    }
    fn get_locating_code_size<Adj>(&self, pos: (isize, isize)) -> usize
    where Adj: AdjacentIterator
    {
        Adj::new(pos.0, pos.1).filter(|p| self.detectors.contains(&p)).count()
    }
    fn is_valid_over_append<Adj, T>(&mut self, range: T) -> bool
    where Adj: AdjacentIterator, T: Iterator<Item = (isize, isize)>
    {
        for p in range {
            let is_detector = self.detectors.contains(&p);
            let code = self.get_locating_code::<Adj>(p);
            if !self.codes.add(is_detector, code) {
                return false;
            }
        }
        true
    }
    fn is_valid_over<Adj, T>(&mut self, range: T) -> bool
    where Adj: AdjacentIterator, T: Iterator<Item = (isize, isize)>
    {
        self.codes.clear();
        self.is_valid_over_append::<Adj, _>(range)
    }
    fn calc_share<Adj>(&self, pos: (isize, isize)) -> f64
    where Adj: AdjacentIterator
    {
        assert!(self.detectors.contains(&pos));

        let mut share = 0.0;
        for p in Adj::new(pos.0, pos.1) {
            let c = cmp::max(self.get_locating_code_size::<Adj>(p), Codes::MIN_DOM);
            share += 1.0 / c as f64;
        }
        share
    }
    fn calc_share_boundary_recursive<Adj, P>(&mut self, pos: (isize, isize), mut far_pos: P) -> f64
    where Adj: AdjacentIterator, P: Iterator<Item = (isize, isize)> + Clone
    {
        // check the farlands position
        match far_pos.next() {
            // if no more far positions, check for third order validity
            None => {
                // if it's not valid this case was never possible in the first case - return -1 to show that this is ok so far
                if !self.is_valid_over::<Adj, _>(self.closed_interior.clone().borrow().iter().copied()) {
                    return -1.0;
                }
                if !self.is_valid_over_append::<Adj, _>(self.boundary_map.clone().borrow().get(&pos).unwrap().closelands.iter().copied()) {
                    return -1.0;
                }

                // otherwise compute and return the share
                return self.calc_share::<Adj>(pos);
            }
            // otherwise recurse on both branches at this position
            Some(p) => {
                self.detectors.insert(p);
                let res1 = self.calc_share_boundary_recursive::<Adj, _>(pos, far_pos.clone());
                if res1 > self.thresh {
                    return res1; // if first branch is over thresh then max will be too - short circuit
                }
                self.detectors.remove(&p);
                let res2 = self.calc_share_boundary_recursive::<Adj, _>(pos, far_pos);

                // return whichever was larger
                return if res1 >= res2 { res1 } else { res2 };
            }
        }
    }
    fn calc_share_boundary<Adj>(&mut self, pos: (isize, isize)) -> f64
    where Adj: AdjacentIterator
    {
        let boundary_map = self.boundary_map.clone();
        let boundary_map = boundary_map.borrow();
        let lands = boundary_map.get(&pos).unwrap(); // this is done to ensure the boundary assertion even if trivial works

        // attempt a trivial calculation, if no larger than thresh it's good enough
        let trivial = self.calc_share::<Adj>(pos);
        if trivial <= self.thresh {
            return trivial;
        }

        // otherwise we need to recurse into farlands and check for third order validity
        self.calc_share_boundary_recursive::<Adj, _>(pos, lands.farlands.iter().copied())
    }
    fn can_average_recursive<Adj, P>(&mut self, mut ext_pos: P) -> bool
    where Adj: AdjacentIterator, P: Iterator<Item = (isize, isize)> + Clone
    {
        match ext_pos.next() {
            // if we have no exterior position remaining, check for second order validity
            None => {
                // if it's not valid we need not consider this case - just return true to indicate it's still possible another configuration could work
                if !self.is_valid_over::<Adj, _>(self.closed_interior.clone().borrow().iter().copied()) {
                    return true;
                }

                {
                    let mut averagees = self.averagees.borrow_mut();

                    // go through averagees and compute share, take highest for each averagee, but if not less than thresh remove (not useful)
                    self.averagee_drops.clear();
                    for x in averagees.iter_mut() {
                        let share = self.calc_share::<Adj>(*x.0);

                        if share >= self.thresh {
                            // drop it and its neighbors - this avoids recomputing neighborhood adjacent shares in logic below
                            self.averagee_drops.extend(Adj::Closed::at(*x.0));
                        }
                        else if *x.1 < share {
                            *x.1 = share;
                        }
                    }
                    for x in &self.averagee_drops {
                        averagees.remove(x);
                    }
                }

                // grab the boundary detectors neighboring remaining averagees - collect into BTreeSet to avoid duplicates
                let boundary_detectors: BTreeSet<_> = {
                    let boundary_map = self.boundary_map.clone();
                    let boundary_map = boundary_map.borrow();
                    self.averagees.borrow().iter().flat_map(|x| Adj::Open::at(*x.0)).filter(|x| self.detectors.contains(&x) && boundary_map.contains_key(&x)).collect()
                };

                // for each boundary detector
                for p in boundary_detectors.into_iter() {
                    // if we don't neighbor an averagee, skip it (short circuiting from previous averagee removal - see below)
                    {
                        let averagees = self.averagees.borrow();
                        if !Adj::Open::at(p).any(|x| averagees.contains_key(&x)) {
                            continue;
                        }
                    }
                    
                    // compute its share with farlands extended constraints
                    let share = self.calc_share_boundary::<Adj>(p);

                    // if it's strictly greater than thresh we can't use any of p's neighbors
                    if share > self.thresh {
                        let mut averagees = self.averagees.borrow_mut();
                        for x in Adj::Open::at(p) {
                            averagees.remove(&x);
                        }
                    }
                }

                // only return false if we no longer have any valid averagees
                return !self.averagees.borrow().is_empty();
            }
            // otherwise recurse on both branches at this position
            Some(p) => {
                self.detectors.insert(p);
                if !self.can_average_recursive::<Adj, _>(ext_pos.clone()) {
                    return false;
                }
                self.detectors.remove(&p);
                return self.can_average_recursive::<Adj, _>(ext_pos);
            }
        }
    }
    fn can_average<Adj>(&mut self, center_share: f64) -> Option<f64>
    where Adj: AdjacentIterator
    {
        let exterior = self.exterior.clone();

        // set open neighbors which are detectors as possible averagees
        {
            let mut averagees = self.averagees.borrow_mut();
            averagees.clear();
            averagees.extend(Adj::Open::at(self.center).filter(|p| self.detectors.contains(p)).map(|p| (p, -1.0)));
        }

        // perform the search
        if !self.can_average_recursive::<Adj, _>(exterior.borrow().iter().copied()) {
            return None;
        }

        let avg = {
            let averagees = self.averagees.borrow();
            assert!(!averagees.is_empty());

            // if any of the averagees is still negative then there were no legal second order configurations, which means this case was impossible in the first place
            for x in averagees.iter() {
                if *x.1 < 0.0 {
                    return Some(0.0) // return an average share of 0, which will always be within thresh
                }
            }

            // compute the average of center and averagees
            (center_share + averagees.values().copied().sum::<f64>()) / (averagees.len() + 1) as f64
        };

        // if it was within thresh, return it - otherwise indicate failure to resolve
        if avg <= self.thresh {
            Some(avg)
        }
        else {
            None
        }
    }
    fn calc_recursive<Adj, P>(&mut self, mut pos: P)
    where Adj: AdjacentIterator, P: Iterator<Item = (isize, isize)> + Clone
    {
        match pos.next() {
            // if we have no positions remaining, check for first order validity
            None => {
                // if not valid on first order, ignore
                if !self.is_valid_over::<Adj, _>(Adj::Closed::at(self.center)) {
                    return;
                }

                // compute share of center
                let mut share = self.calc_share::<Adj>(self.center);
                
                // if share is over thresh, attempt to perform averaging - on success update share to reflect average
                if share > self.thresh {
                    match self.can_average::<Adj>(share) {
                        Some(avg) => share = avg,
                        None => (),
                    }
                }

                // take the max (average) share
                if self.highest < share {
                    self.highest = share;
                }
                // if it was over thresh, display as problem case
                if share > self.thresh {
                    match self.pipe {
                        Some(ref mut f) => {
                            let geo = Geometry::for_printing(&*self.closed_interior.clone().borrow(), &self.detectors);
                            writeln!(f, "problem: {}\ncenter: {:?}\n{}", share, self.center, geo).unwrap();
                        }
                        None => (),
                    }
                }
            }
            // otherwise recurse on both branches at this position
            Some(p) => {
                self.detectors.insert(p);
                self.calc_recursive::<Adj, _>(pos.clone());
                self.detectors.remove(&p);
                self.calc_recursive::<Adj, _>(pos)
            }
        }
    }
    fn calc<Adj>(&mut self, thresh: f64, pipe: Option<&'a mut dyn io::Write>, centers: &[(isize, isize)]) -> ((usize, usize), f64)
    where Adj: AdjacentIterator
    {
        let closed_interior = self.closed_interior.clone();
        let open_interior = self.open_interior.clone();
        let exterior = self.exterior.clone();
        let boundary_map = self.boundary_map.clone();

        // prepare shared search state before starting
        self.highest = 0.0;
        self.thresh = thresh;
        self.pipe = pipe;

        // fold recursive results from all provided center values
        for c in centers {
            // set the center
            self.center = *c;

            // generate closed interior - everything up to radius 2
            {
                let mut closed_interior = closed_interior.borrow_mut();
                closed_interior.clear();
                closed_interior.extend(Adj::Open::at(*c).flat_map(|p| Adj::Closed::at(p)));
            }

            // generate open interior - everything up to radius 2 except the center
            {
                let mut open_interior = open_interior.borrow_mut();
                let closed_interior = closed_interior.borrow();
                open_interior.clone_from(&*closed_interior);
                open_interior.remove(c);
            }

            // generate exterior - everything at exactly radius 3 (excluding closed interior)
            {
                let mut exterior = exterior.borrow_mut();
                let closed_interior = closed_interior.borrow();
                let open_interior = open_interior.borrow();
                exterior.clear();
                exterior.extend(open_interior.iter().flat_map(|p| Adj::Open::at(*p)));
                for p in closed_interior.iter() {
                    exterior.remove(p);
                }
            }

            // generate farlands - everything at exactly radius 4
            let farlands = {
                let mut farlands: BTreeSet<(isize, isize)> = Default::default();
                let exterior = exterior.borrow();
                let open_interior = open_interior.borrow();
                farlands.clear();
                farlands.extend(exterior.iter().flat_map(|p| Adj::Open::at(*p)));
                for p in open_interior.iter() {
                    farlands.remove(p);
                }
                farlands
            };

            // generate boundary map - maps interior boundary to farlands intersection within radius 2 of itself
            {
                let mut boundary_map = boundary_map.borrow_mut();
                let open_interior = open_interior.borrow();
                let exterior = exterior.borrow();
                boundary_map.clear();
                for &p in open_interior.iter().filter(|&p| Adj::Open::at(*p).any(|x| exterior.contains(&x))) {
                    let close: BTreeSet<_> = Adj::Open::at(p).filter(|x| !open_interior.contains(x)).collect();
                    let far: BTreeSet<_> = close.iter().flat_map(|x| Adj::Open::at(*x)).filter(|x| farlands.contains(x)).collect();

                    let lands = Lands {
                        closelands: close.into_iter().collect(),
                        farlands: far.into_iter().collect(),
                    };
                    boundary_map.insert(p, lands);
                }
            }

            // each pass starts with no detectors except the center
            self.detectors.clear();
            self.detectors.insert(*c);

            // perform center folding
            self.calc_recursive::<Adj, _>(open_interior.borrow().iter().copied());
        }

        // lcm(1..=9) = 2520, so multiply and divide by 2520 to create an exact fractional representation
        let v = (self.highest * 2520.0).round() as usize;
        let d = util::gcd(v, 2520);
        ((2520 / d, v / d), 1.0 / self.highest)
    }
    fn new() -> Self {
        Default::default()
    }
}

struct FiniteGraphSolver<'a, Codes> {
    verts: &'a [Vertex],
    detectors: &'a mut HashSet<usize>,
    needed: usize,
    codes: Codes,
}
impl<Codes> FiniteGraphSolver<'_, Codes>
where Codes: codesets::Set<usize>
{
    fn get_locating_code(&self, p: usize) -> Vec<usize> {
        let mut v = Vec::with_capacity(9);
        for x in &self.verts[p].adj {
            if self.detectors.contains(x) {
                v.push(*x);
            }
        }
        v
    }
    fn is_old(&mut self) -> bool {
        self.codes.clear();
        for i in 0..self.verts.len() {
            let is_detector = self.detectors.contains(&i);
            let code = self.get_locating_code(i);
            if !self.codes.add(is_detector, code) {
                return false;
            }
        }
        true
    }
    fn find_solution_recursive(&mut self, pos: usize) -> bool {
        if self.needed == self.detectors.len() {
            if self.is_old() {
                return true;
            }
        }
        else if pos < self.verts.len() {
            self.detectors.insert(pos);
            if self.find_solution_recursive(pos + 1) {
                return true;
            }
            self.detectors.remove(&pos);
            return self.find_solution_recursive(pos + 1);
        }

        false
    }
    fn find_solution(&mut self, n: usize) -> bool {
        self.detectors.clear();
        self.needed = n;
        self.find_solution_recursive(0)
    }
}

enum GraphLoadError {
    FileOpenFailure,
    InvalidFormat(&'static str),
}
struct Vertex {
    label: String,
    adj: Vec<usize>,
}
struct FiniteGraph {
    verts: Vec<Vertex>,
    detectors: HashSet<usize>,
}
impl FiniteGraph {
    fn with_shape<P: AsRef<Path>>(path: P) -> Result<Self, GraphLoadError> {
        let mut f = BufReader::new(match File::open(path) {
            Ok(x) => x,
            Err(_) => return Err(GraphLoadError::FileOpenFailure),
        });

        struct Vertexish {
            label: String,
            adj: BTreeSet<usize>,
        }
        let mut v: Vec<Vertexish> = vec![];
        let mut m: HashMap<String, usize> = Default::default();

        let get_vert = |verts: &mut Vec<Vertexish>, map: &mut HashMap<String, usize>, a: &str| {
            match map.get(a) {
                Some(&p) => p,
                None => {
                    verts.push(Vertexish {
                        label: a.into(),
                        adj: Default::default(),
                    });
                    let p = verts.len() - 1;
                    map.insert(a.into(), p);
                    p
                }
            }
        };
        let mut add_edge = |a: &str, b: &str| {
            let idx_a = get_vert(&mut v, &mut m, a);
            let idx_b = get_vert(&mut v, &mut m, b);
            v[idx_a].adj.insert(idx_b);
            v[idx_b].adj.insert(idx_a);
        };

        let mut s = String::new();
        while { s.clear(); let r = f.read_line(&mut s); r.is_ok() && r.unwrap() != 0 } {
            for tok in s.split_whitespace() {
                let p = match tok.find(':') {
                    Some(x) => x,
                    None => return Err(GraphLoadError::InvalidFormat("encountered token without a ':' separator")),
                };
                let a = tok[..p].trim();
                let b = tok[p+1..].trim();
                if b.find(':').is_some() {
                    return Err(GraphLoadError::InvalidFormat("encoundered token with multiple ':' separators"));
                }
                if a == b {
                    return Err(GraphLoadError::InvalidFormat("encountered reflexive connection"));
                }
                add_edge(a, b);
            }
        }

        let mut verts: Vec<Vertex> = Vec::with_capacity(v.len());
        for i in v {
            verts.push(Vertex {
                label: i.label,
                adj: i.adj.into_iter().collect(),
            });
        }
        Ok(FiniteGraph {
            verts,
            detectors: Default::default(),
        })
    }
    fn solver<Codes>(&mut self) -> FiniteGraphSolver<'_, Codes>
    where Codes: codesets::Set<usize>
    {
        FiniteGraphSolver {
            verts: &self.verts,
            detectors: &mut self.detectors,
            needed: 0,
            codes: Default::default(),
        }
    }
    fn get_solution(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.detectors.iter().map(|&p| self.verts[p].label.as_str()).collect();
        v.sort();
        v
    }
}

#[test]
fn test_rect_pos() {
    let mut ggg = RectTessellation::new(4, 4);
    let mut gg = ggg.solver::<codesets::OLD<(isize, isize)>>();
    let g = &mut gg.base;

    assert_eq!(g.id_to_inside(0, 0), 0);
    assert_eq!(g.id_to_inside(1, 0), 4);
    assert_eq!(g.id_to_inside(0, 1), 1);
    assert_eq!(g.id_to_inside(2, 1), 9);

    assert_eq!(g.id_to_inside(-1, 0), 12);
    assert_eq!(g.id_to_inside(-1, 2), 14);
    assert_eq!(g.id_to_inside(-1, 4), 12);
    assert_eq!(g.id_to_inside(4, 4), 0);
    assert_eq!(g.id_to_inside(4, 1), 1);
    assert_eq!(g.id_to_inside(4, -1), 3);
    assert_eq!(g.id_to_inside(-1, -1), 15);

    g.src.phase = (1, 0);

    assert_eq!(g.id_to_inside(0, 0), 0);
    assert_eq!(g.id_to_inside(1, 0), 4);
    assert_eq!(g.id_to_inside(0, 1), 1);
    assert_eq!(g.id_to_inside(2, 1), 9);

    assert_eq!(g.id_to_inside(-1, 0), 15);
    assert_eq!(g.id_to_inside(-1, 2), 13);
    assert_eq!(g.id_to_inside(-1, 4), 15);
    assert_eq!(g.id_to_inside(4, 4), 1);
    assert_eq!(g.id_to_inside(4, 1), 2);
    assert_eq!(g.id_to_inside(4, -1), 0);
    assert_eq!(g.id_to_inside(-1, -1), 14);

    g.src.phase = (0, 1);

    assert_eq!(g.id_to_inside(0, 0), 0);
    assert_eq!(g.id_to_inside(1, 0), 4);
    assert_eq!(g.id_to_inside(0, 1), 1);
    assert_eq!(g.id_to_inside(2, 1), 9);

    assert_eq!(g.id_to_inside(-1, 0), 12);
    assert_eq!(g.id_to_inside(-1, 2), 14);
    assert_eq!(g.id_to_inside(-1, 4), 0);
    assert_eq!(g.id_to_inside(4, 4), 4);
    assert_eq!(g.id_to_inside(4, 1), 1);
    assert_eq!(g.id_to_inside(4, -1), 15);
    assert_eq!(g.id_to_inside(-1, -1), 11);
}

fn parse_thresh(v: &str) -> f64 {
    match v.parse::<f64>() {
        Ok(v) if v > 0.0 && v <= 1.0 => v,
        Ok(v) => {
            eprintln!("thresh {} was outside valid range (0, 1]", v);
            std::process::exit(7);
        }
        Err(_) => {
            eprintln!("failed to parse '{}' as float", v);
            std::process::exit(7);
        }
    }
}
fn parse_exact(v: &str, max: usize) -> usize {
    match v.parse::<usize>() {
        Ok(v) if v <= max => v,
        Ok(v) => {
            eprintln!("count {} exceeded max {}", v, max);
            std::process::exit(7);
        }
        Err(_) => {
            eprintln!("failed to parse '{}' as uint", v);
            std::process::exit(7);
        }
    }
}
fn parse_share(v: &str) -> f64 {
    match v.parse::<f64>() {
        Ok(v) if v > 0.0 => v,
        Ok(v) => {
            eprintln!("share {} was outside valid range (0, inf)", v);
            std::process::exit(7);
        }
        Err(_) => {
            eprintln!("failed to parse '{}' as float", v);
            std::process::exit(7);
        }
    }
}

fn tess_helper<T: Tessellation>(mut tess: T, mode: &str, goal: &str) {
    let res = match mode {
        "dom:king" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::ClosedKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "odom:king" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::OpenKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "edom:king" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::ClosedKing>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "eodom:king" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::OpenKing>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "ld:king" => tess.try_satisfy::<codesets::LD<(isize, isize)>, adj::OpenKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "ic:king" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::ClosedKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "redic:king" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::ClosedKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "detic:king" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::ClosedKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "erric:king" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::ClosedKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "old:king" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::OpenKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "red:king" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::OpenKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "det:king" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::OpenKing>(Goal::MeetOrBeat(parse_thresh(goal))),
        "err:king" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::OpenKing>(Goal::MeetOrBeat(parse_thresh(goal))),

        "dom:tri" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::ClosedTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "odom:tri" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::OpenTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "edom:tri" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::ClosedTri>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "eodom:tri" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::OpenTri>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "ld:tri" => tess.try_satisfy::<codesets::LD<(isize, isize)>, adj::OpenTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "ic:tri" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::ClosedTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "redic:tri" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::ClosedTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "detic:tri" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::ClosedTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "erric:tri" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::ClosedTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "old:tri" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::OpenTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "red:tri" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::OpenTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "det:tri" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::OpenTri>(Goal::MeetOrBeat(parse_thresh(goal))),
        "err:tri" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::OpenTri>(Goal::MeetOrBeat(parse_thresh(goal))),

        "dom:grid" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::ClosedGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "odom:grid" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::OpenGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "edom:grid" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::ClosedGrid>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "eodom:grid" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::OpenGrid>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "ld:grid" => tess.try_satisfy::<codesets::LD<(isize, isize)>, adj::OpenGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "ic:grid" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::ClosedGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "redic:grid" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::ClosedGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "detic:grid" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::ClosedGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "erric:grid" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::ClosedGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "old:grid" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::OpenGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "red:grid" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::OpenGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "det:grid" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::OpenGrid>(Goal::MeetOrBeat(parse_thresh(goal))),
        "err:grid" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::OpenGrid>(Goal::MeetOrBeat(parse_thresh(goal))),

        "dom:hex" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::ClosedHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "odom:hex" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::OpenHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "edom:hex" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::ClosedHex>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "eodom:hex" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::OpenHex>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "ld:hex" => tess.try_satisfy::<codesets::LD<(isize, isize)>, adj::OpenHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "ic:hex" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::ClosedHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "redic:hex" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::ClosedHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "detic:hex" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::ClosedHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "erric:hex" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::ClosedHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "old:hex" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::OpenHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "red:hex" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::OpenHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "det:hex" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::OpenHex>(Goal::MeetOrBeat(parse_thresh(goal))),
        "err:hex" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::OpenHex>(Goal::MeetOrBeat(parse_thresh(goal))),

        "dom:tmb" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::ClosedTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "odom:tmb" => tess.try_satisfy::<codesets::DOM<(isize, isize)>, adj::OpenTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "edom:tmb" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::ClosedTMB>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "eodom:tmb" => tess.try_satisfy::<codesets::EDOM<(isize, isize)>, adj::OpenTMB>(Goal::Exactly(parse_exact(goal, tess.size()))),
        "ld:tmb" => tess.try_satisfy::<codesets::LD<(isize, isize)>, adj::OpenTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "ic:tmb" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::ClosedTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "redic:tmb" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::ClosedTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "detic:tmb" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::ClosedTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "erric:tmb" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::ClosedTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "old:tmb" => tess.try_satisfy::<codesets::OLD<(isize, isize)>, adj::OpenTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "red:tmb" => tess.try_satisfy::<codesets::RED<(isize, isize)>, adj::OpenTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "det:tmb" => tess.try_satisfy::<codesets::DET<(isize, isize)>, adj::OpenTMB>(Goal::MeetOrBeat(parse_thresh(goal))),
        "err:tmb" => tess.try_satisfy::<codesets::ERR<(isize, isize)>, adj::OpenTMB>(Goal::MeetOrBeat(parse_thresh(goal))),

        _ => {
            eprintln!("unknown type: {}", mode);
            std::process::exit(4);
        }
    };
    match res {
        Some(min) => {
            let n = tess.size();
            let d = util::gcd(min, n);
            println!("found a {}/{} ({}) solution:\n{}", (min / d), (n / d), (min as f64 / n as f64), tess);
        },
        None => println!("no solution found"),
    }
}
fn theo_helper(mode: &str, thresh: &str) {
    let thresh = parse_share(thresh);
    let ((n, k), f) = match mode {
        "dom:king" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::ClosedKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "odom:king" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::OpenKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ld:king" => LowerBoundSearcher::<codesets::LD<(isize, isize)>>::new().calc::<adj::OpenKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ic:king" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::ClosedKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "redic:king" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::ClosedKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "detic:king" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::ClosedKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "erric:king" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::ClosedKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "old:king" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::OpenKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "red:king" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::OpenKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "det:king" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::OpenKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "err:king" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::OpenKing>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),

        "dom:tri" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::ClosedTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "odom:tri" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::OpenTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ld:tri" => LowerBoundSearcher::<codesets::LD<(isize, isize)>>::new().calc::<adj::OpenTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ic:tri" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::ClosedTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "redic:tri" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::ClosedTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "detic:tri" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::ClosedTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "erric:tri" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::ClosedTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "old:tri" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::OpenTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "red:tri" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::OpenTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "det:tri" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::OpenTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "err:tri" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::OpenTri>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),

        "dom:grid" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::ClosedGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "odom:grid" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::OpenGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ld:grid" => LowerBoundSearcher::<codesets::LD<(isize, isize)>>::new().calc::<adj::OpenGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ic:grid" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::ClosedGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "redic:grid" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::ClosedGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "detic:grid" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::ClosedGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "erric:grid" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::ClosedGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "old:grid" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::OpenGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "red:grid" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::OpenGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "det:grid" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::OpenGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "err:grid" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::OpenGrid>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),

        "dom:hex" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::ClosedHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "odom:hex" => LowerBoundSearcher::<codesets::DOM<(isize, isize)>>::new().calc::<adj::OpenHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ld:hex" => LowerBoundSearcher::<codesets::LD<(isize, isize)>>::new().calc::<adj::OpenHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "ic:hex" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::ClosedHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "redic:hex" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::ClosedHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "detic:hex" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::ClosedHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "erric:hex" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::ClosedHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "old:hex" => LowerBoundSearcher::<codesets::OLD<(isize, isize)>>::new().calc::<adj::OpenHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "red:hex" => LowerBoundSearcher::<codesets::RED<(isize, isize)>>::new().calc::<adj::OpenHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "det:hex" => LowerBoundSearcher::<codesets::DET<(isize, isize)>>::new().calc::<adj::OpenHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),
        "err:hex" => LowerBoundSearcher::<codesets::ERR<(isize, isize)>>::new().calc::<adj::OpenHex>(thresh, Some(&mut io::stdout().lock()), &[(0, 0)]),

        _ => {
            eprintln!("unknown type: {}", mode);
            std::process::exit(4);
        }
    };

    println!("found theo lower bound {}/{} ({})", n, k, f);
}
fn finite_helper(mut g: FiniteGraph, mode: &str, count: usize) {
    let success = match mode {
        "old" => g.solver::<codesets::OLD<usize>>().find_solution(count),
        "red" => g.solver::<codesets::RED<usize>>().find_solution(count),
        "det" => g.solver::<codesets::DET<usize>>().find_solution(count),
        "err" => g.solver::<codesets::ERR<usize>>().find_solution(count),

        _ => {
            eprintln!("unknown type: {}", mode);
            std::process::exit(4);
        }
    };
    if success {
        println!("found solution:\n{:?}", g.get_solution());
    }
    else {
        println!("no solution found");
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let show_usage = |ret| -> ! {
        eprintln!("usage: {} (rect w h|geo shape) (old|det):(king|tri) [thresh]", args[0]);
        std::process::exit(ret);
    };

    if args.len() < 2 { show_usage(1); }
    match args[1].as_str() {
        "finite" => {
            if args.len() < 5 { show_usage(1); }
            let g = match FiniteGraph::with_shape(&args[2]) {
                Ok(g) => g,
                Err(e) => {
                    match e {
                        GraphLoadError::FileOpenFailure => eprintln!("failed to open graph file {}", args[2]),
                        GraphLoadError::InvalidFormat(msg) => eprintln!("file {} was invalid format: {}", args[2], msg),
                    }
                    std::process::exit(5);
                }
            };
            let count = match args[4].parse::<usize>() {
                Ok(n) => {
                    if n == 0 {
                        eprintln!("count cannot be zero");
                        std::process::exit(6);
                    }
                    if n > g.verts.len() {
                        eprintln!("count cannot be larger than graph size");
                        std::process::exit(6);
                    }
                    n
                }
                Err(_) => {
                    eprintln!("failed to parse '{}' as positive integer", args[4]);
                    std::process::exit(7);
                }
            };
            finite_helper(g, &args[3], count);
        }
        "theo" => {
            if args.len() < 4 { show_usage(1); }
            theo_helper(&args[2], &args[3]);
        }
        "rect" => {
            if args.len() < 6 { show_usage(1); }
            let rows: usize = args[2].parse().unwrap();
            let cols: usize = args[3].parse().unwrap();
            if rows < 2 || cols < 2 {
                eprintln!("1x1, 1xn, nx1 are not supported to avoid branch conditions\nthey also cannot result in lower than 2/3");
                std::process::exit(3);
            }
            let tess = RectTessellation::new(rows, cols);
            tess_helper(tess, &args[4], &args[5])
        }
        "geo" => {
            let tess = match GeometryTessellation::with_shape(&args[2]) {
                Ok(x) => x,
                Err(e) => {
                    match e {
                        GeometryLoadResult::FileOpenFailure => eprintln!("failed to open tessellation file {}", args[2]),
                        GeometryLoadResult::InvalidFormat(msg) => eprintln!("file {} was invalid format: {}", args[2], msg),
                        GeometryLoadResult::TessellationFailure(msg) => eprintln!("file {} had a tessellation failure: {}", args[2], msg),
                    };
                    std::process::exit(5);
                },
            };
            println!("loaded geometry:\n{}", tess);
            tess_helper(tess, &args[3], &args[4])
        }
        _ => show_usage(1),
    };
}
