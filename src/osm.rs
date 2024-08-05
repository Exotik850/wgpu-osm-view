use crate::vertex::Vertex;
use glam::Vec2;
use osmpbf::Element;
use std::{io::BufReader, path::Path};

use anyhow::Result;
struct OSM {
    nodes: Vec<Vec2>,
    min: Vec2,
    max: Vec2,
    ways: Vec<Vec<usize>>,
}

impl OSM {
    fn load(path: impl AsRef<Path>) -> Result<OSM> {
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
                temp_map.insert(node.id(), nodes.len());
                let node = Vec2::new(node.lon() as f32, node.lat() as f32);
                min.x = min.x.min(node.x);
                min.y = min.y.min(node.y);
                max.x = max.x.max(node.x);
                max.y = max.y.max(node.y);
                nodes.push(node);
            }
            Element::DenseNode(node) => {
                temp_map.insert(node.id(), nodes.len());
                let node = Vec2::new(node.lon() as f32, node.lat() as f32);
                min.x = min.x.min(node.x);
                min.y = min.y.min(node.y);
                max.x = max.x.max(node.x);
                max.y = max.y.max(node.y);
                nodes.push(node);
            }
            Element::Way(way) => {
                let ids = way.raw_refs();
                let mut inds = Vec::with_capacity(ids.len());
                for id in ids {
                    if let Some(&ind) = temp_map.get(id) {
                        inds.push(ind);
                    }
                }
                ways.push(inds);
            }
            _ => {}
        })?;

        Ok(OSM {
            nodes,
            ways,
            min,
            max,
        })
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
    if Path::new("./data.bin").exists() {
        let data = std::fs::read("./data.bin").unwrap();
        if let Ok(data) = bytemuck::try_cast_slice(&data) {
            return data.to_vec();
        }
    }

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
