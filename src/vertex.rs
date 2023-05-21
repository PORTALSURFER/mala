use crate::constants::VertexFloat;

pub struct Vertex {
    pub position: [VertexFloat; 2],
}

impl Vertex {
    pub fn new(x: VertexFloat, y: VertexFloat) -> Self {
        Self {
            position: [x, y],
        }
    }
}