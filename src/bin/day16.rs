use anyhow::{Context, Result};
use itertools::Itertools;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    io::Write,
    str::FromStr,
};
use std::hash::Hash;

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
    Wait,
}

#[derive(Clone, Debug)]
struct PathState {
    actions: Vec<Action>,
    flow_cur: u32,
    flow_acc: u32,
}

impl PathState {
    #[must_use]
    fn new() -> Self {
        PathState {
            actions: Vec::new(),
            flow_cur: 0,
            flow_acc: 0,
        }
    }

    #[must_use]
    fn with_move(&self, vid: ValveId) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::MoveTo(vid));
        let flow_cur = self.flow_cur;
        let flow_acc = self.flow_acc + self.flow_cur;

        PathState {
            actions,
            flow_cur,
            flow_acc,
        }
    }

    #[must_use]
    fn with_open(&self, v: &Valve) -> Option<Self> {
        if self.opened().contains(&v.id) {
            return None;
        }

        let mut actions = self.actions.clone();
        actions.push(Action::Open(v.id));
        let flow_cur = self.flow_cur + v.rate;
        let flow_acc = self.flow_acc + self.flow_cur;

        Some(PathState {
            actions,
            flow_cur,
            flow_acc,
        })
    }

    #[must_use]
    fn with_wait(&self) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::Wait);
        let flow_cur = self.flow_cur;
        let flow_acc = self.flow_acc + self.flow_cur;

        PathState {
            actions,
            flow_cur,
            flow_acc,
        }
    }

    fn opened(&self) -> impl Iterator<Item = &ValveId> {
        self.actions.iter().filter_map(|a| match a {
            Action::Open(vid) => Some(vid),
            _ => None,
        })
    }

    fn mins(&self) -> u32 {
        self.actions
            .len()
            .try_into()
            .expect("should be between 0 and 30")
    }

    fn eventual_flow(&self) -> u32 {
        let mins_left = 30 - self.mins();
        self.flow_acc + mins_left * self.flow_cur
    }
}

struct ValveState {
    /// The currently favoured path
    flow: PathState,
    /// Other paths to this node (that do not already open the valve)
    alternatives: Vec<PathState>,
}

impl ValveState {
    fn new(path: PathState) -> Self {
        ValveState{flow: path, alternatives: Vec::new()}
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
            self.eventual_flow()
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
    println!("{}", find_path_new(&g));

    let input = aoc_2022::input(16);
    let g = build_graph(&input)?;
    write_graph(&g, "input-graph")?;
    println!("{}", find_path(&g));
    println!("{}", find_path_new(&g));

    Ok(())
}

fn find_path_new(g: &HashMap<ValveId, Valve>) -> u32 {
    let mut history = Vec::with_capacity(30);
    let mut start_state = HashMap::new();
    let start_node: ValveId = "AA".parse().unwrap();
    start_state.insert(start_node, ValveState::new(PathState::new()));

    let mut last_state = &start_state;
    for min in 0..30 {
        let mut cur_state = HashMap::new();

        for (v_id, v_state) in last_state {
            let v = g.get(v_id).unwrap();

            match v_state.flow.with_open(v) {
                Some(with_open) => cur_state.insert(*v_id, ValveState::new(with_open)),
                None =>
                    cur_state.insert(*v_id, ValveState::new(v_state.flow.with_wait())),
            };

            // prune last round's alternatives
            for alt in &v_state.alternatives {
                // we know the valve is not yet open
                let with_open = alt.with_open(v).unwrap();
                // ToDo: use Entry instead
                if with_open.eventual_flow() > cur_state.get(v_id).unwrap().flow.eventual_flow() {
                    cur_state.get_mut(v_id).unwrap().flow = with_open;
                }
            }
        }

        for (v_id, v_state) in last_state {
            let v = g.get(v_id).unwrap();

            for n_id in &v.neighbours {
                let with_move = v_state.flow.with_move(*n_id);
                match cur_state.entry(*n_id) {
                    Entry::Occupied(mut n_state) => {
                        if with_move.eventual_flow() > n_state.get().flow.eventual_flow() {
                            n_state.get_mut().flow = with_move;
                        } else if !with_move.opened().contains(n_id){
                            // if the neighbour is not yet opened, this might be an alternative path
                            n_state.get_mut().alternatives.push(with_move);
                        }
                    }
                    Entry::Vacant(n_state) => {
                        n_state.insert(ValveState::new(with_move));
                    }
                }

            }
        }

        history.push(cur_state);
        last_state = history.last().unwrap();

        // DEBUG
        //eprintln!("new algo: == Minute {} ==", min + 1);
        for v_id in last_state.keys().sorted() {
            //eprintln!("{v_id:?}: {}", last_state.get(v_id).unwrap());
        }
        //eprintln!();
    }

    let mut best = last_state.get(&start_node).unwrap();
    for v_state in last_state.values() {
        if v_state.flow.flow_acc > best.flow.flow_acc {
            best = v_state;
        }
    }
    eprintln!("{:?}", best.flow);

    best.flow.flow_acc
}

fn find_path(g: &HashMap<ValveId, Valve>) -> u32 {
    let mut state = HashMap::new();
    let start = "AA".parse().unwrap();
    state.insert(&start, PathState::new());

    for min in 0..30 {
        // clone here => preserve state of the last minute
        for (v_id, v_state) in state.clone() {
            assert_eq!(v_state.mins(), min);

            let v = g.get(v_id).unwrap();
            for n_id in &v.neighbours {
                let with_move = v_state.with_move(*n_id);
                // Replace the neighbour's state ...
                match state.entry(n_id) {
                    // ... if its expected flow is lower than ours.
                    Entry::Occupied(mut n_state) => {
                        if with_move.eventual_flow() > n_state.get().eventual_flow() {
                            n_state.insert(with_move);
                        }
                    }
                    // ... if it doesn't have a state yet.
                    Entry::Vacant(n_state) => {
                        n_state.insert(with_move);
                    }
                }
            }

            let mut cur_v_state = match state.entry(v_id) {
                Entry::Occupied(cur_v_state) => cur_v_state,
                _ => panic!("v_id must be in the state map"),
            };

            if let Some(with_open) = v_state.with_open(v) {
                if with_open.eventual_flow() > cur_v_state.get().eventual_flow() {
                    // open current valve (valves with rate=0 stay closed)
                    assert!(v.rate > 0);
                    cur_v_state.insert(with_open);
                }
            }
            // wait at current valve (nop)
            let with_wait = v_state.with_wait();
            if with_wait.eventual_flow() >= cur_v_state.get().eventual_flow() {
                // the eventual flow should never increase by waiting
                assert_eq!(with_wait.eventual_flow(), cur_v_state.get().eventual_flow());
                cur_v_state.insert(with_wait);
            }
        }

        // DEBUG
        eprintln!("== Minute {} ==", min + 1);
        for v_id in state.keys().sorted() {
            eprintln!("{v_id:?}: {}", state.get(v_id).unwrap());
        }
        eprintln!();
    }

    let mut best = state.get(&start).unwrap();
    for v_state in state.values() {
        if v_state.flow_acc > best.flow_acc {
            best = v_state;
        }
    }
    eprintln!("{:?}", best);

    best.flow_acc
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
