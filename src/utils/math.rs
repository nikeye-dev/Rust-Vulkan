pub use cgmath::Deg;
use cgmath::Rad;
pub use cgmath::SquareMatrix;
pub use cgmath::Zero;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type Quaternion = cgmath::Quaternion<f32>;
pub type Euler = cgmath::Euler<Deg<f32>>;
pub type EulerRad = cgmath::Euler<Rad<f32>>;
pub type Matrix3x3 = cgmath::Matrix3<f32>;
pub type Matrix4x4 = cgmath::Matrix4<f32>;

pub const VECTOR3_ONE: Vector3 = Vector3::new(1.0, 1.0, 1.0);
pub const VECTOR3_RIGHT: Vector3 = Vector3::new(1.0, 0.0, 0.0);
pub const VECTOR3_LEFT: Vector3 = Vector3::new(-1.0, 0.0, 0.0);
pub const VECTOR3_UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);
pub const VECTOR3_DOWN: Vector3 = Vector3::new(0.0, -1.0, 0.0);
pub const VECTOR3_FORWARD: Vector3 = Vector3::new(0.0, 0.0, 1.0);
pub const VECTOR3_BACKWARD: Vector3 = Vector3::new(0.0, 0.0, -1.0);
