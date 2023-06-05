pub struct Physics {}

pub enum RigidBodyType {
    Dynamic,
    Fixed,
    //KinematicPositionBased
}
pub struct RigidBody {
    body_type: RigidBodyType,
}
