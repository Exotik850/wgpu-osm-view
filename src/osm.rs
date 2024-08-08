use crate::vertex::Vertex;
use glam::Vec2;
use osmpbf::Element;
use std::{io::BufReader, path::Path};

use anyhow::Result;

pub struct OSM {
    nodes: Vec<Vec2>,
    min: Vec2,
    max: Vec2,
    ways: Vec<Vec<u32>>,
}

impl OSM {
    pub fn load(path: impl AsRef<Path>) -> Result<OSM> {
        let reader = std::fs::File::open(path)?;
        let reader = BufReader::new(reader);
        let file = osmpbf::ElementReader::new(reader);
        let mut temp_map = std::collections::HashMap::new();
        let mut nodes = Vec::new();
        let mut ways = Vec::new();
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        let mut small = 0;
        file.for_each(|ele| match ele {
            Element::Node(node) => {
                temp_map.insert(node.id(), nodes.len());
                let node = Vec2::new(node.lon() as f32, node.lat() as f32);
                nodes.push(node);
                min = min.min(node);
                max = max.max(node);
            }
            Element::DenseNode(node) => {
                temp_map.insert(node.id(), nodes.len());
                let node = Vec2::new(node.lon() as f32, node.lat() as f32);
                min = min.min(node);
                max = max.max(node);
                nodes.push(node);
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
                    .map(|id| temp_map.get(&id).copied().unwrap() as u32)
                    .collect::<Vec<u32>>()
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
            indices.extend(way);
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
                pos: Vec2::new((node.x - center.x) / size.x, (node.y - center.y) / size.y),
            });
        }
        vertices
    }

    fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }
}