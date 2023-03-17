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

                // A face must have at least 3 vertices and at most 30.
                let mut faces = [0; 3];
                let max_indices = 30;

                // We don't know how many indices will be on a line, so we will just initialize it without a size for now
                let mut parsed_line = vec![];

                // Allocate a collection so we can better manage the indices
                // How do we parse through the line AND add it to a mutable array? Use .push()?
                // Potentially infitie loops are terrifying. Re evaluate this later
                loop { 
                    let Some(text) = rest.next() else { break };            // Refutable pattern match - if nothing left, break
                                                                            // Index from string to int, check if index exists
                                                                            // Add the index to the vector
                    let idx: u32 = text.parse().expect("Invalid index");
                    parsed_line.push(idx - 1);                              // OBJ files are one-indexed                                                
                    
                    // We don't want a face with more than ten triangles. Break the loop.
                    // Probably need to produce an error here
                    if parsed_line.len() > max_indices {
                        break
                    }; 
                }

                // When we read the entire line, we need to divide the indexes into triangles
                // i.e. if we have a face with 5 verts:
                // read in [0,1,2] as a triangle, [0,2,3] as another triangle, [0,3,4] as the next triangle
                // Delimit first by whitespace -- then need to check for slashes to delimit texture/vertex normals later
                

                // Will loop through the parsed line and divide them into triangles
                let mut i = 0;
                while i+1 < parsed_line.len() {
                    // For each iteration, while i is less than the size of the parsed line:
                    // 3 elements will be pushed at a time
                    // First element will always be the first index in the parsed line
                    faces[0] = parsed_line[0];
                    // Second element will always be the ith index
                    faces[1] = parsed_line[i];
                    // Third element will always be the (i+1)th index
                        //If there is no third element, return error
                    faces[2] = parsed_line[i+1];

                    // Add those indices to be rendered
                    m.indices.extend(faces);

                    // Increment index
                    i += 1;
                }
            },

            // Some("vn") => { // Vertex normals

            // },

            // Ignore the rest
            _ => (),
        }
    }

    m
}
