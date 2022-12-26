use anyhow::{Context, Result};
use itertools::Itertools;
use std::{collections::HashMap, fmt, str::FromStr};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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
        (Into::<u16>::into(value.0[0])) << 8 | (Into::<u16>::into(value.0[1]))
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
    mins: u32,
}

impl PathState {
    fn new() -> Self {
        PathState {
            actions: Vec::new(),
            flow_cur: 0,
            flow_acc: 0,
            mins: 0,
        }
    }

    #[must_use]
    fn with_move(&self, vid: ValveId) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::MoveTo(vid));
        let flow_cur = self.flow_cur;
        let flow_acc = self.flow_acc + self.flow_cur;
        let mins = self.mins + 1;

        PathState {
            actions,
            flow_cur,
            flow_acc,
            mins,
        }
    }

    #[must_use]
    fn with_open(&self, v: &Valve) -> Option<Self> {
        if self.is_opened(v.id) {
            return None;
        }

        let mut actions = self.actions.clone();
        actions.push(Action::Open(v.id));
        let flow_cur = self.flow_cur + v.rate;
        let flow_acc = self.flow_acc + self.flow_cur;
        let mins = self.mins + 1;

        Some(PathState {
            actions,
            flow_cur,
            flow_acc,
            mins,
        })
    }

    #[must_use]
    fn with_wait(&self) -> Self {
        let mut actions = self.actions.clone();
        actions.push(Action::Wait);
        let flow_cur = self.flow_cur;
        let flow_acc = self.flow_acc + self.flow_cur;
        let mins = self.mins + 1;

        PathState {
            actions,
            flow_cur,
            flow_acc,
            mins,
        }
    }

    fn is_opened(&self, vid: ValveId) -> bool {
        self.actions.iter().contains(&Action::Open(vid))
    }

    fn eventual_flow(&self) -> u32 {
        let mins_left = 30 - self.mins;
        self.flow_acc + mins_left * self.flow_cur
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
        let id = first.nth(1).context("id not found").unwrap().parse()?;
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

impl Valve {}

fn main() -> Result<()> {
    let example = aoc_2022::example(16);
    println!("{}", find_path(&example)?);

    let input = aoc_2022::input(16);
    println!("{}", find_path(&input)?);

    Ok(())
}

fn find_path(input: &str) -> Result<u32> {
    let mut valves_map = HashMap::new();
    for ll in input.lines() {
        let v: Valve = ll.parse()?;
        valves_map.insert(v.id, v);
    }

    let mut state = HashMap::new();
    let start = "AA".parse()?;
    for vid in valves_map.keys() {
        state.insert(vid, None);
    }
    state.insert(&start, Some(PathState::new()));

    for min in 0..30 {
        // clone here, since we want to change the underlying hashmap... alternatives?
        for (v_id, v_state) in state.clone() {
            if let Some(v_state) = v_state {
                assert!(v_state.mins == min);
                let v = valves_map.get(v_id).unwrap();
                for n in &v.neighbours {
                    let with_move = v_state.with_move(*n);
                    match state.get(n).unwrap() {
                        Some(n_state) => {
                            if with_move.eventual_flow() > n_state.eventual_flow() {
                                state.insert(n, Some(with_move));
                            }
                        }
                        None => {
                            state.insert(n, Some(with_move));
                        }
                    }
                }

                if let Some(with_open) = v_state.with_open(v) {
                    let cur_v_state = state.get(v_id).unwrap().as_ref().unwrap(); // we need the current state here, could have been updated above. unwrapping is safe bc we checked with if let above
                    if with_open.eventual_flow() > cur_v_state.eventual_flow() {
                        // greater here s.th. useless valves stay closed
                        state.insert(v_id, Some(with_open));
                    }
                }
                // stay here, wait
                let with_wait = v_state.with_wait();
                let cur_v_state = state.get(v_id).unwrap().as_ref().unwrap(); // we need the current state here, could have been updated above. unwrapping is safe bc we checked with if let above
                if with_wait.eventual_flow() >= cur_v_state.eventual_flow() {
                    // greater or equal here s.th. we wait at AA even if the flow is 0
                    state.insert(v_id, Some(with_wait));
                }
            }
        }
        //println!("state after min. {min}: {state:?}")
    }

    let mut best = state.get(&start).unwrap().as_ref().unwrap().clone();
    for (_, v_state) in state {
        if v_state.as_ref().unwrap().flow_acc > best.flow_acc {
            best = v_state.unwrap().clone()
        }
    }

    println!("{:#?}", best);
    Ok(best.flow_acc)
}
