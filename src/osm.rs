use crate::vertex::Vertex;
use glam::{DVec2, Vec2};
use osmpbf::Element;
use radix_trie::Trie;
use std::{collections::HashMap, io::BufReader, path::Path};

use anyhow::Result;

struct Node {
    // id: i64,
    connected_index: usize,
    pos: DVec2,
}

pub struct OSMGraph {
    nodes: Vec<Node>,
    connected: Vec<Vec<usize>>,
    buckets: HashMap<(u32, u32), Vec<usize>>,
}

impl OSMGraph {
    pub fn from_osm(osm: &OSM) -> Self {
        let mut nodes = Vec::new();
        let mut connected = Vec::new();
        let mut buckets = HashMap::new();
        for &TempNode { pos, .. } in osm.nodes.iter() {
            let connected_index = connected.len();
            // let id = node.id;
            // node_map.insert(i, id);
            let bucket = ((pos.x as u32 / 100) * 100, (pos.y as u32 / 100) * 100);
            buckets
                .entry(bucket)
                .or_insert_with(Vec::new)
                .push(nodes.len());
            nodes.push(Node {
                connected_index,
                pos,
            });
            connected.push(Vec::new());
        }
        for way in &osm.ways {
            for i in 0..way.len() - 1 {
                let a = way[i];
                let b = way[i + 1];
                connected[a].push(b);
                connected[b].push(a);
            }
        }
        Self {
            nodes,
            connected,
            buckets,
        }
    }

    fn closest_node(&self, pos: DVec2) -> Option<usize> {
        let bucket = ((pos.x as u32 / 100) * 100, (pos.y as u32 / 100) * 100);
        let mut min_distance = f64::INFINITY;
        let mut closest = None;
        for &node in self.buckets.get(&bucket).into_iter().flatten() {
            let distance = self.nodes[node].pos.distance(pos);
            if distance < min_distance {
                min_distance = distance;
                closest = Some(node);
            }
        }
        closest
    }

    // Breadth-first search
    fn plan_path(&self, a: usize, b: usize) -> Option<Vec<usize>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut queue = std::collections::VecDeque::new();
        let mut prev = vec![std::usize::MAX; self.nodes.len()];
        queue.push_back(a);
        visited[a] = true;
        while let Some(node) = queue.pop_front() {
            if node == b {
                let mut path = Vec::new();
                let mut node = b;
                while node != a {
                    path.push(node);
                    node = prev[node];
                }
                path.push(a);
                path.reverse();
                return Some(path);
            }
            for &next in &self.connected[node] {
                if !visited[next] {
                    visited[next] = true;
                    prev[next] = node;
                    queue.push_back(next);
                }
            }
        }
        None
    }

    // A* pathfinding
    pub fn plan_path_a_star(&self, a: usize, b: usize) -> Option<Vec<usize>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut queue = std::collections::BinaryHeap::new();
        let mut prev = vec![std::usize::MAX; self.nodes.len()];
        let mut cost = vec![f64::INFINITY; self.nodes.len()];
        let mut heuristic = vec![u64::MAX; self.nodes.len()];
        queue.push(std::cmp::Reverse((0, a)));
        cost[a] = 0.0;
        heuristic[a] = self.nodes[a].pos.distance(self.nodes[b].pos) as u64;
        while let Some(std::cmp::Reverse((_, node))) = queue.pop() {
            if node == b {
                let mut path = Vec::new();
                let mut node = b;
                while node != a {
                    path.push(node);
                    node = prev[node];
                }
                path.push(a);
                path.reverse();
                return Some(path);
            }
            if visited[node] {
                continue;
            }
            visited[node] = true;
            for &next in &self.connected[node] {
                let next_cost = cost[node] + self.nodes[node].pos.distance(self.nodes[next].pos);
                if next_cost < cost[next] {
                    cost[next] = next_cost;
                    heuristic[next] =
                        (next_cost + self.nodes[next].pos.distance(self.nodes[b].pos)) as u64;
                    prev[next] = node;
                    queue.push(std::cmp::Reverse((heuristic[next], next)));
                }
            }
        }
        None
    }
}

struct TempNode {
    // id: i64,
    pos: DVec2,
    tags: HashMap<String, String>,
}

pub struct OSM {
    nodes: Vec<TempNode>,
    min: DVec2,
    max: DVec2,
    ways: Vec<Vec<usize>>,
}

impl OSM {
    pub fn load(path: impl AsRef<Path>) -> Result<OSM> {
        let reader = std::fs::File::open(path)?;
        let reader = BufReader::new(reader);
        let file = osmpbf::ElementReader::new(reader);
        let mut temp_map = std::collections::HashMap::new();
        let mut nodes = Vec::new();
        let mut ways = Vec::new();
        let mut min = DVec2::new(f64::MAX, f64::MAX);
        let mut max = DVec2::new(f64::MIN, f64::MIN);
        let mut small = 0;
        file.for_each(|ele| match ele {
            Element::Node(node) => {
                temp_map.insert(node.id(), nodes.len());
                let pos = DVec2::new(node.lon(), node.lat());
                // nodes.push(node);
                min = min.min(pos);
                max = max.max(pos);
                nodes.push(TempNode {
                    pos,
                    tags: node
                        .tags()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect(),
                });
            }
            Element::DenseNode(node) => {
                temp_map.insert(node.id(), nodes.len());
                let pos = DVec2::new(node.lon(), node.lat());
                min = min.min(pos);
                max = max.max(pos);
                nodes.push(TempNode {
                    pos,
                    tags: node
                        .tags()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect(),
                });
                // nodes.push(node);
            }
            Element::Way(way) => {
                let ids = way.refs();
                if ids.len() <= 2 {
                    small += 1;
                    // eprintln!("Way with less than 2 nodes");
                    return;
                }
                ways.push(ids.collect());
            }
            _ => {}
        })?;
        eprintln!("Small ways: {}", small);

        // Go through the ways and replace the ids with the indices
        let ways = ways
            .into_iter()
            .map(|way: Vec<_>| {
                way.into_iter()
                    .map(|id| temp_map.get(&id).copied().unwrap())
                    .collect()
            })
            .collect();

        Ok(OSM {
            nodes,
            ways,
            min,
            max,
        })
    }

    pub fn indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        for (i, way) in self.ways.iter().enumerate() {
            indices.extend(way.iter().map(|&x| x as u32));
            if i != self.ways.len() - 1 {
                indices.push(std::u32::MAX);
            }
        }
        indices
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let size = self.size();
        let center = self.center();
        for node in &self.nodes {
            vertices.push(Vertex {
                pos: node.pos.as_vec2() - center / size,
                // pos: Vec2::new((node.x - center.x) / size.x, (node.y - center.y) / size.y),
            });
        }
        vertices
    }

    fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    fn width(&self) -> f32 {
        (self.max.x - self.min.x) as f32
    }

    fn height(&self) -> f32 {
        (self.max.y - self.min.y) as f32
    }

    fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) as f32 / 2.0,
            (self.min.y + self.max.y) as f32 / 2.0,
        )
    }

    pub fn trie(&mut self) -> Trie<String, usize> {
      let mut trie = Trie::new();
      for (i, node) in self.nodes.iter_mut().enumerate() {
        if let Some(name) = node.tags.remove("name") {
          trie.insert(name, i);
        }
      }
      trie
  }
}
