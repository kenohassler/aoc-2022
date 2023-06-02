use anyhow::{Context, Result};
use itertools::Itertools;
use std::hash::Hash;
use std::{collections::HashMap, fmt, io::Write, str::FromStr};

const MAX_MINUTES: u32 = 30;

#[derive(Debug, Clone, Eq)]
struct ValveId {
    id: Option<usize>,
    label: [u8; 2],
}

impl ValveId {
    fn numeric(&self) -> usize {
        self.id
            .expect("The numeric ID is guaranteed to exist by Network::build")
    }
}

impl FromStr for ValveId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bb = s.as_bytes();
        Ok(ValveId {
            id: None,
            label: [bb[0], bb[1]],
        })
    }
}

impl PartialEq for ValveId {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Hash for ValveId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state)
    }
}

impl fmt::Display for ValveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.label))
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

#[derive(Clone)]
struct Network {
    nodes: Vec<Valve>,
}

impl Network {
    fn build(input: &str) -> Result<Self> {
        let mut valves_map = HashMap::new();
        for ll in input.lines() {
            let v: Valve = ll.parse()?;
            valves_map.insert(v.id.clone(), v);
        }

        let mut valves_vec = Vec::new();
        // sort labels, resolve IDs
        for (i, (_, v)) in valves_map
            .iter_mut()
            .sorted_by_key(|(k, _)| k.label)
            .enumerate()
        {
            v.id.id = Some(i);
            valves_vec.push(v.clone());
        }

        // convert edges
        for v in &mut valves_vec {
            for n in &mut v.neighbours {
                let id = valves_map.get(n).unwrap().id.numeric();
                n.id = Some(id);
            }
        }

        Ok(Self { nodes: valves_vec })
    }

    fn node(&self, id: usize) -> Option<&Valve> {
        // self.map.get(&id)
        self.nodes.get(id)
    }

    // ToDo: remove
    fn node_mut(&mut self, id: usize) -> Option<&mut Valve> {
        // self.map.get_mut(&id)
        self.nodes.get_mut(id)
    }

    fn nodes(&self) -> impl Iterator<Item = &Valve> {
        self.nodes.iter()
    }

    /// write the graph to disk in Trivial Graph Format for debugging
    fn write_tgf(&self, name: &str) -> std::io::Result<()> {
        let mut f = std::fs::File::create(name.to_owned() + ".tgf")?;

        // list of nodes first
        for v in &self.nodes {
            writeln!(f, "{} {},{}", v.id.numeric(), v.id, v.rate)?;
        }
        // hashtag separator
        writeln!(f, "#")?;
        // list of edges
        for v in &self.nodes {
            for n in &v.neighbours {
                writeln!(f, "{} {}", v.id.numeric(), n.id.unwrap())?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
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
    fn with_move(&self, v_id: &ValveId) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::MoveTo(v_id.clone()));

        Self { actions }
    }

    #[must_use]
    fn with_open(&self, v: &Valve) -> Self {
        assert!(!self.is_open(v.id.numeric()), "cannot open an opened valve");

        let mut actions = self.actions.clone();
        actions.push(Action::Open(v.id.clone()));

        Self { actions }
    }

    #[must_use]
    fn with_wait(&self) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::Wait);

        Self { actions }
    }

    fn is_open(&self, num: usize) -> bool {
        self.opened().any(|v_id| v_id.numeric() == num)
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

    fn current_flow(&self, g: &Network) -> u32 {
        self.opened()
            .fold(0, |sum, v_id| sum + g.node(v_id.numeric()).unwrap().rate)
    }

    fn total_flow(&self, g: &Network) -> u32 {
        let mut cur_rate = 0;
        let mut sum = 0;
        for act in &self.actions {
            sum += cur_rate;
            if let Action::Open(v_id) = act {
                cur_rate += g.node(v_id.numeric()).unwrap().rate;
            }
        }
        sum
    }

    fn projected_flow(&self, g: &Network) -> u32 {
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
        write!(f, "open: ")?;
        for open in self.opened() {
            write!(f, "{} ", open)?;
        }

        write!(f, "\thistory: ")?;
        let start_v = "AA".parse().unwrap();
        let mut last_v = &start_v;
        for act in &self.actions {
            if let Action::MoveTo(vid) = act {
                last_v = vid;
            }
            write!(f, "{} ", last_v)?;
        }

        Ok(())
    }
}

fn find_path_elephant(g: &Network) -> u32 {
    let mut human_best = PathState::new_elephant();
    let mut elephant_best = PathState::new_elephant();
    let mut max_flow = 0;

    let mut last_state = vec![None; g.nodes().count()];
    last_state[0] = Some(ValveState::new(PathState::new_elephant()));
    for min in 4..MAX_MINUTES {
        simulate_step(g, &mut last_state, min);

        // iterate over best state for all nodes
        for v_state in last_state.iter().flatten() {
            // calculate flow by human
            let human_flow = v_state.path.projected_flow(g);

            // graph without valves already opened by the human
            let mut g_clone = g.clone();
            for v_id in v_state.path.opened() {
                g_clone.node_mut(v_id.numeric()).unwrap().rate = 0;
            }

            let mut last_elephant_state = vec![None; g.nodes().count()];
            last_elephant_state[0] = Some(ValveState::new(PathState::new_elephant()));
            for jj in 4..min + 1 {
                simulate_step(&g_clone, &mut last_elephant_state, jj);
            }

            // calculate best elephant path for this round
            let mut round_elephant_best = last_elephant_state[0].as_ref().unwrap();
            for v_state in last_elephant_state.iter().flatten() {
                if v_state.path.projected_flow(g) > round_elephant_best.path.projected_flow(g) {
                    round_elephant_best = v_state;
                }
            }

            // if this is a new best overall, replace
            let elephant_flow = round_elephant_best.path.projected_flow(g);
            if elephant_flow + human_flow > max_flow {
                elephant_best = round_elephant_best.path.clone();
                human_best = v_state.path.clone();
                max_flow = human_flow + elephant_flow;
            }
        }
    }

    eprintln!("best      {human_best}");
    eprintln!("elephant  {elephant_best}");
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

fn find_path_solo(g: &Network) -> u32 {
    let mut last_state = vec![None; g.nodes().count()];
    last_state[0] = Some(ValveState::new(PathState::new()));

    for min in 0..MAX_MINUTES {
        simulate_step(g, &mut last_state, min);
    }

    let mut best = last_state[0].as_ref().unwrap();
    for v_state in last_state.iter().flatten() {
        if v_state.path.total_flow(g) > best.path.total_flow(g) {
            best = v_state;
        }
    }
    eprintln!("{}", best.path);

    best.path.total_flow(g)
}

fn simulate_step(g: &Network, last_state: &mut Vec<Option<ValveState>>, min: u32) {
    let mut cur_state = vec![None; last_state.len()];

    // first iteration: open valves
    for (v_id, v_state) in last_state.iter().enumerate() {
        if let Some(v_state) = v_state {
            debug_assert_eq!(v_state.path.minutes(), min);
            let v = g.node(v_id).unwrap();

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

                if with_open.projected_flow(g) > best_path.projected_flow(g) {
                    best_path = with_open;
                }
            }

            cur_state[v_id] = Some(ValveState::new(best_path));
        }
    }

    // second iteration: move
    for (v_id, v_state) in last_state.iter().enumerate() {
        if let Some(v_state) = v_state {
            debug_assert_eq!(v_state.path.minutes(), min);
            let v = g.node(v_id).unwrap();

            for n_id in &v.neighbours {
                let with_move = v_state.path.with_move(n_id);

                // Move to the neighbour node if...
                match cur_state[n_id.numeric()].as_mut() {
                    Some(n_state) => {
                        // ...our flow is bigger than the existing flow
                        if with_move.projected_flow(g) > n_state.path.projected_flow(g) {
                            n_state.path = with_move;
                        } else if !with_move.opened().contains(&n_id) {
                            // if the neighbour is not yet opened, this might be an alternative path
                            n_state.alternatives.push(with_move);
                        }
                    }
                    None => {
                        // ...or the neighbour has not yet been visited.
                        cur_state[n_id.numeric()] = Some(ValveState::new(with_move));
                    }
                }
            }
        }
    }

    *last_state = cur_state;
}

fn main() -> Result<()> {
    let example = aoc_2022::example(16);
    let g = Network::build(&example)?;
    g.write_tgf("example-graph")?;
    println!("{}", find_path_solo(&g));
    println!("{}", find_path_elephant(&g));

    let input = aoc_2022::input(16);
    let g = Network::build(&input)?;
    g.write_tgf("input-graph")?;
    println!("{}", find_path_solo(&g));
    println!("{}", find_path_elephant(&g));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

    use crate::*;

    #[test]
    fn part1_example() {
        let example = aoc_2022::example(16);
        let g = Network::build(&example).unwrap();
        assert_eq!(find_path_solo(&g), 1651);
    }

    #[test]
    fn part1_input() {
        let input = aoc_2022::input(16);
        let g = Network::build(&input).unwrap();
        assert_eq!(find_path_solo(&g), 1871);
    }

    #[test]
    fn part2_example() {
        let example = aoc_2022::example(16);
        let g = Network::build(&example).unwrap();
        assert_eq!(find_path_elephant(&g), 1707);
    }

    #[test]
    fn part2_input() {
        let input = aoc_2022::input(16);
        let g = Network::build(&input).unwrap();
        assert_eq!(find_path_elephant(&g), 2416);
    }

    #[test]
    fn valveid_equality() {
        let mut v1: ValveId = "AA".parse().unwrap();
        v1.id = Some(42);
        let mut v2: ValveId = "AA".parse().unwrap();
        v2.id = Some(21);
        assert_eq!(v1, v2);

        let v3: ValveId = "AA".parse().unwrap();
        assert_eq!(v1, v3);
        let v4: ValveId = "XX".parse().unwrap();
        assert_ne!(v1, v4);
    }

    #[test]
    fn valveid_hash_equality() {
        fn hash(v: &ValveId) -> u64 {
            let mut hasher = DefaultHasher::new();
            v.hash(&mut hasher);
            hasher.finish()
        }

        let mut v1: ValveId = "AA".parse().unwrap();
        v1.id = Some(42);
        let mut v2: ValveId = "AA".parse().unwrap();
        v2.id = Some(21);
        assert_eq!(hash(&v1), hash(&v2));

        let v3: ValveId = "AA".parse().unwrap();
        assert_eq!(hash(&v1), hash(&v3));
        let v4: ValveId = "XX".parse().unwrap();
        assert_ne!(hash(&v1), hash(&v4));
        println!("{}", hash(&v4));
    }
}
