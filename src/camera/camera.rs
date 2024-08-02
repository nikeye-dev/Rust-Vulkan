use cgmath::{InnerSpace, point3, SquareMatrix, Transform as tf};
use log::debug;
use crate::utils::math::{Matrix3x3, Matrix4x4, Quaternion, Vector3};
use crate::world::transform::{OwnedTransform, Transform};

#[derive(Default, Debug, Copy, Clone)]
pub struct ViewSettings {
    pub near: f32,
    pub far: f32,
    pub fov: f32
}

#[derive(Debug)]
pub struct Camera {
    view_settings: ViewSettings,
    transform: Transform
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_settings: ViewSettings { near: 0.1, far: 10000000000.0, fov: 80.0 },
            transform: Transform::identity()
        }
    }
}

impl OwnedTransform for Camera {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Camera {
    pub fn view(&self) -> ViewSettings {
        self.view_settings
    }

    pub fn view_matrix(&self) -> Matrix4x4 {
        let mut t = Matrix4x4::from_translation(self.transform().location());
        let mut r = Matrix4x4::from(self.transform().rotation().normalize());

        let trans = self.transform.location();

        // let x = cgmath::dot(trans, r.row(0));
        // let y = cgmath::dot(trans, r.row(1));
        // let z = cgmath::dot(trans, r.row(2));

        // let mut view2 = Matrix4x4::from(r);
        // view2[0][3] = -x;
        // view2[1][3] = -y;
        // view2[2][3] = -z;
        // view2[3][3] = 1.0;

        let local_t = r.transform_vector(trans);
        let t_mat = Matrix4x4::from_translation(local_t);
        let mut view2 = r * t;


        let loc = point3(self.transform.location().x, self.transform.location().y, self.transform.location().z);
        let look_at = loc + Vector3::new(0., 1., 0.);

        let view = Matrix4x4::look_to_rh(
            point3(self.transform.location().x, self.transform.location().y, self.transform.location().z),
            Vector3::new(0., 1., 0.),
            Vector3::new(0.0, 0.0, 1.0)
        );

        debug!("{:?} :::: {:?}", view2, view);

        view2
    }
}
