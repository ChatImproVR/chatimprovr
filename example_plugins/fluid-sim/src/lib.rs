use cimvr_common::{
    glam::Vec3,
    render::{CameraComponent, Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println};

make_app_state!(ClientState, ServerState);

struct ClientState {
    //fluid_render_buf: UploadMesh,
    //fluid_vel_render_buf: UploadMesh,
    fluid_sim: FluidSim,
    particles: ParticleState,
    frame: usize,
    tracking: VrTracking,
    is_vr: bool,
    //last: [f32; 3],
}

//const VEL_Z: f32 = 0.5;
const FLUID_VEL_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Fluid velocity"));
const CUBE_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

const FLUID_POS: Vec3 = Vec3::new(0., 1.2, 0.);

struct ServerState;

impl UserState for ServerState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Fluid lines mesh
        let fluid_vel_rdr = Render::new(FLUID_VEL_ID).primitive(Primitive::Lines);

        io.create_entity()
            .add_component(Transform::default().with_position(FLUID_POS))
            .add_component(fluid_vel_rdr)
            .add_component(Synchronized)
            .build();

        let cube_rdr = Render::new(CUBE_ID).primitive(Primitive::Lines);

        io.create_entity()
            .add_component(Transform::default().with_position(FLUID_POS))
            .add_component(cube_rdr)
            .add_component(Synchronized)
            .build();

        Self
    }
}

impl UserState for ClientState {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        let s = 15;
        let fluid_sim = FluidSim::new(s, s, s);

        let mut line_mesh = Mesh::default();
        draw_velocity_lines(&mut line_mesh, fluid_sim.uvw(), 0.);

        let fluid_vel_render_buf = UploadMesh {
            id: FLUID_VEL_ID,
            mesh: line_mesh,
        };
        io.send(&fluid_vel_render_buf);
        io.send(&cube(1.));

        // Schedule the systems
        schedule
            .add_system(Self::vr_interaction)
            .subscribe::<VrUpdate>()
            .query(
                "Camera",
                Query::new()
                    .intersect::<Transform>(Access::Read)
                    .intersect::<CameraComponent>(Access::Read),
            )
            .build();

        schedule.add_system(Self::fluid_update).build();

        let particles = ParticleState::new(20_000, io, fluid_sim.uvw().0);

        Self {
            tracking: VrTracking::new(),
            is_vr: false,
            //fluid_vel_render_buf,
            //fluid_render_buf,
            fluid_sim,
            frame: 0,
            particles,
            //last: [0.; 3],
        }
    }
}

fn cube(s: f32) -> UploadMesh {
    let mut mesh = Mesh::new();
    let color = [1.; 3];

    let a = mesh.push_vertex(Vertex::new([-s, -s, -s], color));
    let b = mesh.push_vertex(Vertex::new([-s, -s, s], color));
    let c = mesh.push_vertex(Vertex::new([-s, s, -s], color));
    let d = mesh.push_vertex(Vertex::new([-s, s, s], color));

    let e = mesh.push_vertex(Vertex::new([s, -s, s], color));
    let f = mesh.push_vertex(Vertex::new([s, -s, -s], color));
    let g = mesh.push_vertex(Vertex::new([s, s, s], color));
    let h = mesh.push_vertex(Vertex::new([s, s, -s], color));

    mesh.push_indices(&[
        a, b, c, d, e, f, g, h, a, c, b, d, e, g, f, h, a, f, b, e, c, h, d, g,
    ]);

    UploadMesh { mesh, id: CUBE_ID }
}

impl ClientState {
    fn vr_interaction(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(update) = io.inbox_first() {
            self.tracking.update(&update);
            self.is_vr = true;
        }

        let mut vr_space_transf = Transform::identity();
        for entity in query.iter("Camera") {
            vr_space_transf = query.read::<Transform>(entity);
        }

        for hand in [&self.tracking.left, &self.tracking.right] {
            if let Some(grip_pos) = hand.grip_pos {
                let pos = grip_pos + vr_space_transf.pos - FLUID_POS;
                push_fluid(&mut self.fluid_sim, 0, pos, hand.vel);
            }
        }

        if !self.is_vr {
            push_fluid(
                &mut self.fluid_sim,
                1,
                Vec3::ZERO,
                Vec3::new(1., 2., -1.2) / 1e2,
            );
        }
    }

    fn fluid_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let dt = 1.;
        self.fluid_sim.step(dt, 1.9, 20);
        self.particles.step(self.fluid_sim.uvw(), io, dt);

        io.send(&self.particles.render);

        self.frame += 1;
    }
}

pub struct ParticleState {
    particles: Vec<[f32; 3]>,
    render: UploadMesh,
}

impl ParticleState {
    pub fn new(n: usize, io: &mut EngineIo, u: &Array3D<f32>) -> Self {
        Self {
            particles: (0..n).map(|_| Self::random_vert(u, io)).collect(),
            render: UploadMesh {
                mesh: Mesh::new(),
                id: FLUID_VEL_ID,
            },
        }
    }

    pub fn step(
        &mut self,
        (u, v, w): (&Array3D<f32>, &Array3D<f32>, &Array3D<f32>),
        io: &mut EngineIo,
        dt: f32,
    ) {
        self.render.mesh.clear();

        for part in &mut self.particles {
            let before = *part;
            let [x, y, z] = before;
            let (x, y, z) = advect(u, v, w, x, y, z, -dt);
            let after = [x, y, z];

            *part = after;

            if Self::bounds(u, after) && Self::bounds(u, before) {
                let w = u.width() as f32;
                let downscale = |i| (i / w) * 2. - 1.;

                let beforer = Vec3::from(before) + (Vec3::from(before) - Vec3::from(after)) * 4.;
                let beforer: [f32; 3] = beforer.into();

                let x = self
                    .render
                    .mesh
                    .push_vertex(Vertex::new(beforer.map(downscale), [1.; 3]));
                let y = self
                    .render
                    .mesh
                    .push_vertex(Vertex::new(after.map(downscale), [1.; 3]));
                self.render.mesh.push_indices(&[x, y]);
            } else {
                *part = Self::random_vert(u, io);
            }
        }
    }

    fn random_vert(u: &Array3D<f32>, io: &mut EngineIo) -> [f32; 3] {
        let margin = 1;
        let mut v = || io.random() as u64 as f32 / u64::MAX as f32;
        [
            (u.width() - margin * 2 - 1) as f32 * v() + margin as f32,
            (u.height() - margin * 2 - 1) as f32 * v() + margin as f32,
            (u.length() - margin * 2 - 1) as f32 * v() + margin as f32,
        ]
    }

    fn bounds(u: &Array3D<f32>, [x, y, z]: [f32; 3]) -> bool {
        let check = |x, w| x > 1. && x < (w - 1) as f32;
        check(x, u.width()) && check(y, u.height()) && check(z, u.length())
    }
}

#[derive(Clone)]
pub struct FluidState {
    u: Array3D<f32>,
    v: Array3D<f32>,
    w: Array3D<f32>,
}

pub struct FluidSim {
    read: FluidState,
    write: FluidState,
}

/// Transport x and y (relative to fluid grid coordinates) along `u` and `v` by a step `dt`
fn advect(
    u: &Array3D<f32>,
    v: &Array3D<f32>,
    w: &Array3D<f32>,
    x: f32,
    y: f32,
    z: f32,
    dt: f32,
) -> (f32, f32, f32) {
    let u = interp(&u, x, y - 0.5, z - 0.5);
    let v = interp(&v, x - 0.5, y, z - 0.5);
    let w = interp(&w, x - 0.5, y - 0.5, z);

    let px = x - u * dt;
    let py = y - v * dt;
    let pz = z - w * dt;

    (px, py, pz)
}

impl FluidSim {
    pub fn new(width: usize, height: usize, length: usize) -> Self {
        assert_eq!(width, height);
        assert_eq!(width, length);
        let k = width + 1;
        let empty = FluidState {
            u: Array3D::new(k, k, k),
            v: Array3D::new(k, k, k),
            w: Array3D::new(k, k, k),
        };

        Self {
            read: empty.clone(),
            write: empty,
        }
    }

    pub fn step(&mut self, dt: f32, overstep: f32, n_iters: u32) {
        // Force incompressibility
        for _ in 0..n_iters {
            for z in 1..self.read.v.length() - 2 {
                for y in 1..self.read.v.height() - 2 {
                    for x in 1..self.read.u.width() - 2 {
                        let dx = self.read.u[(x + 1, y, z)] - self.read.u[(x, y, z)];
                        let dy = self.read.v[(x, y + 1, z)] - self.read.v[(x, y, z)];
                        let dz = self.read.w[(x, y, z + 1)] - self.read.w[(x, y, z)];

                        let d = overstep * (dx + dy + dz) / 6.;

                        self.read.u[(x, y, z)] += d;
                        self.read.u[(x + 1, y, z)] -= d;

                        self.read.v[(x, y, z)] += d;
                        self.read.v[(x, y + 1, z)] -= d;

                        self.read.w[(x, y, z)] += d;
                        self.read.w[(x, y, z + 1)] -= d;
                    }
                }
            }
            let l = self.read.u.height();

            for (i, arr) in [&mut self.read.u, &mut self.read.v, &mut self.read.w]
                .into_iter()
                .enumerate()
            {
                for (a, b) in [(l - 1, l - 2), (2, 1)] {
                    for u in 0..l {
                        for v in 0..l {
                            let mut pa = [a, u, v];
                            let mut pb = [b, u, v];
                            pa.rotate_right(i);
                            pb.rotate_right(i);

                            fn shonk([x, y, z]: [usize; 3]) -> (usize, usize, usize) {
                                (x, y, z)
                            }

                            arr[shonk(pb)] = -arr[shonk(pa)];
                        }
                    }
                }
            }
        }

        // Advect velocity
        for z in 1..self.read.u.length() - 1 {
            for y in 1..self.read.u.height() - 1 {
                for x in 1..self.read.u.width() - 1 {
                    let (px, py, pz) = advect(
                        &self.read.u,
                        &self.read.v,
                        &self.read.w,
                        x as f32,
                        y as f32 + 0.5,
                        z as f32 + 0.5,
                        dt,
                    );
                    self.write.u[(x, y, z)] = interp(&self.read.u, px, py - 0.5, pz - 0.5);

                    let (px, py, pz) = advect(
                        &self.read.u,
                        &self.read.v,
                        &self.read.w,
                        x as f32 + 0.5,
                        y as f32,
                        z as f32 + 0.5,
                        dt,
                    );
                    self.write.v[(x, y, z)] = interp(&self.read.v, px - 0.5, py, pz - 0.5);

                    let (px, py, pz) = advect(
                        &self.read.u,
                        &self.read.v,
                        &self.read.w,
                        x as f32 + 0.5,
                        y as f32 + 0.5,
                        z as f32,
                        dt,
                    );
                    self.write.w[(x, y, z)] = interp(&self.read.w, px - 0.5, py - 0.5, pz);
                }
            }
        }

        // Swap the written buffers back into read again
        std::mem::swap(&mut self.read.u, &mut self.write.u);
        std::mem::swap(&mut self.read.v, &mut self.write.v);
        std::mem::swap(&mut self.read.w, &mut self.write.w);
    }

    pub fn uvw(&self) -> (&Array3D<f32>, &Array3D<f32>, &Array3D<f32>) {
        (&self.read.u, &self.read.v, &self.read.w)
    }

    pub fn uv_mut(&mut self) -> (&mut Array3D<f32>, &mut Array3D<f32>, &mut Array3D<f32>) {
        (&mut self.read.u, &mut self.read.v, &mut self.read.w)
    }

    pub fn width(&self) -> usize {
        self.read.u.width()
    }

    pub fn height(&self) -> usize {
        self.read.u.height()
    }
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1. - t) * a + t * b
}

/// Bilinear interpolation of the given grid at the given coordinates
#[track_caller]
fn interp(grid: &Array3D<f32>, x: f32, y: f32, z: f32) -> f32 {
    // Bounds enforcement. No panics!
    let tl_x = (x.floor() as isize).clamp(0, grid.width() as isize - 1) as usize;
    let tl_y = (y.floor() as isize).clamp(0, grid.height() as isize - 1) as usize;
    let tl_z = (z.floor() as isize).clamp(0, grid.length() as isize - 1) as usize;

    // Get corners
    let tlu = grid[(tl_x, tl_y, tl_z)];
    let tru = grid[(tl_x + 1, tl_y, tl_z)];
    let blu = grid[(tl_x, tl_y + 1, tl_z)];
    let bru = grid[(tl_x + 1, tl_y + 1, tl_z)];

    let tld = grid[(tl_x, tl_y, tl_z + 1)];
    let trd = grid[(tl_x + 1, tl_y, tl_z + 1)];
    let bld = grid[(tl_x, tl_y + 1, tl_z + 1)];
    let brd = grid[(tl_x + 1, tl_y + 1, tl_z + 1)];

    // Bilinear interpolation
    lerp(
        lerp(
            lerp(tlu, tru, x.fract()), // Top row
            lerp(blu, bru, x.fract()), // Bottom row
            y.fract(),
        ),
        lerp(
            lerp(tld, trd, x.fract()), // Top row
            lerp(bld, brd, x.fract()), // Bottom row
            y.fract(),
        ),
        z.fract(),
    )
}

#[derive(Clone)]
pub struct Array3D<T> {
    width: usize,
    height: usize,
    length: usize,
    data: Vec<T>,
}

pub type Index3D = (usize, usize, usize);

impl<T> Array3D<T> {
    pub fn from_array(width: usize, height: usize, data: Vec<T>) -> Self {
        assert_eq!(data.len() % (width * height), 0);
        assert_eq!(data.len() % width, 0);
        let length = data.len() / (width * height);

        Self {
            width,
            height,
            length,
            data,
        }
    }

    pub fn new(width: usize, height: usize, length: usize) -> Self
    where
        T: Default + Copy,
    {
        Self {
            width,
            height,
            length,
            data: vec![T::default(); width * height * length],
        }
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    #[track_caller]
    fn calc_index(&self, (x, y, z): Index3D) -> usize {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        debug_assert!(z < self.length);
        x + (y * self.width) + z * (self.width * self.height)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl<T> std::ops::Index<Index3D> for Array3D<T> {
    type Output = T;
    fn index(&self, pos: Index3D) -> &T {
        &self.data[self.calc_index(pos)]
    }
}

impl<T> std::ops::IndexMut<Index3D> for Array3D<T> {
    fn index_mut(&mut self, pos: Index3D) -> &mut T {
        let idx = self.calc_index(pos);
        &mut self.data[idx]
    }
}

fn draw_velocity_lines(
    mesh: &mut Mesh,
    (u, v, w): (&Array3D<f32>, &Array3D<f32>, &Array3D<f32>),
    _z: f32,
) {
    mesh.indices.clear();
    mesh.vertices.clear();

    let cell_width = 2. / u.width() as f32;
    let cell_height = 2. / u.height() as f32;

    let step = 3;

    for i in (0..u.width()).step_by(step) {
        let i_frac = (i as f32 / u.width() as f32) * 2. - 1.;
        for j in (0..u.height()).step_by(step) {
            let j_frac = (j as f32 / u.height() as f32) * 2. - 1.;

            for k in (0..u.length()).step_by(step) {
                let k_frac = (k as f32 / u.length() as f32) * 2. - 1.;

                let u = u[(i, j, k)];
                let v = v[(i, j, k)];
                let w = w[(i, j, k)];

                let speed = (u.powf(2.) + v.powf(2.)).sqrt();

                let color = [1., 0., 0.5];

                let mut push = |x: f32, y: f32, z: f32, color: [f32; 3]| {
                    let pos = [x, y, z];
                    let base = mesh.vertices.len() as u32;
                    mesh.vertices.push(Vertex::new(pos, color));
                    base
                };

                let tail_x = i_frac + cell_width / 2.;
                let tail_y = j_frac + cell_height / 2.;
                let tail_z = k_frac + cell_height / 2.;
                let tail = push(tail_x, tail_y, tail_z, color);

                let len = cell_height / speed;
                let tip = push(tail_x + u * len, tail_y + v * len, tail_z + w * len, color);

                mesh.indices.push(tip);
                mesh.indices.push(tail);
            }
        }
    }
}

/// Push a fluid body give a relative velocity and position (assumes -1 to 1 fluid volume in world
/// size, matching draw_velocity_lines & friends)
fn push_fluid(state: &mut FluidSim, brush_size: isize, pos: Vec3, vel: Vec3) {
    let (u, v, w) = state.uv_mut();
    let width = u.width() as isize;

    // Convert to fluid units
    let vel = vel * width as f32 / 2.;
    let pos = ((pos + 1.) / 2.) * width as f32;

    let margin = 1;

    let bounds = |pos| (pos > margin && pos < width - margin).then(|| pos as usize);
    let cx = pos.x as isize;
    let cy = pos.y as isize;
    let cz = pos.z as isize;

    // Size of the cube of force around the hand will have a width of 2m + 1
    let m = brush_size;
    for i in cx - m..=cx + m {
        for j in cy - m..=cy + m {
            for k in cz - m..=cz + m {
                let Some(x) = bounds(i) else { continue };
                let Some(y) = bounds(j) else { continue };
                let Some(z) = bounds(k) else { continue };

                u[(x, y, z)] = vel.x;
                v[(x, y, z)] = vel.y;
                w[(x, y, z)] = vel.z;
            }
        }
    }
}

#[derive(Default)]
struct HandMotion {
    aim_pos: Option<Vec3>,
    grip_pos: Option<Vec3>,
    vel: Vec3,
    // TODO: Angular velocity!
}

struct VrTracking {
    left: HandMotion,
    right: HandMotion,
}

impl VrTracking {
    pub fn new() -> Self {
        Self {
            left: HandMotion::default(),
            right: HandMotion::default(),
        }
    }

    pub fn update(&mut self, update: &VrUpdate) {
        let VrUpdate {
            left_controller,
            right_controller,
            ..
        } = update;

        for (controller, last) in [
            (left_controller, &mut self.left),
            (right_controller, &mut self.right),
        ] {
            if let Some(aim) = controller.aim {
                last.vel = last.aim_pos.map(|p| aim.pos - p).unwrap_or(Vec3::ZERO);
                last.aim_pos = Some(aim.pos);
            }

            if let Some(grip) = controller.grip {
                last.grip_pos = Some(grip.pos);
            }
        }
    }
}
