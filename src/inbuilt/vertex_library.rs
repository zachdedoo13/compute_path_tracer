use crate::inbuilt::vertex_package::Vertex;

pub const SQUARE_VERTICES: &[Vertex] = &[
   Vertex { position: [1.0, 1.0, 0.0] }, // Vertex 0: top-right
   Vertex { position: [-1.0, -1.0, 0.0] }, // Vertex 1: bottom-left
   Vertex { position: [1.0, -1.0, 0.0] }, // Vertex 2: bottom-right
   Vertex { position: [-1.0, 1.0, 0.0] }, // Vertex 3: top-left
];
pub const SQUARE_INDICES: &[u16] = &[
   0, 1, 2,
   3, 1, 0,
];

pub const CUBE_VERTICES: &[Vertex] = &[
   Vertex { position: [-1.0, -1.0, -1.0] }, // Vertex 0: bottom-left-back
   Vertex { position: [1.0, -1.0, -1.0] },  // Vertex 1: bottom-right-back
   Vertex { position: [1.0, 1.0, -1.0] },   // Vertex 2: top-right-back
   Vertex { position: [-1.0, 1.0, -1.0] },  // Vertex 3: top-left-back
   Vertex { position: [-1.0, -1.0, 1.0] },  // Vertex 4: bottom-left-front
   Vertex { position: [1.0, -1.0, 1.0] },   // Vertex 5: bottom-right-front
   Vertex { position: [1.0, 1.0, 1.0] },    // Vertex 6: top-right-front
   Vertex { position: [-1.0, 1.0, 1.0] },   // Vertex 7: top-left-front
];

pub const CUBE_INDICES: &[u16] = &[
   // Back face
   0, 2, 1,
   0, 3, 2,
   // Front face
   4, 5, 6,
   4, 6, 7,
   // Left face
   4, 7, 3,
   4, 3, 0,
   // Right face
   1, 2, 6,
   1, 6, 5,
   // Bottom face
   4, 0, 1,
   4, 1, 5,
   // Top face
   3, 7, 6,
   3, 6, 2,
];