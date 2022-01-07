mod internal {
    use std::collections::{VecDeque, BinaryHeap};
    use std::cmp::Ordering;
    use std::fmt::Debug;
    use std::hash::Hash;
    use ahash::{AHashMap, AHashSet};

    // References:
    // https://www.redblobgames.com/pathfinding/a-star/introduction.html
    // http://theory.stanford.edu/~amitp/GameProgramming/AStarComparison.html
    // https://doc.rust-lang.org/std/collections/binary_heap/
    pub trait Graph {
        // TODO at least sometimes the default Hash impl is surprisingly expensive (see Day 23).
        //    Should benchmark using BTreeMap/Set to see if that's preferable, or look into revising
        //    the Hash impls of all Nodes.
        type Node: Clone + Debug + Eq + Hash;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>>;

        fn bfs_all(&self, start: &Self::Node) -> AHashMap<Self::Node, Vec<Self::Node>> {
            let mut frontier = VecDeque::new();
            frontier.push_back(start.clone());
            let mut routes = AHashMap::new();
            routes.insert(start.clone(), start.clone()); // careful, potential infinite loop

            while ! frontier.is_empty() {
                let current = frontier.pop_front().expect("frontier is not empty");
                for edge in self.neighbors(&current) {
                    assert!(edge.weight() == 1, "BFS does not support weighted edges: {:?}", edge);
                    let next = edge.dest();
                    if !routes.contains_key(next) {
                        frontier.push_back(next.clone());
                        routes.insert(next.clone(), current.clone());
                    }
                }
            }

            let mut paths = AHashMap::new();
            'outer: for goal in routes.keys() {
                let mut path = Vec::new();
                let mut current = goal.clone();
                while current != *start {
                    if let Some(next) = routes.get(&current) {
                        path.push(current);
                        current = next.clone();
                    } else {
                        continue 'outer;
                    }
                }
                path.push(start.clone());
                path.reverse();
                paths.insert(goal.clone(), path);
            }
            paths
        }

        fn bfs(&self, start: &Self::Node, mut goal_predicate: impl FnMut(&Self::Node) -> bool) -> Option<Vec<Self::Node>> {
            let mut frontier = VecDeque::new();
            frontier.push_back(start.clone());
            let mut routes = AHashMap::new();
            let mut goal = None;

            while let Some(current) = frontier.pop_front() {
                if goal_predicate(&current) {
                    goal = Some(current.clone());
                    break;
                }
                for edge in self.neighbors(&current) {
                    assert!(edge.weight() == 1, "BFS does not support weighted edges: {:?}", edge);
                    let next = edge.dest();
                    if !routes.contains_key(next) {
                        frontier.push_back(next.clone());
                        routes.insert(next.clone(), current.clone());
                    }
                }
            }

            let mut current = goal?;
            let mut path = Vec::new();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(current);
                    current = next.clone();
                } else {
                    unreachable!();
                }
            }
            path.push(start.clone());
            path.reverse();
            Some(path)
        }

        fn dijkstras(&self, start: &Self::Node, mut goal_predicate: impl FnMut(&Self::Node) -> bool) -> Option<Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut visited = AHashSet::new();
            let mut costs = AHashMap::new();
            let mut routes = AHashMap::new();
            let mut goal = None;
            frontier.push(State { cost: 0, node: start.clone() });
            costs.insert(start.clone(), 0);

            while let Some(current) = frontier.pop() {
                if goal_predicate(&current.node) {
                    goal = Some(current.node.clone());
                    break;
                }
                if visited.contains(&current.node) { continue; }
                visited.insert(current.node.clone());
                debug_assert_eq!(Some(&current.cost), costs.get(&current.node));
                for edge in self.neighbors(&current.node) {
                    let next = edge.dest();
                    let next_cost = current.cost + edge.weight();

                    let prior_next_cost = costs.get(next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        costs.insert(next.clone(), next_cost);
                        frontier.push(State { cost: next_cost, node: next.clone() });
                        routes.insert(next.clone(), edge.clone());
                    }
                }
            }

            let mut current = goal?;
            let mut path = Vec::new();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(next.clone());
                    current = next.source().clone();
                } else {
                    unreachable!();
                }
            }
            path.reverse();
            Some(path)
        }

        fn dijkstras_all(&self, start: &Self::Node) -> AHashMap<Self::Node, Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut costs = AHashMap::new();
            let mut routes = AHashMap::new();
            frontier.push(State { cost: 0, node: start.clone() });
            costs.insert(start.clone(), 0);
            routes.insert(start.clone(),
                          Edge::new(0, start.clone(), start.clone())); // careful, potential infinite loop

            while let Some(current) = frontier.pop() {
                debug_assert_eq!(Some(&current.cost), costs.get(&current.node));
                for edge in self.neighbors(&current.node) {
                    let next = edge.dest();
                    let next_cost = current.cost + edge.weight();

                    let prior_next_cost = costs.get(next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        costs.insert(next.clone(), next_cost);
                        frontier.push(State { cost: next_cost, node: next.clone() });
                        routes.insert(next.clone(), edge.clone());
                    }
                }
            }

            let mut paths = AHashMap::new();
            for goal in routes.keys() {
                let mut path = Vec::new();
                let mut current = goal.clone();
                while current != *start {
                    if let Some(next) = routes.get(&current) {
                        path.push(next.clone());
                        current = next.source().clone();
                    } else {
                        unreachable!();
                    }
                }
                path.reverse();
                paths.insert(goal.clone(), path);
            }
            paths
        }

        fn a_star(&self, start: &Self::Node, mut goal_predicate: impl FnMut(&Self::Node) -> bool, heuristic: impl Fn(&Self::Node) -> i32) -> Option<Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut visited = AHashSet::new();
            let mut costs = AHashMap::new();
            let mut est_costs = AHashMap::new();
            let mut routes = AHashMap::new();
            let mut goal = None;
            let start_state = EstState { est_cost: heuristic(start), real_cost: 0, node: start.clone() };
            costs.insert(start.clone(), start_state.real_cost);
            est_costs.insert(start.clone(), start_state.est_cost);
            frontier.push(start_state);

            while let Some(current) = frontier.pop() {
                if goal_predicate(&current.node) {
                    goal = Some(current.node.clone());
                    break;
                }

                if visited.contains(&current.node) { continue; }
                visited.insert(current.node.clone());
                debug_assert_eq!(Some(&current.est_cost), est_costs.get(&current.node));
                debug_assert_eq!(Some(&current.real_cost), costs.get(&current.node));
                for edge in self.neighbors(&current.node) {
                    let next = edge.dest();
                    let next_cost = current.real_cost + edge.weight();

                    let prior_next_cost = costs.get(next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        let next_state = EstState { est_cost: next_cost + heuristic(next), real_cost: next_cost, node: next.clone() };
                        costs.insert(next.clone(), next_state.real_cost);
                        est_costs.insert(next.clone(), next_state.est_cost);
                        frontier.push(next_state);
                        routes.insert(next.clone(), edge);
                    }
                }
            }

            let mut current = goal?;
            let mut path = Vec::new();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(next.clone());
                    current = next.source().clone();
                } else {
                    unreachable!();
                }
            }
            path.reverse();
            Some(path)
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct Edge<N: Clone + Debug> {
        weight: i32,
        source: N,
        dest: N,
    }

    impl<N: Clone + Debug> Edge<N> {
        pub fn new(weight: i32, source: N, dest: N) -> Edge<N> {
            Edge { weight, source, dest }
        }

        pub fn weight(&self) -> i32 { self.weight }
        pub fn source(&self) -> &N { &self.source }
        pub fn dest(&self) -> &N { &self.dest }
    }

    #[derive(Copy, Clone, Debug)]
    struct State<N: Clone + Debug> {
        cost: i32,
        node: N,
    }

    // We don't implement Eq because it's not well defined, but Ord requires it exist
    impl<N: Clone + Debug> PartialEq for State<N> {
        fn eq(&self, _: &Self) -> bool {
            unimplemented!()
        }
    }

    impl<N: Clone + Debug> Eq for State<N> {}

    impl<N: Clone + Debug> Ord for State<N> {
        fn cmp(&self, other: &State<N>) -> Ordering {
            other.cost.cmp(&self.cost)
        }
    }

    impl<N: Clone + Debug> PartialOrd for State<N> {
        fn partial_cmp(&self, other: &State<N>) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    #[derive(Copy, Clone, Debug)]
    struct EstState<N: Clone + Debug> {
        est_cost: i32,
        real_cost: i32,
        node: N,
    }

    // We don't implement Eq because it's not well defined, but Ord requires it exist
    impl<N: Clone + Debug> PartialEq for EstState<N> {
        fn eq(&self, _: &Self) -> bool {
            unimplemented!()
        }
    }

    impl<N: Clone + Debug> Eq for EstState<N> {}

    impl<N: Clone + Debug> Ord for EstState<N> {
        fn cmp(&self, other: &EstState<N>) -> Ordering {
            other.est_cost.cmp(&self.est_cost)
        }
    }

    impl<N: Clone + Debug> PartialOrd for EstState<N> {
        fn partial_cmp(&self, other: &EstState<N>) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
}
pub use self::internal::{Edge,Graph};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid::{point,Point,vector};
    use std::collections::{BTreeMap};
    use ahash::AHashSet;

    struct BasicGraph {
        blocked: AHashSet<Point>,
    }

    impl BasicGraph {
        fn new(blocked: &[Point]) -> BasicGraph {
            BasicGraph { blocked: blocked.iter().cloned().collect() }
        }
    }

    impl Graph for BasicGraph {
        type Node = Point;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
            if self.blocked.contains(source) { return vec!(); }

            vec!(vector(0, 1), vector(1, 0), vector(0, -1), vector(-1, 0)).iter()
                .map(|v| source + v)
                .filter(|p| !self.blocked.contains(p))
                .map(|d| Edge::new(1, *source, d))
                .collect()
        }
    }

    #[test]
    fn direct() {
        let graph = BasicGraph::new(&[]);
        let start = point(1, 1);
        let goal = point(3, 4);

        let bfs_route = graph.bfs(&start, |n| n == &goal).unwrap();
        assert_eq!(bfs_route.len(), 6);
        assert_eq!(bfs_route[0], start);
        assert_eq!(bfs_route[bfs_route.len()-1], goal);

        let djk_route = graph.dijkstras(&start, |n| n == &goal).unwrap();
        assert_eq!(djk_route.len(), 5);
        assert_eq!(djk_route[0].source(), &start);
        assert_eq!(djk_route[djk_route.len()-1].dest(), &goal);

        let as_route = graph.a_star(&start, |n| n == &goal, |n| (goal - *n).grid_len() as i32).unwrap();
        assert_eq!(as_route.len(), 5);
        assert_eq!(as_route[0].source(), &start);
        assert_eq!(as_route[djk_route.len()-1].dest(), &goal);
    }

    #[test]
    fn wall() {
        let graph = BasicGraph::new(&[
            point(0, 3), point(1, 3), point(2, 3), point(3, 3), point(4, 3)
        ]);
        let start = point(1, 1);
        let goal = point(3, 4);

        let bfs_route = graph.bfs(&start, |n| n == &goal).unwrap();
        assert_eq!(bfs_route.len(), 10);
        assert_eq!(bfs_route[0], start);
        assert_eq!(bfs_route[bfs_route.len()-1], goal);

        let djk_route = graph.dijkstras(&start, |n| n == &goal).unwrap();
        assert_eq!(djk_route.len(), 9);
        assert_eq!(djk_route[0].source(), &start);
        assert_eq!(djk_route[djk_route.len()-1].dest(), &goal);

        let as_route = graph.a_star(&start, |n| n == &goal, |n| (goal - *n).grid_len() as i32).unwrap();
        assert_eq!(as_route.len(), 9);
        assert_eq!(as_route[0].source(), &start);
        assert_eq!(as_route[djk_route.len()-1].dest(), &goal);
    }

    #[test]
    fn all_paths() {
        // From 2019 Day 15 pt 2 - forms a small room
        let graph = BasicGraph::new(&[
            point(1,0), point(2, 0),
            point(0, 1), point(3, 1), point(4, 1),
            point(0, 2), point(2, 2), point(5, 2),
            point(0, 3), point(4, 3),
            point(1, 4), point(2, 4), point(3, 4)
        ]);
        let start = point(2, 3);
        let farthest = point(2, 1);

        let bfs_routes = graph.bfs_all(&start);
        let djk_routes = graph.dijkstras_all(&start);

        let bfs_routes_lens: BTreeMap<_,_> = bfs_routes.iter().map(|(&k, v)| (k, v.len() as i32 - 1)).collect();
        let djk_routes_lens: BTreeMap<_,_> = djk_routes.iter()
            .map(|(&k, v)| (k, v.iter().map(|e| e.weight()).sum())).collect();
        let expected_routes: BTreeMap<_,_> = vec!(
            (point(1, 1), 3), (point(2, 1), 4), (point(1, 2), 2), (point(3, 2), 2),
            (point(4, 2), 3), (point(1, 3), 1), (point(2, 3), 0), (point(3, 3), 1)
        ).iter().cloned().collect();
        assert_eq!(bfs_routes_lens, expected_routes);
        assert_eq!(djk_routes_lens, expected_routes);

        let bfs_route = graph.bfs(&start, |n| n == &farthest).unwrap();
        let bfs_all_route = bfs_routes.get(&farthest).unwrap();
        assert_eq!(bfs_route.len(), bfs_all_route.len());
        // This is not strictly true, but there's only one route to this point for this graph,
        // so it should be reliable for this test case
        assert_eq!(&bfs_route, bfs_all_route);
    }
}