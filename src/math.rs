use bevy::math::{Vec2, Vec3};

pub struct CylinderCoordinates3d {
    // distance to the center
    pub distance: f32,
    // rotation about the center
    pub rotation: f32,
    // height about the cylinder
    pub height: f32,
}

// TODO move smooth damp to own module

// variables for smooth damp transition
pub struct SmoothDampTransitionVariables<T> {
    pub velocity: T,
}

// parameters for smooth damp transition of cylinder coordinates 3d.
pub struct CylinderCoordinates3dSmoothDampTransitionVariables {
    pub distance: SmoothDampTransitionVariables<f32>,
    pub rotation: SmoothDampTransitionVariables<f32>,
    pub height: SmoothDampTransitionVariables<f32>,
}

pub trait SmoothDamp
where
    Self: Sized,
{
    fn smooth_damp(
        self,
        target: Self,
        current_velocity: Self,
        smooth_time: f32,
        max_speed: f32,
        delta_time: f32,
    ) -> (Self, Self);
}

impl SmoothDamp for Vec3 {
    fn smooth_damp(
        self,
        mut target: Self,
        mut current_velocity: Vec3,
        mut smooth_time: f32,
        max_speed: f32,
        delta_time: f32,
    ) -> (Self, Vec3) {
        let mut output_x: f32;
        let mut output_y: f32;
        let mut output_z: f32;

        // Based on Game Programming Gems 4 Chapter 1.10
        smooth_time = f32::max(0.0001, smooth_time);
        let omega = 2.0 / smooth_time;

        let x = omega * delta_time;
        let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

        let mut change_x = self.x - target.x;
        let mut change_y = self.y - target.y;
        let mut change_z = self.z - target.z;
        let original_to = target;

        // Clamp maximum speed
        let max_change = max_speed * smooth_time;

        let max_change_sq = max_change * max_change;
        let sqrmag = change_x * change_x + change_y * change_y + change_z * change_z;
        if sqrmag > max_change_sq {
            let mag = f32::sqrt(sqrmag);
            change_x = change_x / mag * max_change;
            change_y = change_y / mag * max_change;
            change_z = change_z / mag * max_change;
        }

        target.x = self.x - change_x;
        target.y = self.y - change_y;
        target.z = self.z - change_z;

        let temp_x = (current_velocity.x + omega * change_x) * delta_time;
        let temp_y = (current_velocity.y + omega * change_y) * delta_time;
        let temp_z = (current_velocity.z + omega * change_z) * delta_time;

        current_velocity.x = (current_velocity.x - omega * temp_x) * exp;
        current_velocity.y = (current_velocity.y - omega * temp_y) * exp;
        current_velocity.z = (current_velocity.z - omega * temp_z) * exp;

        output_x = target.x + (change_x + temp_x) * exp;
        output_y = target.y + (change_y + temp_y) * exp;
        output_z = target.z + (change_z + temp_z) * exp;

        // Prevent overshooting
        let orig_minus_current_x = original_to.x - self.x;
        let orig_minus_current_y = original_to.y - self.y;
        let orig_minus_current_z = original_to.z - self.z;
        let out_minus_orig_x = output_x - original_to.x;
        let out_minus_orig_y = output_y - original_to.y;
        let out_minus_orig_z = output_z - original_to.z;

        if orig_minus_current_x * out_minus_orig_x
            + orig_minus_current_y * out_minus_orig_y
            + orig_minus_current_z * out_minus_orig_z
            > 0.0
        {
            output_x = original_to.x;
            output_y = original_to.y;
            output_z = original_to.z;

            current_velocity.x = (output_x - original_to.x) / delta_time;
            current_velocity.y = (output_y - original_to.y) / delta_time;
            current_velocity.z = (output_z - original_to.z) / delta_time;
        }

        return (Vec3::new(output_x, output_y, output_z), current_velocity);
    }
}

impl SmoothDamp for f32 {
    fn smooth_damp(
        self,
        mut target: Self,
        mut current_velocity: f32,
        mut smooth_time: f32,
        max_speed: f32,
        delta_time: f32,
    ) -> (Self, f32) {
        // Based on Game Programming Gems 4 Chapter 1.10
        // https://stackoverflow.com/questions/61372498/how-does-mathf-smoothdamp-work-what-is-it-algorithm
        smooth_time = f32::max(0.0001, smooth_time);
        let omega: f32 = 2.0 / smooth_time;

        let x: f32 = omega * delta_time;
        let exp: f32 = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
        let mut change = self - target;
        let original_to = target;

        // Clamp maximum speed
        let max_change = max_speed * smooth_time;
        change = f32::clamp(change, -max_change, max_change);
        target = self - change;

        let temp = (current_velocity + omega * change) * delta_time;
        current_velocity = (current_velocity - omega * temp) * exp;
        let mut output = target + (change + temp) * exp;

        // Prevent overshooting
        if (original_to - self > 0.0) == (output > original_to) {
            output = original_to;
            current_velocity = (output - original_to) / delta_time;
        }

        return (output, current_velocity);
    }
}

pub trait MoveTowards {
    fn move_towards(self, target: Self, max_delta: f32) -> Self;
}

impl MoveTowards for f32 {
    fn move_towards(self, target: Self, max_delta: f32) -> f32 {
        if f32::abs(target - self) <= max_delta {
            return target;
        }
        return self + f32::signum(target - self) * max_delta;
    }
}

impl MoveTowards for Vec2 {
    fn move_towards(self, target: Vec2, max_delta: f32) -> Vec2 {
        let to_vector_x: f32 = target.x - self.x;
        let to_vector_y: f32 = target.y - self.y;

        let square_distance: f32 = to_vector_x * to_vector_x + to_vector_y * to_vector_y;

        if square_distance == 0.0 || (max_delta >= 0.0 && square_distance <= max_delta * max_delta)
        {
            return target;
        }

        let distance: f32 = f32::sqrt(square_distance);

        return Vec2::new(
            self.x + to_vector_x / distance * max_delta,
            self.y + to_vector_y / distance * max_delta,
        );
    }
}

impl MoveTowards for Vec3 {
    fn move_towards(self, target: Vec3, max_delta: f32) -> Vec3 {
        let to_vector_x: f32 = target.x - self.x;
        let to_vector_y: f32 = target.y - self.y;
        let to_vector_z: f32 = target.z - self.z;

        let square_distance: f32 =
            to_vector_x * to_vector_x + to_vector_y * to_vector_y + to_vector_z * to_vector_z;

        if square_distance == 0.0 || (max_delta >= 0.0 && square_distance <= max_delta * max_delta)
        {
            return target;
        }

        let distance: f32 = f32::sqrt(square_distance);

        return Vec3::new(
            self.x + to_vector_x / distance * max_delta,
            self.y + to_vector_y / distance * max_delta,
            self.z + to_vector_z / distance * max_delta,
        );
    }
}
