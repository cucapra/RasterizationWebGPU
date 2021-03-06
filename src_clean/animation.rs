use gltf::{animation::Channel, animation::util::ReadOutputs, buffer::Data};
use std::collections::BTreeMap;
use glam::{Quat, Vec3};
use ordered_float::OrderedFloat;

#[derive(Debug)]
pub struct Animation {
    pub target: usize,
    duration: f32,
    map: BTreeMap<OrderedFloat<f32>, Transformation>,
}

impl Animation {
    pub fn new(channel: Channel, buffers: &Vec<Data>, duration: f32) -> Self {
        let target = channel.target().node().index();
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

        let inputs = reader.read_inputs().unwrap().map(|f| OrderedFloat(f));
        let map = match reader.read_outputs().unwrap() {
            ReadOutputs::Translations(ts) => {
                let outputs = ts.map(|t| Transformation::Translate(t.into()));
                inputs.zip(outputs).collect::<BTreeMap<OrderedFloat<f32>, Transformation>>()
            }
            ReadOutputs::Rotations(rs) => {
                let outputs = rs.into_f32().map(|r| Transformation::Rotate(Quat::from_slice(&r)));
                inputs.zip(outputs).collect::<BTreeMap<OrderedFloat<f32>, Transformation>>()
            }
            ReadOutputs::Scales(ss) => {
                let outputs = ss.map(|s| Transformation::Scale(s.into()));
                inputs.zip(outputs).collect::<BTreeMap<OrderedFloat<f32>, Transformation>>()
            },
            _ => BTreeMap::new(),
        };
        Self {
            target,
            map,
            duration,
        }
    }

    pub fn get(&self, time: f32) -> Option<Transformation> {
        let local_time = time % self.duration;
        let (OrderedFloat(prev_time), prev) = self.map.range(..OrderedFloat(local_time)).next_back().unwrap_or(self.map.first_key_value()?);
        let (OrderedFloat(next_time), next) = self.map.range(OrderedFloat(local_time)..).next()?;
        let interp_time = (local_time - prev_time) / (next_time - prev_time);
        Some(if prev == next {
             *prev
        } else {
            match (prev, next) {
                (Transformation::Translate(v0), Transformation::Translate(v1)) => Transformation::Translate((*v0).lerp(*v1, interp_time)),
                (Transformation::Rotate(q0), Transformation::Rotate(mut q1)) => {
                    if q0.dot(q1) < 0.0 {
                        q1 = -(q1);
                    }
                    Transformation::Rotate((*q0).slerp(q1, interp_time).normalize())
                }
                (Transformation::Scale(v0), Transformation::Scale(v1)) => Transformation::Scale((*v0).lerp(*v1, interp_time)),
                _ => unreachable!("By all laws of physics, impossible!"),
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Transformation {
    Scale(Vec3),
    Translate(Vec3),
    Rotate(Quat),
}
