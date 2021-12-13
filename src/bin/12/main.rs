use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::str::FromStr;
use anyhow::{ensure, Error, Result};

fn main() -> Result<()> {
    let input: Caves = include_str!("input.txt").parse()?;

    println!("Paths:              {}", input.all_paths().len());
    println!("Paths (Revisiting): {}", input.all_paths_allow_revisit().len());

    Ok(())
}

trait VisitLog: Clone {
    fn visit(&mut self, node: &Rc<String>);
    fn visited(&self, node: &Rc<String>) -> bool;
}

#[derive(Clone)]
struct SmallCavesVisitLog {
    visit_counts: HashMap<Rc<String>, usize>,
    allow_one_second_visit: bool,
}

impl SmallCavesVisitLog {
    fn new(allow_one_second_visit: bool) -> SmallCavesVisitLog {
        SmallCavesVisitLog { visit_counts: HashMap::new(), allow_one_second_visit }
    }
}

impl VisitLog for SmallCavesVisitLog {
    fn visit(&mut self, node: &Rc<String>) {
        self.visit_counts.entry(node.clone()).and_modify(|v| *v += 1).or_insert(1);
    }

    fn visited(&self, node: &Rc<String>) -> bool {
        let small_cave = node.chars().all(|c| char::is_ascii_lowercase(&c));
        if !small_cave { return false; } // allow visiting large caves at will
        if self.visit_counts.contains_key(node) {
            if self.allow_one_second_visit {
                if node.as_str() == "start" { return true; } // don't re-visit start
                // Allow visiting *one* small cave twice
                return self.visit_counts.iter()
                    .filter(|(k,_)|
                        k.chars().all(|c| char::is_ascii_lowercase(&c)))
                    .any(|(_,&v)| v > 1);
            }
            return true;
        }
        false
    }
}

struct Caves {
    connections: HashMap<Rc<String>, Vec<Rc<String>>>,
}

impl Caves {
    fn new(edges: &[(&str, &str)]) -> Caves {
        let mut refs = HashMap::new();
        let mut connections = HashMap::new();
        for (source, dest) in edges {
            let source = refs.entry(source).or_insert_with(|| Rc::new(source.to_string())).clone();
            let dest = refs.entry(dest).or_insert_with(|| Rc::new(dest.to_string())).clone();
            connections.entry(source.clone()).and_modify(|v: &mut Vec<_>| v.push(dest.clone())).or_insert(vec!(dest.clone()));
            connections.entry(dest.clone()).and_modify(|v: &mut Vec<_>| v.push(source.clone())).or_insert(vec!(source.clone()));
        }
        Caves { connections }
    }

    // TODO this is ~equivalent to pathfinding::Graph::neighbors, but none of the existing
    //   operations on Graph are helpful for this problem. Can the all_paths() operation be
    //   generalized and promoted to pathfinding?
    fn connected_to(&self, source: &Rc<String>) -> &[Rc<String>] {
        &self.connections[source]
    }

    fn all_paths(&self) -> Vec<VecDeque<Rc<String>>> {
        self.subpaths(&Rc::new("start".to_string()), &SmallCavesVisitLog::new(false))
    }

    fn all_paths_allow_revisit(&self) -> Vec<VecDeque<Rc<String>>> {
        self.subpaths(&Rc::new("start".to_string()), &SmallCavesVisitLog::new(true))
    }

    fn subpaths<V: VisitLog>(&self, current: &Rc<String>, visit_log: &V) -> Vec<VecDeque<Rc<String>>> {
        let mut ret = Vec::new();
        let mut visit_log = visit_log.clone();
        visit_log.visit(current);
        for dest in self.connected_to(current) {
            if visit_log.visited(dest) { continue; }
            if dest.as_str() == "end" {
                let mut path = VecDeque::new();
                path.push_back(dest.clone());
                ret.push(path);
            } else {
                for mut subpath in self.subpaths(dest, &visit_log) {
                    subpath.push_front(dest.clone());
                    ret.push(subpath);
                }
            }
        }
        ret
    }
}

impl FromStr for Caves {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let edges = s.lines().map(|l| l.split('-').collect::<Vec<_>>()).map(|p| {
            ensure!(p.len() == 2);
            Ok((p[0], p[1]))
        }).collect::<Result<Vec<_>>>()?;
        Ok(Caves::new(&edges))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    parameterized_test::create!{ paths, (input, num_paths, num_repeats_paths), {
        let input: Caves = input.parse().unwrap();
        assert_eq!(input.all_paths().len(), num_paths);
        assert_eq!(input.all_paths_allow_revisit().len(), num_repeats_paths);
    } }
    paths! {
        example1: (include_str!("example1.txt"), 10, 36),
        example2: (include_str!("example2.txt"), 19, 103),
        example3: (include_str!("example3.txt"), 226, 3509),
    }
}
