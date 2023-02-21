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
                // Treat the line as a list of indices to be divided into triangles
                // Drawing faces as a triangle fan
                
                // While there are still indices to be read on the line:

                
                let mut indices = [0; 3]; // Array of 3 vertices, initialize with zeros

                // Allocate indexes for the next triangle
                for dim in &mut indices {

                    // Take the next index in
                    let Some(text) = rest.next() else { break }; // Refutable pattern match
                    // Index from string to int, check if index exists
                    *dim = text.parse().expect("Invalid index");

                    // OBJ files are one-indexed - what do we mean by this?
                    *dim -= 1;
                }

                // When we read the entire line, we need to divide the indexes
                // into triangles
                // i.e. if we have a face with 5 verts:
                // read in [0,1,2] as a triangle, [0,2,3] as another triangle, [0,3,4] as the next triangle
                // Delimit first by whitespace -- then need to check for slashes to delimit texture/vertex normals

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
