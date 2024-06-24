pub struct Duration {
    sign: i8,
    weeks: Option<u32>,
    days: Option<u32>,
    hours: Option<u32>,
    minutes: Option<u32>,
    seconds: Option<u32>,
}

impl Duration {
    pub fn weeks(sign: i8, weeks: u32) -> Self {
        Duration {
            sign,
            weeks: Some(weeks),
            days: None,
            hours: None,
            minutes: None,
            seconds: None,
        }
    }

    pub fn days(sign: i8, days: u32) -> Self {
        Duration {
            sign,
            weeks: None,
            days: Some(days),
            hours: None,
            minutes: None,
            seconds: None,
        }
    }

    pub fn days_and_time(sign: i8, days: u32) -> DurationTimeBuilder {
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

    pub fn hours(sign: i8, hours: u32) -> DurationTimeMinutesBuilder {
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

    pub fn minutes(sign: i8, minutes: u32) -> DurationTimeSecondsBuilder {
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

    pub fn seconds(sign: i8, seconds: u32) -> Self {
        Duration {
            sign,
            weeks: None,
            days: None,
            hours: None,
            minutes: None,
            seconds: Some(seconds),
        }
    }
}

pub struct DurationTimeBuilder {
    duration: Duration,
}

impl DurationTimeBuilder {
    pub fn hours(mut self, hours: u32) -> DurationTimeMinutesBuilder {
        self.duration.hours = Some(hours);
        DurationTimeMinutesBuilder {
            duration: self.duration,
        }
    }

    pub fn minutes(mut self, minutes: u32) -> DurationTimeSecondsBuilder {
        self.duration.minutes = Some(minutes);
        DurationTimeSecondsBuilder {
            duration: self.duration,
        }
    }

    pub fn seconds(mut self, seconds: u32) -> Duration {
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
    pub fn minutes(mut self, minutes: u32) -> DurationTimeSecondsBuilder {
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
    pub fn seconds(mut self, seconds: u32) -> Duration {
        self.duration.seconds = Some(seconds);
        self.build()
    }

    pub fn build(self) -> Duration {
        self.duration
    }
}
