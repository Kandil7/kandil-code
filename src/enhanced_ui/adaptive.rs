use crate::core::hardware::{detect_hardware, HardwareProfile};
use std::{thread, time::Duration};

#[derive(Clone)]
pub struct AdaptiveUI {
    profile: HardwareProfile,
    latency_target: Duration,
    accessibility_mode: AccessibilityMode,
    rendering_quality: RenderingQuality,
}

#[derive(Clone, Debug)]
pub enum AccessibilityMode {
    /// Standard accessibility features
    Standard,
    /// Enhanced accessibility (screen readers, high contrast)
    Enhanced,
    /// Accessibility for users with visual impairments
    VisualImpaired,
    /// Accessibility for users with hearing impairments
    HearingImpaired,
    /// Accessibility for users with motor impairments
    MotorImpaired,
}

#[derive(Clone, Debug)]
pub enum RenderingQuality {
    /// Minimal rendering for low-resource systems
    Low,
    /// Balanced rendering for most systems
    Medium,
    /// Rich rendering for high-resource systems
    High,
    /// Maximum rendering quality for capable systems
    Ultra,
}

impl AdaptiveUI {
    pub fn new(profile: HardwareProfile) -> Self {
        Self {
            profile,
            latency_target: Duration::from_millis(200),
            accessibility_mode: AccessibilityMode::Standard,
            rendering_quality: Self::determine_rendering_quality(&profile),
        }
    }

    pub fn from_system() -> Self {
        Self::new(detect_hardware())
    }

    /// Create UI optimized for specific accessibility needs
    pub fn with_accessibility_mode(mut self, mode: AccessibilityMode) -> Self {
        self.accessibility_mode = mode;
        self
    }

    /// Detect and return the system's accessibility mode
    pub fn detect_accessibility_mode() -> AccessibilityMode {
        // Check environment variables or system settings for accessibility features
        if std::env::var("HIGH_CONTRAST").is_ok()
            || std::env::var("SCREEN_READER").is_ok()
            || std::env::var("ACCESSIBILITY_ENABLED").is_ok() {
            AccessibilityMode::Enhanced
        } else {
            AccessibilityMode::Standard
        }
    }

    /// Determine rendering quality based on hardware profile
    fn determine_rendering_quality(profile: &HardwareProfile) -> RenderingQuality {
        // Calculate a performance score from 0-10 based on hardware specs
        let performance_score = {
            let ram_score = (profile.total_ram_gb as f64 / 16.0).min(1.0) * 3.0; // 0-3 based on RAM
            let cpu_score = (profile.cpu_physical_cores as f64 / 8.0).min(1.0) * 3.0; // 0-3 based on CPU
            let gpu_score = if profile.gpu.is_some() { 2.0 } else { 0.0 }; // 0-2 based on GPU
            let storage_score = if profile.free_disk_gb > 50.0 { 2.0 } else { 0.0 }; // 0-2 based on storage

            (ram_score + cpu_score + gpu_score + storage_score).round() as u32
        };

        match performance_score {
            0..=2 => RenderingQuality::Low,
            3..=5 => RenderingQuality::Medium,
            6..=8 => RenderingQuality::High,
            _ => RenderingQuality::Ultra,
        }
    }

    /// Check if the system should use rich rendering based on hardware
    pub fn should_rich_render(&self) -> bool {
        self.rendering_quality != RenderingQuality::Low
    }

    /// Check if accessibility features should be enhanced
    pub fn should_enhance_accessibility(&self) -> bool {
        matches!(self.accessibility_mode,
            AccessibilityMode::Enhanced |
            AccessibilityMode::VisualImpaired |
            AccessibilityMode::HearingImpaired |
            AccessibilityMode::MotorImpaired
        )
    }

    /// Get appropriate token delay based on hardware and accessibility settings
    pub fn token_delay(&self) -> Duration {
        let base_delay = match self.profile.cpu_physical_cores {
            0..=2 => Duration::from_millis(50),
            3..=4 => Duration::from_millis(20),
            5..=8 => Duration::from_millis(10),
            _ => Duration::from_millis(5),
        };

        // Increase delay if accessibility mode requires it
        if self.should_enhance_accessibility() {
            Duration::from_millis(base_delay.as_millis() as u64 * 2)
        } else {
            base_delay
        }
    }

    /// Announce a message with appropriate accessibility formatting
    pub fn announce(&self, region: &str, message: &str) {
        match self.accessibility_mode {
            AccessibilityMode::Standard => {
                if self.should_rich_render() {
                    println!("[region=\"{}\"] {}", region, message);
                } else {
                    println!("{}", message);
                }
            }
            AccessibilityMode::Enhanced | AccessibilityMode::VisualImpaired => {
                // Add screen reader friendly annotations
                println!("[{}] {}", region.to_uppercase(), message);
                println!("[SCREEN_READER: {}]", message); // For screen reader compatibility
            }
            AccessibilityMode::HearingImpaired => {
                // Provide visual cues for hearing-impaired users
                println!("[{}] {}", region.to_uppercase(), message);
            }
            AccessibilityMode::MotorImpaired => {
                // Reduce animations and provide easier navigation
                println!("[{}] {}", region, message);
                // Add extra spacing to reduce need for rapid scrolling
                println!();
            }
        }
    }

    /// Render content with progressive animation based on hardware capability
    pub fn progressive_render(&self, lines: &[String]) {
        match self.rendering_quality {
            RenderingQuality::Low => {
                // Fast, simple rendering for low-resource systems
                if let Some(block) = lines.split_first() {
                    println!("{}", block.0);
                }
            }
            RenderingQuality::Medium => {
                // Balanced progressive rendering
                for line in lines {
                    println!("{}", line);
                    thread::sleep(self.token_delay());
                }
            }
            RenderingQuality::High | RenderingQuality::Ultra => {
                // Rich progressive rendering with accessibility features
                for (i, line) in lines.iter().enumerate() {
                    match self.accessibility_mode {
                        AccessibilityMode::VisualImpaired => {
                            // Add extra delay for visual processing
                            println!("{}. {}", i + 1, line);
                            thread::sleep(Duration::from_millis(self.token_delay().as_millis() as u64 * 3));
                        }
                        AccessibilityMode::MotorImpaired => {
                            // Show each line with a pause for switch control users
                            println!("{}", line);
                            println!("[PAUSE - press any key to continue]"); // Simulated
                            thread::sleep(self.token_delay());
                        }
                        _ => {
                            println!("{}", line);
                            thread::sleep(self.token_delay());
                        }
                    }
                }
            }
        }
    }

    /// Add appropriate pause between UI token updates
    pub fn rest_between_tokens(&self) {
        thread::sleep(self.token_delay());
    }

    /// Get rendering quality for the current system
    pub fn rendering_quality(&self) -> &RenderingQuality {
        &self.rendering_quality
    }

    /// Get accessibility mode for the current system
    pub fn accessibility_mode(&self) -> &AccessibilityMode {
        &self.accessibility_mode
    }

    /// Check if this system has GPU acceleration available
    pub fn has_gpu_acceleration(&self) -> bool {
        self.profile.gpu.is_some()
    }

    /// Get estimated frame rate based on hardware capabilities
    pub fn estimated_frame_rate(&self) -> u32 {
        match &self.profile.gpu {
            Some(gpu) => {
                // Higher frame rate with dedicated GPU
                if gpu.memory_gb >= 4.0 {
                    60
                } else {
                    30
                }
            }
            None => {
                // Lower frame rate with integrated or no GPU
                if self.profile.cpu_physical_cores >= 4 {
                    30
                } else {
                    15
                }
            }
        }
    }

    /// Get a description of the UI capabilities based on hardware
    pub fn capabilities_description(&self) -> String {
        let quality_str = match self.rendering_quality {
            RenderingQuality::Low => "Basic",
            RenderingQuality::Medium => "Standard",
            RenderingQuality::High => "Enhanced",
            RenderingQuality::Ultra => "Premium",
        };

        let mut desc = format!("UI Quality: {}, ", quality_str);

        if self.has_gpu_acceleration() {
            desc.push_str("GPU acceleration: Available, ");
        } else {
            desc.push_str("GPU acceleration: Not available, ");
        }

        desc.push_str(&format!("Target FPS: {}-{}", self.estimated_frame_rate()/2, self.estimated_frame_rate()));

        desc
    }
}
