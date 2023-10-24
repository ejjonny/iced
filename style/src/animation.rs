pub trait Animatable {
    fn on_redraw_request_update(&mut self, now: std::time::Instant) -> bool;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct AnimatedValue {
    pub position: f32,
    pub started: Option<std::time::Instant>,
    pub last: Option<std::time::Instant>,
    pub from: f32,
    pub to: Option<f32>,
    pub duration: std::time::Duration,
    pub timing: Timing,
}

// impl Clone for AnimatedValue {
//     fn clone(&self) -> Self {
//         Self {
//             axis_location: self.linear_progress.clone(),
//             started: self.started.clone(),
//             last: self.last.clone(),
//             from: self.from.clone(),
//             to: self.to.clone(),
//             duration: self.duration.clone(),
//             timing: self.timing.clone(),
//             direction: self.direction.clone(),
//         }
//     }
// }

impl AnimatedValue {
    pub fn new(position: f32) -> Self {
        AnimatedValue {
            position: 0.0,
            started: None,
            last: None,
            from: position,
            to: None,
            duration: std::time::Duration::from_millis(500),
            timing: Timing::Linear,
        }
    }
    pub fn real_value(&self) -> f32 {
        self.to.unwrap_or(self.from)
    }
    pub fn transition<F>(&mut self, update: F)
    where
        F: Fn(&mut f32),
    {
        let mut target = self.from.clone();
        update(&mut target);
        if self.animating() {
            // Snapshot current state as the new animation origin
            self.from = self.position;
            self.to = Some(target);
        }
        self.started = Some(std::time::Instant::now());
        self.last = None;
        self.from = self.position;
        self.to = Some(target);
    }
    pub fn on_redraw_request_update(
        &mut self,
        now: std::time::Instant,
    ) -> bool {
        if let Some(start) = self.started {
            let elapsed = (now - self.last.unwrap_or(start)).as_millis() as f32;
            let duration = self.duration.as_millis() as f32;
            let delta = elapsed / duration; // * (self.to.unwrap_or(self.from) - self.from);
            let mut finished = false;
            if self.to.unwrap() > self.from {
                self.position += delta;
                if self.position >= self.to.unwrap() {
                    finished = true;
                }
            } else {
                self.position -= delta;
                if self.position <= self.to.unwrap() {
                    finished = true;
                }
            }
            if finished {
                if let Some(to) = self.to {
                    self.from = to;
                    self.position = to;
                }
                self.started = None;
                self.to = None;
                self.last = None;
            }
            self.last = Some(now);
            return true;
        };
        false
    }

    pub fn timed_progress(self) -> f32 {
        let beginning = self.from;
        let end = self.to.unwrap_or(self.from);
        let current = self.position - beginning;
        let range = end - beginning;
        let completion = f32::abs(current / range);
        let timed: f32;
        // If the animation switches directions & the asymmetrical timing curve also switches directions,
        // my linear time position will mean something else in curved time because my timing curve flipped!
        // How do I figure out the exact spot in curved time that corresponds with my current
        // position, but with a flipped timing curve???
        if range != 0.0 {
            timed = beginning + (self.timing.timing(completion) * range);
            dbg!(timed, completion, current, range, beginning, self.timing.timing(completion));
        } else {
            timed = self.position;
        }
        return timed
    }

    pub fn animating(self) -> bool {
        self.to.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Timing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuint,
    Custom,
}

impl Timing {
    fn timing(self, linear_progress: f32) -> f32 {
        let x = linear_progress;
        let pi = std::f32::consts::PI;
        match self {
            Timing::Linear => linear_progress,
            Timing::EaseIn => 1.0 - f32::cos((x * pi) / 2.0),
            Timing::EaseOut => f32::sin((x * pi) / 2.0),
            Timing::EaseInOut => -(f32::cos(pi * x) - 1.0) / 2.0,
            Timing::EaseInQuint => x * x * x * x * x,
            _ => linear_progress,
        }
    }
}

pub trait Interpolable {
    fn interpolated(self, other: Self, ratio: f32) -> Self;
}
impl Interpolable for crate::core::Color {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self.mixed(other, ratio)
    }
}
impl Interpolable for f32 {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self * (1.0 - ratio) + other * ratio
    }
}

impl<T> Interpolable for Option<T>
where
    T: Interpolable + Copy,
{
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.interpolated(b, ratio)),
            _ => other,
        }
    }
}
