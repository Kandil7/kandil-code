use crate::core::hardware::{detect_hardware, HardwareProfile};
use std::{thread, time::Duration};

#[derive(Clone)]
pub struct AdaptiveUI {
    profile: HardwareProfile,
    latency_target: Duration,
}

impl AdaptiveUI {
    pub fn new(profile: HardwareProfile) -> Self {
        Self {
            profile,
            latency_target: Duration::from_millis(200),
        }
    }

    pub fn from_system() -> Self {
        Self::new(detect_hardware())
    }

    pub fn should_rich_render(&self) -> bool {
        self.profile.cpu_physical_cores >= 4 && self.profile.total_ram_gb >= 8
    }

    pub fn token_delay(&self) -> Duration {
        match self.profile.cpu_physical_cores {
            0..=2 => Duration::from_millis(50),
            3..=4 => Duration::from_millis(20),
            _ => Duration::from_millis(5),
        }
    }

    pub fn announce(&self, region: &str, message: &str) {
        if self.should_rich_render() {
            println!("[region=\"{}\"] {}", region, message);
        } else {
            println!("{}", message);
        }
    }

    pub fn progressive_render(&self, lines: &[String]) {
        if self.should_rich_render() {
            for line in lines {
                println!("{}", line);
                thread::sleep(self.token_delay());
            }
        } else if let Some(block) = lines.split_first() {
            println!("{}", block.0);
        }
    }

    pub fn rest_between_tokens(&self) {
        thread::sleep(self.token_delay());
    }
}
