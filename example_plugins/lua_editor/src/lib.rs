use cimvr_common::{
    render::{
        Mesh, MeshHandle, Primitive, Render, RenderExtra, ShaderHandle, ShaderSource, UploadMesh,
        Vertex,
    },
    ui::{
        egui::{
            color_picker::{color_edit_button_rgba, color_edit_button_srgba, Alpha},
            Color32, DragValue, Grid, Rgba, ScrollArea, TextEdit, Ui,
        },
        GuiInputMessage, GuiTab,
    },
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct ClientState {
    shader_sources: ShaderSource,
    fragment_tab: GuiTab,
    vertex_tab: GuiTab,
    config_tab: GuiTab,
}

make_app_state!(ClientState, DummyUserState);

/// This handle uniquely identifies the mesh data between all clients, and the server.
/// When the server copies the ECS data to the clients, they immediately know which mesh to render!
///
/// Note how we've used pkg_namespace!() to ensure that the name is closer to universally unique
const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

const DEFAULT_VERTEX: &str = include_str!("shaders/default.vert");
const DEFAULT_FRAGMENT: &str = include_str!("shaders/default.frag");

const SHADER_ID: ShaderHandle = ShaderHandle::new(pkg_namespace!("ShaderEditor"));

#[derive(Component, Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FullScreenTri;

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Make the cube mesh available to the rendering engine
        // This defines the CUBE_HANDLE id to refer to the mesh we get from cube()
        io.send(&UploadMesh {
            mesh: dummy_mesh(),
            id: CUBE_HANDLE,
        });

        // Crate entity
        io.create_entity()
            // Attach a Transform component (which defaults to the origin)
            .add_component(Transform::default())
            // Attach the Render component, which details how the object should be drawn
            // Note that we use CUBE_HANDLE here, to tell the rendering engine to draw the cube
            .add_component(
                Render::new(CUBE_HANDLE)
                    .primitive(Primitive::Triangles)
                    .shader(SHADER_ID),
            )
            // Add shader metadata
            .add_component(RenderExtra([
                1., 0., 0., 1., // .
                1., 1., 0., 1., // .
                0., 0., 1., 1., // .
                -0.413, -0.216, 1., 1., // .
            ]))
            // Flag
            .add_component(FullScreenTri)
            .build();

        // Declare shaders
        let shader_sources = ShaderSource {
            vertex_src: DEFAULT_VERTEX.into(),
            fragment_src: DEFAULT_FRAGMENT.into(),
            id: SHADER_ID,
        };

        io.send(&shader_sources);

        sched
            .add_system(Self::update_ui)
            .query(
                "ShaderPlane",
                Query::new()
                    .intersect::<RenderExtra>(Access::Write)
                    .intersect::<FullScreenTri>(Access::Read),
            )
            .subscribe::<GuiInputMessage>()
            .build();

        Self {
            shader_sources,
            fragment_tab: GuiTab::new(io, pkg_namespace!("Fragment")),
            vertex_tab: GuiTab::new(io, pkg_namespace!("Vertex")),
            config_tab: GuiTab::new(io, pkg_namespace!("Config")),
        }
    }
}

impl ClientState {
    fn update_ui(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Shader code editors
        let mut code_changed = false;

        self.fragment_tab.show(io, |ui| {
            code_changed |= code_edit(ui, &mut self.shader_sources.fragment_src);
        });

        self.vertex_tab.show(io, |ui| {
            code_changed |= code_edit(ui, &mut self.shader_sources.vertex_src);
        });

        let presets = [
            ("Defaults", DEFAULT_VERTEX, DEFAULT_FRAGMENT),
            (
                "Fractal",
                DEFAULT_VERTEX,
                include_str!("shaders/mandelbrot.frag"),
            ),
            (
                "Raycast",
                include_str!("shaders/raycast.vert"),
                include_str!("shaders/raycast.frag"),
            ),
        ];

        // Config editor
        let Some(entity) = query.iter("ShaderPlane").next() else { return };
        query.modify(entity, |RenderExtra(array)| {
            self.config_tab.show(io, |ui| {
                ui.label("RenderExtra:");
                Grid::new("RenderExtra").show(ui, |ui| {
                    for row in array.chunks_exact_mut(4) {
                        // Edit grid
                        for field in &mut *row {
                            ui.add(DragValue::new(field).speed(3e-2));
                        }

                        // Color editor
                        if row.iter().all(|x| (0.0..=1.0).contains(x)) {
                            let mut rgba =
                                Rgba::from_rgba_unmultiplied(row[0], row[1], row[2], row[3]);
                            color_edit_button_rgba(ui, &mut rgba, Alpha::Opaque);
                            row.copy_from_slice(&rgba.to_array());
                        }

                        ui.end_row();
                    }
                });

                // Presets
                ui.label("Presets:");
                ui.horizontal(|ui| {
                    for (name, vert_src, frag_src) in presets {
                        if ui.button(name).clicked() {
                            self.shader_sources = ShaderSource {
                                vertex_src: vert_src.into(),
                                fragment_src: frag_src.into(),
                                id: SHADER_ID,
                            };
                            code_changed = true;
                        }
                    }
                });
            });
        });

        if code_changed {
            io.send(&self.shader_sources);
        }
    }
}

fn code_edit(ui: &mut Ui, code: &mut String) -> bool {
    ScrollArea::vertical()
        .show(ui, |ui| {
            ui.add_sized(ui.available_size(), TextEdit::multiline(code).code_editor())
                .changed()
        })
        .inner
}

/// Defines the mesh data fro a cube
fn dummy_mesh() -> Mesh {
    // List of vertex positions and colors
    let vertices = vec![Vertex::new([0.; 3], [0.; 3]); 3];

    // Each 3 indices (indexing into vertices) define a triangle
    let indices = vec![0, 1, 2];

    Mesh { vertices, indices }
}
