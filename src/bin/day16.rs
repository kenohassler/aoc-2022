use anyhow::{Context, Result};
use itertools::Itertools;
use std::hash::Hash;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    io::Write,
    str::FromStr,
};

const MAX_MINUTES: u32 = 30;

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
struct ValveId([u8; 2]);

impl FromStr for ValveId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bb = s.as_bytes();
        Ok(ValveId([bb[0], bb[1]]))
    }
}

impl From<ValveId> for u16 {
    fn from(value: ValveId) -> Self {
        assert!(value.0[0].is_ascii_uppercase() && value.0[1].is_ascii_uppercase());
        let lhs: u16 = (value.0[0] - 65).into();
        let rhs: u16 = (value.0[1] - 65).into();
        lhs << 5 | rhs
    }
}

impl fmt::Debug for ValveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.0))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Action {
    MoveTo(ValveId),
    Open(ValveId),
}

#[derive(Debug)]
struct PathState {
    actions: Vec<Action>,
    minutes: u32,
    flow_cur: u32,
    flow_acc: u32,
}

impl Clone for PathState {
    #[must_use]
    fn clone(&self) -> Self {
        Self {
            actions: self.actions.clone(),
            minutes: self.minutes + 1,
            flow_cur: self.flow_cur,
            flow_acc: self.flow_acc + self.flow_cur,
        }
    }
}

impl PathState {
    #[must_use]
    fn new() -> Self {
        PathState {
            actions: Vec::new(),
            minutes: 0,
            flow_cur: 0,
            flow_acc: 0,
        }
    }

    fn add_move(&mut self, v_id: ValveId) {
        self.actions.push(Action::MoveTo(v_id));
    }

    fn add_open(&mut self, v: &Valve) {
        assert!(!self.is_open(&v.id), "cannot open an opened valve");

        self.actions.push(Action::Open(v.id));
        self.flow_cur += v.rate;
    }

    fn is_open(&self, v_id: &ValveId) -> bool {
        self.opened().contains(v_id)
    }

    fn opened(&self) -> impl Iterator<Item = &ValveId> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                Action::Open(vid) => Some(vid),
                _ => None,
            })
    }

    fn flow_projected(&self) -> u32 {
        let minutes_left = MAX_MINUTES - self.minutes;
        self.flow_acc + minutes_left * self.flow_cur
    }
}

struct ValveState {
    /// The currently favoured path
    path: PathState,
    /// Other paths to this node (that do not already open the valve)
    alternatives: Vec<PathState>,
}

impl ValveState {
    fn new(path: PathState) -> Self {
        ValveState {
            path,
            alternatives: Vec::new(),
        }
    }
}

impl fmt::Display for PathState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "open ")?;
        f.debug_list().entries(self.opened()).finish()?;
        write!(
            f,
            "; flow {{current {}, total {}, estimated {}}}; ",
            self.flow_cur,
            self.flow_acc,
            self.flow_projected()
        )?;
        match self.actions.last() {
            Some(Action::MoveTo(vid)) => {
                write!(f, "moved to {vid:?}")?;
            }
            Some(Action::Open(vid)) => {
                write!(f, "opened {vid:?}")?;
            }
            _ => (),
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Valve {
    id: ValveId,
    neighbours: Vec<ValveId>,
    rate: u32,
}

impl FromStr for Valve {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, last) = s.split_once(';').context("expected ';' in the middle")?;
        let mut first = first.split_ascii_whitespace();
        let id = first.nth(1).context("id not found")?.parse()?;
        let rate = first
            .last()
            .context("rate not found")?
            .split_once('=')
            .context("expected '=' in rate")?
            .1
            .parse()?;
        let neighbours = last
            .split_ascii_whitespace()
            .skip(4)
            .map(ValveId::from_str)
            .collect::<Result<Vec<ValveId>, _>>()?;
        Ok(Valve {
            id,
            neighbours,
            rate,
        })
    }
}

fn main() -> Result<()> {
    let example = aoc_2022::example(16);
    let g = build_graph(&example)?;
    write_graph(&g, "example-graph")?;
    println!("{}", find_path(&g));

    let input = aoc_2022::input(16);
    let g = build_graph(&input)?;
    write_graph(&g, "input-graph")?;
    println!("{}", find_path(&g));

    Ok(())
}

fn find_path(g: &HashMap<ValveId, Valve>) -> u32 {
    let mut last_state = HashMap::new();
    let start_node: ValveId = "AA".parse().unwrap();
    last_state.insert(start_node, ValveState::new(PathState::new()));

    for min in 0..MAX_MINUTES {
        let mut cur_state = HashMap::new();

        // first iteration: open valves
        for (v_id, v_state) in &last_state {
            debug_assert_eq!(v_state.path.minutes, min);
            let v = g.get(v_id).unwrap();

            let mut best_path = v_state.path.clone();
            if !best_path.is_open(v_id) {
                // open valve
                best_path.add_open(v);
            }

            // check last round's alternatives
            for alt_path in &v_state.alternatives {
                // we know the valve is not yet open
                let mut with_open = alt_path.clone();
                with_open.add_open(v);

                if with_open.flow_projected() > best_path.flow_projected() {
                    best_path = with_open;
                }
            }

            cur_state.insert(*v_id, ValveState::new(best_path));
        }

        // second iteration: move
        for (v_id, v_state) in &last_state {
            debug_assert_eq!(v_state.path.minutes, min);
            let v = g.get(v_id).unwrap();

            for n_id in &v.neighbours {
                let mut with_move = v_state.path.clone();
                with_move.add_move(*n_id);

                // Move to the neighbour node if...
                match cur_state.entry(*n_id) {
                    Entry::Occupied(mut n_state) => {
                        // ...our flow is bigger than the existing flow
                        if with_move.flow_projected() > n_state.get().path.flow_projected() {
                            n_state.get_mut().path = with_move;
                        } else if !with_move.opened().contains(n_id) {
                            // if the neighbour is not yet opened, this might be an alternative path
                            n_state.get_mut().alternatives.push(with_move);
                        }
                    }
                    Entry::Vacant(n_state) => {
                        // ...or the neighbour has not yet been visited.
                        n_state.insert(ValveState::new(with_move));
                    }
                }
            }
        }

        last_state = cur_state;

        // DEBUG
        // eprintln!("new algo: == Minute {} ==", min + 1);
        // for v_id in last_state.keys().sorted() {
        //     eprintln!("{v_id:?}: {}", last_state.get(v_id).unwrap());
        // }
        // eprintln!();
    }

    let mut best = last_state.get(&start_node).unwrap();
    for v_state in last_state.values() {
        if v_state.path.flow_acc > best.path.flow_acc {
            best = v_state;
        }
    }
    eprintln!("{:?}", best.path);

    best.path.flow_acc
}

fn build_graph(input: &str) -> Result<HashMap<ValveId, Valve>> {
    let mut valves_map = HashMap::new();
    for ll in input.lines() {
        let v: Valve = ll.parse()?;
        valves_map.insert(v.id, v);
    }
    Ok(valves_map)
}

/// write the graph to disk in Trivial Graph Format for debugging
fn write_graph(g: &HashMap<ValveId, Valve>, name: &str) -> std::io::Result<()> {
    let mut f = std::fs::File::create(name.to_owned() + ".tgf")?;

    // list of nodes first
    for (vid, v) in g {
        writeln!(f, "{vid:?} {vid:?},{}", v.rate)?;
    }
    // hashtag separator
    writeln!(f, "#")?;
    // list of edges
    for (vid, v) in g {
        for n in &v.neighbours {
            writeln!(f, "{vid:?} {n:?}")?;
        }
    }

    Ok(())
}
