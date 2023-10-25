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
    pub speed: Option<f32>,
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
    Time: AnimationTime + std::fmt::Debug,
{
    pub fn new(position: f32) -> Self {
        AnimatedValue {
            position,
            started_ms: None,
            last_tick_ms: None,
            from: position,
            to: None,
            duration_ms: 0.0,
            speed: None,
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
            if self.speed.is_none() {
                self.speed = Some(f32::abs((self.to.unwrap() - self.from) / self.duration_ms));
            }
            self.from = self.timed_progress();
            self.position = self.from;
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
            let position_delta: f32;
            if let Some(speed) = self.speed {
                let signed_speed = if self.from > self.to.unwrap() { -speed } else { speed };
                position_delta = elapsed * signed_speed;
            } else {
                let duration = self.duration_ms;
                let delta = elapsed / duration;
                position_delta =
                    delta * (self.to.unwrap_or(self.from) - self.from);
            }
            let mut finished = false;
            if self.duration_ms == 0.0 {
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
            self.last_tick_ms = Some(now);
            if finished {
                if let Some(to) = self.to {
                    self.from = to;
                    self.position = to;
                }
                self.started_ms = None;
                self.to = None;
                self.last_tick_ms = None;
                self.speed = None;
                return true;
            }
            return true;
        };
        false
    }

    pub fn timed_progress(self) -> f32 {
        let beginning = self.from;
        let end = self.to.unwrap_or(self.from);
        let current = self.position - beginning;
        let range = end - beginning;
        let completion = current / range;
        let timing: f32;
        if range != 0.0 {
            timing = beginning + (self.timing.timing(completion) * range);
        } else {
            timing = self.position;
        }
        return timing;
    }

    pub fn animating(self) -> bool {
        self.to.is_some()
    }
}
#[cfg(test)]
mod animatedvalue_tests {
    use super::*;

    #[test]
    fn test_instant_animation() {
        let mut anim = AnimatedValue::<f32>::new(0.0);
        let clock = 0.0;
        // If animation duration is 0.0 the transition should happen instantly
        // & require a redraw without any time passing
        assert_eq!(anim.position, 0.0);
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        assert_eq!(anim.position, 0.0);
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 10.0);
    }

    #[test]
    fn test_progression() {
        let mut anim = AnimatedValue::<f32>::new(0.0);
        let mut clock = 0.0;
        // With a duration of 1.0 & linear timing we should be halfway to our
        // destination at 0.5
        anim.duration_ms = 1.0;
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 5.0);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 10.0);

        // Progression backward
        anim.duration_ms = 0.5;
        anim.transition(clock, |animation| {
            *animation = 0.0;
        });
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 0.0);

        // Progression forward in thirds
        anim.duration_ms = 1.0;
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 2.0));
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 4.0));
        clock += 0.4;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 8.0));
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 10.0));
    }
    #[test]
    fn test_interrupt() {
        let mut anim = AnimatedValue::<f32>::new(0.0);
        let mut clock = 0.0;
        anim.duration_ms = 1.0;
        // Interruptions should continue at the same speed the interrupted
        // animation was progressing at.
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 5.0);
        // If we interrupt exactly halfway through distance & duration we
        // should arrive back at the start with another half of the duration
        anim.transition(clock, |animation| {
            *animation = 0.0;
        });
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 0.0);
        assert!(anim.to.is_none());
        assert!(!anim.animating());
        assert!(anim.speed.is_none());

        // Begin an animation
        anim.transition(clock, |animation| {
            *animation = 10.0;
        });
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(anim.animating());
        assert!(approximately_equal(anim.position, 2.0));
        // Interrupt one fifth of the way through
        // The animation is playing at 10 units per time unit
        // The target is only 1.0 away
        // We should arrive at the target after 0.1 time units
        anim.transition(clock, |animation| {
            *animation = 1.0;
        });
        clock += 0.100001;
        dbg!(anim.position);
        assert!(anim.tick(clock));
        dbg!(anim.position);
        assert!(!anim.animating());
        assert!(approximately_equal(anim.position, 1.0));
    }

    #[test]
    fn test_interrupt_nonlinear() {
        let mut anim = AnimatedValue::<f32>::new(1.0);
        let mut clock = 0.0;
        anim.duration_ms = 10.0;
        anim.timing = Timing::EaseIn;

        // Interrupt halfway through with asymmetrical timing
        anim.transition(clock, |animation| {
            *animation = 0.0;
        });
        assert!(anim.animating());
        assert_eq!(anim.position, 1.0);
        clock += 1.0;
        assert!(anim.tick(clock));
        let progress_at_interrupt = anim.timed_progress();
        assert_eq!(progress_at_interrupt, 1.0 - Timing::EaseIn.timing(0.1));

        // Interrupted animation should begin from wherever the timed function
        // was interrupted, which is different from the linear progress.
        anim.transition(clock, |animation| {
            *animation = 1.0;
        });
        assert_eq!(anim.to, Some(1.0));
        assert_eq!(anim.timed_progress(), progress_at_interrupt);
        assert!(anim.animating());
        assert!(anim.speed.is_some());
        // Since we've interrupted at some in-between, non-linear point in
        // the animation, the time it takes to finish won't be as clean.
        // It should take a bit less time to return home because it's an
        // EaseIn timing curve. The animation we interrupted was easing in
        // & therefore closer to where it started.
        clock += 3.0;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 1.0);
        assert!(anim.to.is_none());
        assert!(!anim.animating());
        assert!(anim.speed.is_none());
    }

    impl AnimationTime for f32 {
        fn ms_elapsed_since(self, time: Self) -> f32 {
            self - time
        }
    }
    fn approximately_equal(a: f32, b: f32) -> bool {
        let close = f32::abs(a - b) < 1e-5;
        if !close {
            dbg!(a, b);
        }
        close
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
