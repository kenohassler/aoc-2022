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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq)]
enum Action {
    MoveTo(ValveId),
    Open(ValveId),
    Wait,
}

#[derive(Clone, Debug)]
struct PathState {
    actions: Vec<Action>,
}

impl PathState {
    #[must_use]
    fn new() -> Self {
        PathState {
            actions: Vec::new(),
        }
    }

    #[must_use]
    fn new_elephant() -> Self {
        PathState {
            actions: vec![Action::Wait; 4],
        }
    }

    #[must_use]
    fn with_move(&self, v_id: ValveId) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::MoveTo(v_id));

        Self { actions }
    }

    #[must_use]
    fn with_open(&self, v: &Valve) -> Self {
        assert!(!self.is_open(&v.id), "cannot open an opened valve");

        let mut actions = self.actions.clone();
        actions.push(Action::Open(v.id));

        Self { actions }
    }

    #[must_use]
    fn with_wait(&self) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::Wait);

        Self { actions }
    }

    fn is_open(&self, v_id: &ValveId) -> bool {
        self.opened().contains(v_id)
    }

    fn opened(&self) -> impl Iterator<Item = &ValveId> {
        self.actions.iter().filter_map(|a| match a {
            Action::Open(vid) => Some(vid),
            _ => None,
        })
    }

    fn minutes(&self) -> u32 {
        self.actions
            .len()
            .try_into()
            .expect("should be between 0 and 30")
    }

    fn current_flow(&self, g: &HashMap<ValveId, Valve>) -> u32 {
        self.opened()
            .fold(0, |sum, v_id| sum + g.get(v_id).unwrap().rate)
    }

    fn total_flow(&self, g: &HashMap<ValveId, Valve>) -> u32 {
        let mut cur_rate = 0;
        let mut sum = 0;
        for act in &self.actions {
            sum += cur_rate;
            if let Action::Open(v_id) = act {
                cur_rate += g.get(&v_id).unwrap().rate;
            }
        }
        sum
    }

    fn flow_projected(&self, g: &HashMap<ValveId, Valve>) -> u32 {
        let minutes_left = MAX_MINUTES - self.minutes();
        self.total_flow(g) + minutes_left * self.current_flow(g)
    }
}

#[derive(Clone)]
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

        write!(f, "; history ")?;
        let mut history = Vec::new();
        let mut last_v = "AA".parse().unwrap();
        for act in &self.actions {
            if let Action::MoveTo(vid) = act {
                last_v = *vid;
            }
            history.push(last_v);
        }
        f.debug_list().entries(history).finish()?;

        Ok(())
    }
}

fn find_path_elephant(g: &HashMap<ValveId, Valve>) -> u32 {
    let start_node: ValveId = "AA".parse().unwrap();

    let mut human_best = PathState::new_elephant();
    let mut elephant_best = PathState::new_elephant();
    let mut max_flow = 0;

    let mut last_state = HashMap::new();
    last_state.insert(start_node, ValveState::new(PathState::new_elephant()));
    for min in 4..MAX_MINUTES {
        simulate_step(g, &mut last_state, min);

        // iterate over best state for all nodes
        for v_state in last_state.values() {
            // calculate flow by human
            let human_flow = v_state.path.flow_projected(g);

            // graph without valves already opened by the human
            let mut g_clone = g.clone();
            for v_id in v_state.path.opened() {
                g_clone.get_mut(v_id).unwrap().rate = 0;
            }

            let mut last_elephant_state = HashMap::new();
            last_elephant_state.insert(start_node, ValveState::new(PathState::new_elephant()));
            for jj in 4..min + 1 {
                simulate_step(&g_clone, &mut last_elephant_state, jj);
            }

            // calculate best elephant path for this round
            let mut round_elephant_best = last_elephant_state.get(&start_node).unwrap();
            for v_state in last_elephant_state.values() {
                if v_state.path.flow_projected(g) > round_elephant_best.path.flow_projected(g) {
                    round_elephant_best = v_state;
                }
            }

            // if this is a new best overall, replace
            let elephant_flow = round_elephant_best.path.flow_projected(g);
            if elephant_flow + human_flow > max_flow {
                elephant_best = round_elephant_best.path.clone();
                human_best = v_state.path.clone();
                max_flow = human_flow + elephant_flow;
            }
        }
    }

    eprintln!("best: {}, elephant: {}", human_best, elephant_best);
    debug_assert_eq!(
        human_best
            .opened()
            .filter(|v| elephant_best.opened().contains(v))
            .count(),
        0,
        "elephant and human must not open the same valves"
    );

    max_flow
}

fn find_path_solo(g: &HashMap<ValveId, Valve>) -> u32 {
    let mut last_state = HashMap::new();
    let start_node: ValveId = "AA".parse().unwrap();
    last_state.insert(start_node, ValveState::new(PathState::new()));

    for min in 0..MAX_MINUTES {
        simulate_step(g, &mut last_state, min);
    }

    let mut best = last_state.get(&start_node).unwrap();
    for v_state in last_state.values() {
        if v_state.path.total_flow(g) > best.path.total_flow(g) {
            best = v_state;
        }
    }
    eprintln!("{}", best.path);

    best.path.total_flow(g)
}

fn simulate_step(
    g: &HashMap<ValveId, Valve>,
    last_state: &mut HashMap<ValveId, ValveState>,
    min: u32,
) {
    let mut cur_state = HashMap::new();

    // first iteration: open valves
    for (v_id, v_state) in &*last_state {
        debug_assert_eq!(v_state.path.minutes(), min);
        let v = g.get(v_id).unwrap();

        let mut best_path;
        if !v_state.path.is_open(v_id) && v.rate > 0 {
            // open valve
            best_path = v_state.path.with_open(v);
        } else {
            best_path = v_state.path.with_wait();
        }

        // check last round's alternatives
        for alt_path in &v_state.alternatives {
            // we know the valve is not yet open
            let with_open = alt_path.with_open(v);

            if with_open.flow_projected(g) > best_path.flow_projected(g) {
                best_path = with_open;
            }
        }

        cur_state.insert(*v_id, ValveState::new(best_path));
    }

    // second iteration: move
    for (v_id, v_state) in &*last_state {
        debug_assert_eq!(v_state.path.minutes(), min);
        let v = g.get(v_id).unwrap();

        for n_id in &v.neighbours {
            let with_move = v_state.path.with_move(*n_id);

            // Move to the neighbour node if...
            match cur_state.entry(*n_id) {
                Entry::Occupied(mut n_state) => {
                    // ...our flow is bigger than the existing flow
                    if with_move.flow_projected(g) > n_state.get().path.flow_projected(g) {
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

    *last_state = cur_state;
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

fn main() -> Result<()> {
    let example = aoc_2022::example(16);
    let g = build_graph(&example)?;
    write_graph(&g, "example-graph")?;
    println!("{}", find_path_solo(&g));
    println!("{}", find_path_elephant(&g));

    let input = aoc_2022::input(16);
    let g = build_graph(&input)?;
    write_graph(&g, "input-graph")?;
    println!("{}", find_path_solo(&g));
    println!("{}", find_path_elephant(&g));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn part1_example() {
        let example = aoc_2022::example(16);
        let g = build_graph(&example).unwrap();
        assert_eq!(find_path_solo(&g), 1651);
    }

    #[test]
    fn part1_input() {
        let input = aoc_2022::input(16);
        let g = build_graph(&input).unwrap();
        assert_eq!(find_path_solo(&g), 1871);
    }

    #[test]
    fn part2_example() {
        let example = aoc_2022::example(16);
        let g = build_graph(&example).unwrap();
        assert_eq!(find_path_elephant(&g), 1707);
    }

    #[test]
    fn part2_input() {
        let input = aoc_2022::input(16);
        let g = build_graph(&input).unwrap();
        assert_eq!(find_path_elephant(&g), 2416);
    }
}
