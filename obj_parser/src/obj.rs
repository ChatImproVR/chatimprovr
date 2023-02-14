use cimvr_common::render::{Mesh, Vertex};
use cimvr_engine_interface::{dbg, prelude::*};
use std::{io::Read, str::FromStr};

/// Read OBJ lines into the mesh
/// OBJ line specs: https://all3dp.com/1/obj-file-format-3d-printing-cad/
pub fn obj_lines_to_mesh(obj: &str) -> Mesh {
    let mut m = Mesh::new();

    for line in obj.lines() {
        // Split the line by whitespace
        let mut line = line.split_whitespace();

        // Break the first bit off
        let (first, mut rest) = (line.next(), line);

        // Which kind of line is it?
        match first {
            Some("v") => { // Vertex
                // Treat the line as two arrays of 3 elements (x, y, z) coords and perhaps (u, v, w)
                let mut parts = [[0.; 3], [1.; 3]];

                for part in &mut parts {
                    // Get strings from the rest of the line
                    // The by_ref() here allows us to keep eating the line on the next loop
                    for dim in part {
                        let Some(text) = rest.next() else { break };
                        *dim = text.parse().expect("Invalid float");
                    }
                }

                // Split the parts back up
                let [pos, uvw] = parts;

                // Assemble the vertex
                m.vertices.push(Vertex { pos, uvw });
            },
            Some("l") => { // Line
                // Do the same for indices
                let mut indices = [0; 2]; 
                for dim in &mut indices {
                    let Some(text) = rest.next() else { break };
                    *dim = text.parse().expect("Invalid index");

                    // OBJ files are one-indexed
                    *dim -= 1;
                }
                m.indices.extend(indices);
            },
            Some("f") => { // Faces
                // At this point all vertices have been declared
                // Treat the line as a list of indices
                let mut indices = [0; 5]; // Array of 5 elements?
                                                    // Need to make this more dynamic(?) to hold faces with different nums of vertices
                // Goes through each of the index values on the line
                for dim in &mut indices {
                    let Some(text) = rest.next() else { break };
                    *dim = text.parse().expect("Invalid index");

                    // OBJ files are one-indexed
                    *dim -= 1;
                }

                // When we read the entire line, we need to divide the indexes
                // into triangles
                // i.e. if we have a face with 5 verts:
                // read in [0,1,2] as a triangle, [2,3,0] as another triangle, [3,4,0] as the next triangle

                // Add those indices to be rendered

                m.indices.extend(indices);
            },
            // Some("vn") => { // Vertex normals

            // },
            // Ignore the rest
            _ => (),
        }
    }

    m
}
