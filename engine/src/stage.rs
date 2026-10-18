//! Generic solver stage: brings a cube into a smaller group using a fixed set of moves.
//!
//! A stage looks at the cube through a few *components*. Each component is a key that
//! depends only on which piece sits in which slot (orientation included), so its value
//! after a move depends only on its value before. That lets us:
//! 1. discover every reachable key by BFS and build a move table per component;
//! 2. BFS the product of the components from the goal, giving the exact distance of
//!    every reachable state to the goal;
//! 3. solve by always playing a move that lowers the distance by one (optimal for the stage).

use std::collections::{HashMap, VecDeque};

use crate::{Cube, Move};

pub type Component = Box<dyn Fn(&Cube) -> u128 + Send + Sync>;

const UNSEEN: u8 = u8::MAX;

struct Table {
    component: Component,
    ids: HashMap<u128, u32>,
    /// `next[id * moves.len() + m]` is the id reached by playing move `m`.
    next: Vec<u32>,
}

pub struct Stage {
    moves: Vec<Move>,
    tables: Vec<Table>,
    dist: Vec<u8>,
}

impl Stage {
    pub fn new(moves: Vec<Move>, components: Vec<Component>) -> Stage {
        let tables: Vec<Table> = components
            .into_iter()
            .map(|c| Table::build(c, &moves))
            .collect();
        let mut stage = Stage {
            moves,
            tables,
            dist: Vec::new(),
        };
        stage.build_distances();
        stage
    }

    /// Number of distinct reachable-looking states (product of component sizes).
    pub fn size(&self) -> usize {
        self.tables.iter().map(|t| t.ids.len()).product()
    }

    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// Longest optimal solution of this stage.
    pub fn max_distance(&self) -> u8 {
        self.dist
            .iter()
            .copied()
            .filter(|&d| d != UNSEEN)
            .max()
            .unwrap_or(0)
    }

    /// Exact number of moves this stage needs from `cube`, or `None` if `cube` is
    /// not in the group this stage starts from.
    pub fn distance(&self, cube: &Cube) -> Option<u8> {
        let d = self.dist[self.index(&self.ids_of(cube)?)];
        (d != UNSEEN).then_some(d)
    }

    /// Optimal move sequence bringing `cube` into the stage's goal group,
    /// or `None` if `cube` is not in the group this stage starts from.
    pub fn solve(&self, cube: &Cube) -> Option<Vec<Move>> {
        let mut ids = self.ids_of(cube)?;
        let mut d = self.dist[self.index(&ids)];
        if d == UNSEEN {
            return None;
        }
        let mut solution = Vec::with_capacity(d as usize);
        while d > 0 {
            let (m, next) = (0..self.moves.len())
                .map(|m| (m, self.step(&ids, m)))
                .find(|(_, next)| self.dist[self.index(next)] == d - 1)
                .expect("exact distance table always has a move one step closer");
            solution.push(self.moves[m]);
            ids = next;
            d -= 1;
        }
        Some(solution)
    }

    fn ids_of(&self, cube: &Cube) -> Option<Vec<u32>> {
        self.tables
            .iter()
            .map(|t| t.ids.get(&(t.component)(cube)).copied())
            .collect()
    }

    fn step(&self, ids: &[u32], m: usize) -> Vec<u32> {
        let n = self.moves.len();
        self.tables
            .iter()
            .zip(ids)
            .map(|(t, &id)| t.next[id as usize * n + m])
            .collect()
    }

    fn index(&self, ids: &[u32]) -> usize {
        self.tables
            .iter()
            .zip(ids)
            .fold(0, |acc, (t, &id)| acc * t.ids.len() + id as usize)
    }

    fn build_distances(&mut self) {
        let mut dist = vec![UNSEEN; self.size()];
        let goal = self
            .ids_of(&Cube::SOLVED)
            .expect("solved cube is always discovered");
        dist[self.index(&goal)] = 0;
        let mut frontier = vec![goal];
        let mut depth = 0;
        while !frontier.is_empty() {
            depth += 1;
            let mut next_frontier = Vec::new();
            for ids in &frontier {
                for m in 0..self.moves.len() {
                    let next = self.step(ids, m);
                    let i = self.index(&next);
                    if dist[i] == UNSEEN {
                        dist[i] = depth;
                        next_frontier.push(next);
                    }
                }
            }
            frontier = next_frontier;
        }
        self.dist = dist;
    }
}

impl Table {
    /// BFS from the solved cube over `moves`, giving every reachable key a dense id.
    fn build(component: Component, moves: &[Move]) -> Table {
        let n = moves.len();
        let mut ids = HashMap::from([(component(&Cube::SOLVED), 0)]);
        let mut reps = vec![Cube::SOLVED];
        let mut next = Vec::new();
        let mut queue = VecDeque::from([0usize]);
        while let Some(id) = queue.pop_front() {
            let rep = reps[id];
            for &m in moves {
                let cube = rep.apply(m);
                let new_id = ids.len() as u32;
                let target = *ids.entry(component(&cube)).or_insert_with(|| {
                    reps.push(cube);
                    queue.push_back(new_id as usize);
                    new_id
                });
                next.push(target);
            }
            debug_assert_eq!(next.len(), (id + 1) * n);
        }
        Table {
            component,
            ids,
            next,
        }
    }
}
