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
        file.for_each(|ele| match ele {
            Element::Node(node) => {
                temp_map.insert(node.id(), nodes.len() + 1);
                let node = Vec2::new(node.lon() as f32, node.lat() as f32);
                min.x = min.x.min(node.x);
                min.y = min.y.min(node.y);
                max.x = max.x.max(node.x);
                max.y = max.y.max(node.y);
                nodes.push(node);
            }
            Element::DenseNode(node) => {
                temp_map.insert(node.id(), nodes.len() + 1);
                let node = Vec2::new(node.lon() as f32, node.lat() as f32);
                min.x = min.x.min(node.x);
                min.y = min.y.min(node.y);
                max.x = max.x.max(node.x);
                max.y = max.y.max(node.y);
                nodes.push(node);
            }
            Element::Way(way) => {
                let ids = way.raw_refs();
                ways.push(ids.to_vec());
            }
            _ => {}
        })?;

        // Go through the ways and replace the ids with the indices
        let ways = ways.into_iter().map(|way| {
            way.into_iter()
                .filter_map(|id| temp_map.get(&id).copied())
                .map(|x| x as u32)
                .collect::<Vec<u32>>()
        }).collect();

        Ok(OSM {
            nodes,
            ways,
            min,
            max,
        })
    }

    // Seperates all of the ways with the maxval
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

pub fn load_points() -> Vec<Vertex> {
    let osm = OSM::load("./tennessee-latest.osm.pbf").unwrap();
    let mut points = Vec::new();
    let center = osm.center();
    let size = osm.size();
    for node in osm.nodes {
        let pos = Vec2::new((node.x - center.x) / size.x, (node.y - center.y) / size.y);
        points.push(Vertex {
            // _padding:
            pos,
            // color: Vector4::new(pos.x, pos.y, 1.0, 1.0).into(),
        });
    }
    // for way in osm.ways {
    //     for node in way {
    //         let Some(mut pos) = osm.nodes.get(node).copied() else {
    //             continue;
    //         };
    //         pos.x = (pos.x - center.x) / size.x;
    //         pos.y = (pos.y - center.y) / size.y;
    //         points.push(Vertex {
    //             pos,
    //             color: Vector4::new(1.0, 1.0, 1.0, 1.0).into(),
    //         });
    //     }
    // }
    points
}
