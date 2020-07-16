use serde_derive::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum State {
    Alive(f64),
    Dead(f64),
}

#[derive(Debug)]
struct HeartbeatHistory {
    sample_size: usize,
    intervals: VecDeque<u128>,
    sum: u128,
    sum_squared: u128,
}

impl HeartbeatHistory {
    fn new(sample_size: usize) -> HeartbeatHistory {
        HeartbeatHistory {
            sample_size,
            intervals: VecDeque::new(),
            sum: 0,
            sum_squared: 0,
        }
    }

    fn add(&mut self, interval: u128) {
        self.intervals.push_back(interval);
        self.sum += interval;
        self.sum_squared += interval.pow(2);
        if self.intervals.len() > self.sample_size {
            if let Some(i) = self.intervals.pop_front() {
                self.sum -= i;
                self.sum_squared -= i.pow(2)
            }
        }
    }

    fn mean(&self) -> f64 {
        self.sum as f64 / (self.intervals.len() - 1) as f64
    }

    fn variance(&self) -> f64 {
        (self.sum_squared as f64 / (self.intervals.len() - 1) as f64) - self.mean().powi(2)
    }

    fn std_deviation(&self) -> f64 {
        self.variance().sqrt()
    }

    fn history(&self, num: usize) -> std::vec::Vec<u128> {
        self.intervals.iter().rev().cloned().take(num).collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub struct PhiAccrualFailureDetector {
    threshold: f64,
    sample_size: usize,
    min_std_deviation: f64,
    acceptable_heartbeat_pause: f64,
    first_heartbeat_estimate: u128,
    latest: u128,
    history: HeartbeatHistory,
}

pub struct PhiAccrualFailureDetectorBuilder(PhiAccrualFailureDetector);

impl PhiAccrualFailureDetectorBuilder {
    pub fn new() -> Self {
        let detector = PhiAccrualFailureDetector::new(16.0, 200, 500.0, 0.0, 500);
        PhiAccrualFailureDetectorBuilder(detector)
    }

    pub fn with_threshold(&mut self, threshold: f64) -> &mut PhiAccrualFailureDetectorBuilder {
        self.0.threshold = threshold;
        self
    }

    pub fn with_sample_size(
        &mut self,
        sample_size: usize,
    ) -> &mut PhiAccrualFailureDetectorBuilder {
        self.0.sample_size = sample_size;
        self
    }

    pub fn with_min_std_deviation(
        &mut self,
        min_std_deviation: f64,
    ) -> &mut PhiAccrualFailureDetectorBuilder {
        self.0.min_std_deviation = min_std_deviation;
        self
    }

    pub fn with_acceptable_heartbeat_pause(
        &mut self,
        acceptable_heartbeat_pause: f64,
    ) -> &mut PhiAccrualFailureDetectorBuilder {
        self.0.acceptable_heartbeat_pause = acceptable_heartbeat_pause;
        self
    }

    pub fn with_first_heartbeat_estimate(
        &mut self,
        first_heartbeat_estimate: u128,
    ) -> &mut PhiAccrualFailureDetectorBuilder {
        self.0.first_heartbeat_estimate = first_heartbeat_estimate;
        self
    }

    pub fn build(&mut self) -> PhiAccrualFailureDetector {
        let mut detector = PhiAccrualFailureDetector {
            history: HeartbeatHistory::new(self.0.sample_size),
            ..self.0
        };
        let std_deviation = detector.first_heartbeat_estimate / 4;
        detector
            .history
            .add(detector.first_heartbeat_estimate - std_deviation);
        detector
            .history
            .add(detector.first_heartbeat_estimate + std_deviation);

        detector
    }
}

impl PhiAccrualFailureDetector {
    pub fn new(
        threshold: f64,
        sample_size: usize,
        min_std_deviation: f64,
        acceptable_heartbeat_pause: f64,
        first_heartbeat_estimate: u128,
    ) -> Self {
        PhiAccrualFailureDetector {
            threshold,
            sample_size,
            min_std_deviation,
            acceptable_heartbeat_pause,
            first_heartbeat_estimate,
            latest: 0,
            history: HeartbeatHistory::new(sample_size),
        }
    }

    fn is_alive(&self, phi: f64) -> bool {
        return phi != 0.0 && phi < self.threshold;
    }

    pub fn state(&self, timestamp: u128) -> State {
        let phi = self.phi(timestamp);
        if self.is_alive(phi) {
            return State::Alive(phi);
        }
        State::Dead(phi)
    }

    pub fn is_available(&self, timestamp: u128) -> bool {
        let st = self.state(timestamp);
        match st {
            State::Alive(_) => true,
            State::Dead(_) => false,
        }
    }

    pub fn phi(&self, timestamp: u128) -> f64 {
        if self.latest == 0 {
            return 0.0;
        }

        let diff = (timestamp - self.latest) as f64;
        let mean = self.history.mean() + self.acceptable_heartbeat_pause as f64;
        let std_dev = self.ensure_std_deviation();

        let y = (diff - mean) / std_dev;
        let e = (-y * (1.5976 + 0.070566 * y * y)).exp();
        let cdf = if diff > mean {
            e / (1.0 + e)
        } else {
            1.0 - 1.0 / (1.0 + e)
        };

        -cdf.log10()
    }

    pub fn heartbeat(&mut self, timestamp: u128) {
        if self.latest > 0 {
            let interval = timestamp - self.latest;
            if self.is_available(timestamp) {
                self.history.add(interval);
            }
        }
        self.latest = timestamp;
    }

    pub fn history(&self, num: usize) -> std::vec::Vec<u128> {
        self.history.history(num)
    }

    pub fn last(&self) -> u128 {
        self.latest
    }

    fn ensure_std_deviation(&self) -> f64 {
        self.history.std_deviation().max(self.min_std_deviation)
    }
}

pub fn now() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

#[cfg(test)]
mod tests {
    use super::PhiAccrualFailureDetectorBuilder;

    #[test]
    fn should_fail_when_no_heartbeats() {
        let mut detector = PhiAccrualFailureDetectorBuilder::new().build();
        let now = super::now();

        for t in 0..100 {
            let tm = now + t * 1000;
            detector.heartbeat(tm);
            let phi = detector.phi(tm);
            println!("at:{:?}, phi:{:?}; det: {:?}", tm, phi, detector);
            if t > 10 {
                assert!(phi < 1.0);
            }
        }
        for t in 100..110 {
            let tm = now + t * 1000;
            let phi = detector.phi(tm);
            println!("at:{:?}, phi:{:?}; det: {:?}", tm, phi, detector);
        }
        for &t in &[110, 200, 300] {
            let tm = now + t * 1000;
            let phi = detector.phi(tm);
            println!("at:{:?}, phi:{:?}; det: {:?}", tm, phi, detector);
            assert!(phi > 1.0);
        }
    }

    #[test]
    fn should_recover() {
        let mut detector = PhiAccrualFailureDetectorBuilder::new().build();
        let now = super::now();

        for t in 0..10 {
            let tm = now + t * 1000;
            detector.heartbeat(tm);
            let phi = detector.phi(tm);
            println!("at:{:?}, phi:{:?}; det: {:?}", tm, phi, detector);
        }
        for t in 20..30 {
            let tm = now + t * 1000;
            detector.heartbeat(tm);
            let phi = detector.phi(tm);
            println!("at:{:?}, phi:{:?}; det: {:?}", tm, phi, detector);
            if t > 100 {
                assert!(phi < 1.0);
            }
        }
    }

    #[test]
    fn test_phi_fd() {
        let mut detector = PhiAccrualFailureDetectorBuilder::new().build();
        let now = super::now();
        for i in 0..300 {
            let timestamp = now + i * 1000;

            if i > 290 {
                let phi = detector.phi(timestamp);
                if i == 291 {
                    assert!(1.0 < phi && phi < 3.0);
                    assert!(detector.is_available(timestamp));
                } else if i == 292 {
                    assert!(3.0 < phi && phi < 8.0);
                    assert!(detector.is_available(timestamp));
                } else if i == 293 {
                    assert!(8.0 < phi && phi < 16.0);
                    assert!(detector.is_available(timestamp));
                } else if i == 294 {
                    assert!(16.0 < phi && phi < 30.0);
                    assert!(!detector.is_available(timestamp));
                } else if i == 295 {
                    assert!(30.0 < phi && phi < 50.0);
                    assert!(!detector.is_available(timestamp));
                } else if i == 296 {
                    assert!(50.0 < phi && phi < 70.0);
                    assert!(!detector.is_available(timestamp));
                } else if i == 297 {
                    assert!(70.0 < phi && phi < 100.0);
                    assert!(!detector.is_available(timestamp));
                } else {
                    assert!(100.0 < phi);
                    assert!(!detector.is_available(timestamp));
                }
                continue;
            } else if i > 200 {
                if i % 5 == 0 {
                    let phi = detector.phi(timestamp);
                    assert!(0.1 < phi && phi < 0.5);
                    assert!(detector.is_available(timestamp));
                    continue;
                }
            }
            detector.heartbeat(timestamp);
            assert!(detector.phi(timestamp) < 0.1);
            assert!(detector.is_available(timestamp));
        }
    }

    #[test]
    fn history_order() {
        let mut detector = PhiAccrualFailureDetectorBuilder::new().build();
        let now = super::now();
        let actual = vec![90, 110, 625, 375];

        for t in &[120, 230, 320] {
            let tm = now + t;
            detector.heartbeat(tm);
            assert!(detector.is_available(tm));
        }
        let hist = detector.history(5);
        assert_eq!(hist, actual);
    }
}
