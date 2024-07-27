#[derive(Clone, Debug)]
pub struct Duration {
    pub(crate) sign: i8,
    pub(crate) weeks: Option<u64>,
    pub(crate) days: Option<u64>,
    pub(crate) hours: Option<u64>,
    pub(crate) minutes: Option<u64>,
    pub(crate) seconds: Option<u64>,
}

impl Default for Duration {
    fn default() -> Self {
        Duration {
            sign: 1,
            weeks: None,
            days: None,
            hours: None,
            minutes: None,
            seconds: None,
        }
    }
}

impl Duration {
    pub fn weeks(sign: i8, weeks: u64) -> Self {
        Duration {
            sign,
            weeks: Some(weeks),
            days: None,
            hours: None,
            minutes: None,
            seconds: None,
        }
    }

    pub fn days(sign: i8, days: u64) -> Self {
        Duration {
            sign,
            weeks: None,
            days: Some(days),
            hours: None,
            minutes: None,
            seconds: None,
        }
    }

    pub fn days_and_time(sign: i8, days: u64) -> DurationTimeBuilder {
        DurationTimeBuilder {
            duration: Duration {
                sign,
                weeks: None,
                days: Some(days),
                hours: None,
                minutes: None,
                seconds: None,
            },
        }
    }

    pub fn hours(sign: i8, hours: u64) -> DurationTimeMinutesBuilder {
        DurationTimeMinutesBuilder {
            duration: Duration {
                sign,
                weeks: None,
                days: None,
                hours: Some(hours),
                minutes: None,
                seconds: None,
            },
        }
    }

    pub fn minutes(sign: i8, minutes: u64) -> DurationTimeSecondsBuilder {
        DurationTimeSecondsBuilder {
            duration: Duration {
                sign,
                weeks: None,
                days: None,
                hours: None,
                minutes: Some(minutes),
                seconds: None,
            },
        }
    }

    pub fn seconds(sign: i8, seconds: u64) -> Self {
        Duration {
            sign,
            weeks: None,
            days: None,
            hours: None,
            minutes: None,
            seconds: Some(seconds),
        }
    }

    pub fn to_std(self) -> (i8, std::time::Duration) {
        let secs = self
            .weeks
            .map(|weeks| weeks * 7 * 24 * 60 * 60)
            .unwrap_or(0)
            + self.days.map(|days| days * 24 * 60 * 60).unwrap_or(0)
            + self.hours.map(|hours| hours * 60 * 60).unwrap_or(0)
            + self.minutes.map(|minutes| minutes * 60).unwrap_or(0)
            + self.seconds.unwrap_or(0);

        (self.sign, std::time::Duration::from_secs(secs))
    }
}

pub struct DurationTimeBuilder {
    duration: Duration,
}

impl DurationTimeBuilder {
    pub fn hours(mut self, hours: u64) -> DurationTimeMinutesBuilder {
        self.duration.hours = Some(hours);
        DurationTimeMinutesBuilder {
            duration: self.duration,
        }
    }

    pub fn minutes(mut self, minutes: u64) -> DurationTimeSecondsBuilder {
        self.duration.minutes = Some(minutes);
        DurationTimeSecondsBuilder {
            duration: self.duration,
        }
    }

    pub fn seconds(mut self, seconds: u64) -> Duration {
        self.duration.seconds = Some(seconds);
        self.build()
    }

    pub fn build(self) -> Duration {
        self.duration
    }
}

pub struct DurationTimeMinutesBuilder {
    duration: Duration,
}

impl DurationTimeMinutesBuilder {
    pub fn minutes(mut self, minutes: u64) -> DurationTimeSecondsBuilder {
        self.duration.minutes = Some(minutes);
        DurationTimeSecondsBuilder {
            duration: self.duration,
        }
    }

    pub fn build(self) -> Duration {
        self.duration
    }
}

pub struct DurationTimeSecondsBuilder {
    duration: Duration,
}

impl DurationTimeSecondsBuilder {
    pub fn seconds(mut self, seconds: u64) -> Duration {
        self.duration.seconds = Some(seconds);
        self.build()
    }

    pub fn build(self) -> Duration {
        self.duration
    }
}
