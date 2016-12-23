const US_TO_NS: u64 = 1000;

const CPU_CYCLE_PERIOD_NS: u64 = 50;

const LARGE_INTERVAL_PERIOD_NS: u64 = 100 * US_TO_NS;
const SMALL_INTERVAL_PERIOD_NS: u64 = 20 * US_TO_NS;

enum Interval {
    Large,
    Small,
}

pub struct Timer {
    interval: Interval,
    zero_interrupt_enable: bool,
    zero_status: bool,
    enable: bool,
    reload: u16,
    counter: u16,

    tick_counter: u64,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            interval: Interval::Large,
            zero_interrupt_enable: false,
            zero_status: false,
            enable: false,
            reload: 0,
            counter: 0,

            tick_counter: 0,
        }
    }

    pub fn read_control_reg(&self) -> u8 {
        (match self.interval {
            Interval::Large => 0,
            Interval::Small => 1,
        } << 4) |
        (if self.zero_interrupt_enable { 1 } else { 0 } << 3) |
        (if self.zero_status { 1 } else { 0 } << 1) |
        if self.enable { 1 } else { 0 }
    }

    pub fn write_control_reg(&mut self, value: u8) {
        self.interval = if ((value >> 4) & 0x01) == 0 {
            Interval::Large
        } else {
            Interval::Small
        };
        self.zero_interrupt_enable = ((value >> 3) & 0x01) != 0;
        if ((value >> 2) & 0x01) != 0 && !self.zero_interrupt_enable {
            self.zero_status = false;
        }
        self.enable = (value & 0x01) != 0;
    }

    pub fn read_counter_reload_low_reg(&self) -> u8 {
        self.counter as _
    }

    pub fn write_counter_reload_low_reg(&mut self, value: u8) {
        self.reload |= value as u16;
        self.counter = self.reload;
    }

    pub fn read_counter_reload_high_reg(&self) -> u8 {
        (self.counter >> 8) as _
    }

    pub fn write_counter_reload_high_reg(&mut self, value: u8) {
        self.reload |= (value as u16) << 8;
        self.counter = self.reload;
    }

    pub fn cycles(&mut self, cycles: usize) -> bool {
        if self.enable {
            for _ in 0..cycles {
                let tick_period = match self.interval {
                    Interval::Large => LARGE_INTERVAL_PERIOD_NS,
                    Interval::Small => SMALL_INTERVAL_PERIOD_NS,
                };
                self.tick_counter += CPU_CYCLE_PERIOD_NS;
                if self.tick_counter >= tick_period {
                    self.tick_counter = 0;

                    self.counter = match self.counter {
                        0 => {
                            self.zero_status = true;
                            self.reload
                        },
                        _ => self.counter - 1
                    };
                }
            }
        }

        false//self.zero_interrupt_enable && self.zero_status
    }
}
