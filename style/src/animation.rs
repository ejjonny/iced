pub trait Animatable {
    fn on_redraw_request_update(&mut self, now: std::time::Instant) -> bool;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct AnimatedValue<Time> {
    pub position: f32,
    pub started_ms: Option<Time>,
    pub last_tick_ms: Option<Time>,
    pub from: f32,
    pub to: Option<f32>,
    pub duration_ms: f32,
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

pub trait AnimationTime: Copy {
    fn ms_elapsed_since(self, time: Self) -> f32;
}
impl AnimationTime for std::time::Instant {
    fn ms_elapsed_since(self, time: Self) -> f32 {
        (self - time).as_millis() as f32
    }
}

impl<Time> AnimatedValue<Time>
where
    Time: AnimationTime,
{
    pub fn new(position: f32) -> Self {
        AnimatedValue {
            position: 0.0,
            started_ms: None,
            last_tick_ms: None,
            from: position,
            to: None,
            duration_ms: 0.0,
            timing: Timing::Linear,
        }
    }
    pub fn real_value(&self) -> f32 {
        self.to.unwrap_or(self.from)
    }
    pub fn transition<F>(&mut self, now: Time, update: F)
    where
        F: Fn(&mut f32),
    {
        let mut target = self.from.clone();
        update(&mut target);
        if self.animating() {
            // Snapshot current state as the new animation origin
            self.from = self.timing.timing(self.position);
            self.to = Some(target);
        } else {
            self.started_ms = Some(now);
            self.last_tick_ms = None;
        }
        self.from = self.position;
        self.to = Some(target);
    }
    pub fn tick(&mut self, now: Time) -> bool {
        if let Some(start) = self.started_ms {
            let elapsed =
                now.ms_elapsed_since(self.last_tick_ms.unwrap_or(start));
            let duration = self.duration_ms;
            let delta = elapsed / duration;
            dbg!(delta);
            let position_delta =
                delta * (self.to.unwrap_or(self.from) - self.from);
            dbg!(position_delta);
            let mut finished = false;
            if delta.is_nan() {
                finished = true;
            } else {
                self.position += position_delta;
                if position_delta.is_sign_positive()
                    && self.position >= self.to.unwrap()
                    || position_delta.is_sign_negative()
                        && self.position <= self.to.unwrap()
                {
                    finished = true;
                }
            }
            if finished {
                if let Some(to) = self.to {
                    self.from = to;
                    self.position = to;
                }
                self.started_ms = None;
                self.to = None;
                self.last_tick_ms = None;
            }
            self.last_tick_ms = Some(now);
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
        if range != 0.0 {
            dbg!(completion);
            timed = beginning + (self.timing.timing(completion) * range);
            // dbg!(timed, completion, current, range, beginning, self.timing.timing(completion));
        } else {
            timed = self.position;
        }
        return timed;
    }

    pub fn animating(self) -> bool {
        self.to.is_some()
    }
}
#[cfg(test)]
mod animatedvalue_tests {
    use super::*;

    #[test]
    fn test_interrupt() {
        let mut anim = AnimatedValue::<f32>::new(0.0);
        let mut clock = 0.0;
        // If animation duration is 0.0 the transition should happen instantly
        // & require a redraw without any time passing
        assert_eq!(anim.position, 0.0);
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        assert_eq!(anim.position, 0.0);
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 10.0);
        // With a duration of 1.0 & linear timing we should be halfway to our
        // destination at 0.5
        anim.duration_ms = 1.0;
        anim.transition(clock, |animation| {
            *animation = 0.0;
        });
        clock += 0.5;
        dbg!(anim);
        assert!(anim.tick(clock));
        dbg!(anim);
        assert_eq!(anim.position, 5.0);

        // Interrupting halfway should maintain the same speed
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        assert_eq!(anim.position, 5.0);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 7.5);
    }
    impl AnimationTime for f32 {
        fn ms_elapsed_since(self, time: Self) -> f32 {
            self - time
        }
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
