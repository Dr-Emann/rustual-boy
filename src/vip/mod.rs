mod mem_map;

use video_driver::*;
use self::mem_map::*;

const MS_TO_NS: u64 = 1000000;

const CPU_CYCLE_PERIOD_NS: u64 = 50;

const FRAME_CLOCK_PERIOD_MS: u64 = 20;
const FRAME_CLOCK_PERIOD_NS: u64 = FRAME_CLOCK_PERIOD_MS * MS_TO_NS;

const DISPLAY_PROCESSING_DELAY_PERIOD_MS: u64 = 10;
const DISPLAY_PROCESSING_DELAY_PERIOD_NS: u64 = DISPLAY_PROCESSING_DELAY_PERIOD_MS * MS_TO_NS;
const DISPLAY_PROCESSING_BUFFER_PERIOD_MS: u64 = 5;
const DISPLAY_PROCESSING_BUFFER_PERIOD_NS: u64 = DISPLAY_PROCESSING_BUFFER_PERIOD_MS * MS_TO_NS;

// Hardcoded drawing period for now
const DRAWING_PERIOD_MS: u64 = 10;
const DRAWING_PERIOD_NS: u64 = DRAWING_PERIOD_MS * MS_TO_NS;

const FRAMEBUFFER_RESOLUTION_X: usize = 384;
const FRAMEBUFFER_RESOLUTION_Y: usize = 256;
const DISPLAY_RESOLUTION_X: usize = 384;
const DISPLAY_RESOLUTION_Y: usize = 224;

enum DisplayState {
    Idle,
    LeftFramebuffer,
    RightFramebuffer,
}

enum DrawingState {
    Idle,
    Drawing,
}

enum Eye {
    Left,
    Right,
}

#[derive(PartialEq, Eq)]
enum WindowMode {
    Normal,
    LineShift,
    Affine,
    Obj,
}

#[derive(Debug, Clone, Copy)]
enum ObjGroup {
    Group0,
    Group1,
    Group2,
    Group3,
}

pub struct Vip {
    vram: Box<[u8]>,

    display_state: DisplayState,

    drawing_state: DrawingState,

    reg_interrupt_pending_left_display_finished: bool,
    reg_interrupt_pending_right_display_finished: bool,
    reg_interrupt_pending_drawing_started: bool,
    reg_interrupt_pending_start_of_frame_processing: bool,
    reg_interrupt_pending_drawing_finished: bool,

    reg_interrupt_enable_left_display_finished: bool,
    reg_interrupt_enable_right_display_finished: bool,
    reg_interrupt_enable_drawing_started: bool,
    reg_interrupt_enable_start_of_frame_processing: bool,
    reg_interrupt_enable_drawing_finished: bool,

    reg_display_control_display_enable: bool,
    reg_display_control_sync_enable: bool,

    reg_drawing_control_drawing_enable: bool,

    reg_game_frame_control: usize,

    reg_led_brightness_1: u8,
    reg_led_brightness_2: u8,
    reg_led_brightness_3: u8,

    reg_obj_group_0_ptr: u16,
    reg_obj_group_1_ptr: u16,
    reg_obj_group_2_ptr: u16,
    reg_obj_group_3_ptr: u16,

    reg_bg_palette_0: u8,
    reg_bg_palette_1: u8,
    reg_bg_palette_2: u8,
    reg_bg_palette_3: u8,
    reg_obj_palette_0: u8,
    reg_obj_palette_1: u8,
    reg_obj_palette_2: u8,
    reg_obj_palette_3: u8,

    reg_clear_color: u8,

    frame_clock_counter: u64,
    game_frame_clock_counter: usize,

    drawing_counter: u64,
    display_counter: u64,

    display_first_framebuffers: bool,
}

impl Vip {
    pub fn new() -> Vip {
        Vip {
            vram: vec![0xff; VRAM_LENGTH as usize].into_boxed_slice(),

            display_state: DisplayState::Idle,

            drawing_state: DrawingState::Idle,

            reg_interrupt_pending_left_display_finished: false,
            reg_interrupt_pending_right_display_finished: false,
            reg_interrupt_pending_drawing_started: false,
            reg_interrupt_pending_start_of_frame_processing: false,
            reg_interrupt_pending_drawing_finished: false,

            reg_interrupt_enable_left_display_finished: false,
            reg_interrupt_enable_right_display_finished: false,
            reg_interrupt_enable_drawing_started: false,
            reg_interrupt_enable_start_of_frame_processing: false,
            reg_interrupt_enable_drawing_finished: false,

            reg_display_control_display_enable: false,
            reg_display_control_sync_enable: false,

            reg_drawing_control_drawing_enable: false,

            reg_game_frame_control: 1,

            reg_led_brightness_1: 0,
            reg_led_brightness_2: 0,
            reg_led_brightness_3: 0,

            reg_obj_group_0_ptr: 0,
            reg_obj_group_1_ptr: 0,
            reg_obj_group_2_ptr: 0,
            reg_obj_group_3_ptr: 0,

            reg_bg_palette_0: 0,
            reg_bg_palette_1: 0,
            reg_bg_palette_2: 0,
            reg_bg_palette_3: 0,
            reg_obj_palette_0: 0,
            reg_obj_palette_1: 0,
            reg_obj_palette_2: 0,
            reg_obj_palette_3: 0,

            reg_clear_color: 0,

            frame_clock_counter: 0,
            game_frame_clock_counter: 0,

            drawing_counter: 0,
            display_counter: 0,

            display_first_framebuffers: false,
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match map_address(addr) {
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize]
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted read byte from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
            }
            _ => {
                let halfword = self.read_halfword(addr & 0xfffffffe);
                if (addr & 0x01) == 0 {
                    halfword as _
                } else {
                    (halfword >> 8) as _
                }
            }
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match map_address(addr) {
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value;
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted write byte to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:02x})", addr, value);
            }
            _ => {
                let halfword = if (addr & 0x01) == 0 {
                    value as _
                } else {
                    (value as u16) << 8
                };
                self.write_halfword(addr & 0xfffffffe, halfword);
            }
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::InterruptPendingReg => {
                //println!("WARNING: Read halfword from Interrupt Pending Reg not fully implemented");
                (if self.reg_interrupt_pending_left_display_finished { 1 } else { 0 } << 1) |
                (if self.reg_interrupt_pending_right_display_finished { 1 } else { 0 } << 2) |
                (if self.reg_interrupt_pending_drawing_started { 1 } else { 0 } << 3) |
                (if self.reg_interrupt_pending_start_of_frame_processing { 1 } else { 0 } << 4) |
                (if self.reg_interrupt_pending_drawing_finished { 1 } else { 0 } << 14)
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Read halfword from Interrupt Enable Reg not fully implemented");
                (if self.reg_interrupt_enable_left_display_finished { 1 } else { 0 } << 1) |
                (if self.reg_interrupt_enable_right_display_finished { 1 } else { 0 } << 2) |
                (if self.reg_interrupt_enable_drawing_started { 1 } else { 0 } << 3) |
                (if self.reg_interrupt_enable_start_of_frame_processing { 1 } else { 0 } << 4) |
                (if self.reg_interrupt_enable_drawing_finished { 1 } else { 0 } << 14)
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Attempted read halfword from Interrupt Clear Reg");
                0
            }
            MappedAddress::DisplayControlReadReg => {
                let scan_ready = true; // TODO
                // TODO: Not entirely sure this is correct
                let frame_clock = match self.display_state {
                    DisplayState::Idle => true,
                    _ => false
                };
                let mem_refresh = false; // TODO
                let column_table_addr_lock = false; // TODO

                (if self.reg_display_control_display_enable { 1 } else { 0 } << 1) |
                (match self.display_state {
                    DisplayState::Idle => 0b0000,
                    DisplayState::LeftFramebuffer => if self.display_first_framebuffers { 0b0001 } else { 0b0100 },
                    DisplayState::RightFramebuffer => if self.display_first_framebuffers { 0b0010 } else { 0b1000 },
                } << 2) |
                (if scan_ready { 1 } else { 0 } << 6) |
                (if frame_clock { 1 } else { 0 } << 7) |
                (if mem_refresh { 1 } else { 0 } << 8) |
                (if self.reg_display_control_sync_enable { 1 } else { 0 } << 9) |
                (if column_table_addr_lock { 1 } else { 0 } << 10)
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Attempted read halfword from Display Control Write Reg");
                0
            }
            MappedAddress::LedBrightness1Reg => self.reg_led_brightness_1 as _,
            MappedAddress::LedBrightness2Reg => self.reg_led_brightness_2 as _,
            MappedAddress::LedBrightness3Reg => self.reg_led_brightness_3 as _,
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Read halfword from LED Brightness Idle Reg not yet implemented");
                0
            }
            MappedAddress::GameFrameControlReg => {
                (self.reg_game_frame_control - 1) as u16
            }
            MappedAddress::DrawingControlReadReg => {
                let draw_to_first_framebuffers = !self.display_first_framebuffers;
                let (drawing_to_frame_buffer_0, drawing_to_frame_buffer_1) = match self.drawing_state {
                    DrawingState::Drawing => {
                        if draw_to_first_framebuffers {
                            (true, false)
                        } else {
                            (false, true)
                        }
                    }
                    _ => (false, false)
                };
                let drawing_exceeds_frame_period = false;
                let current_y_position = 0; // TODO
                let drawing_at_y_position = false;

                (if self.reg_drawing_control_drawing_enable { 1 } else { 0 } << 1) |
                (if drawing_to_frame_buffer_0 { 1 } else { 0 } << 2) |
                (if drawing_to_frame_buffer_1 { 1 } else { 0 } << 3) |
                (if drawing_exceeds_frame_period { 1 } else { 0 } << 4) |
                (current_y_position << 8) |
                (if drawing_at_y_position { 1 } else { 0 } << 15)
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Attempted read halfword from Drawing Control Write Reg");
                0
            }
            MappedAddress::ObjGroup0PointerReg => self.reg_obj_group_0_ptr,
            MappedAddress::ObjGroup1PointerReg => self.reg_obj_group_1_ptr,
            MappedAddress::ObjGroup2PointerReg => self.reg_obj_group_2_ptr,
            MappedAddress::ObjGroup3PointerReg => self.reg_obj_group_3_ptr,
            MappedAddress::BgPalette0Reg => self.reg_bg_palette_0 as _,
            MappedAddress::BgPalette1Reg => self.reg_bg_palette_1 as _,
            MappedAddress::BgPalette2Reg => self.reg_bg_palette_2 as _,
            MappedAddress::BgPalette3Reg => self.reg_bg_palette_3 as _,
            MappedAddress::ObjPalette0Reg => self.reg_obj_palette_0 as _,
            MappedAddress::ObjPalette1Reg => self.reg_obj_palette_1 as _,
            MappedAddress::ObjPalette2Reg => self.reg_obj_palette_2 as _,
            MappedAddress::ObjPalette3Reg => self.reg_obj_palette_3 as _,
            MappedAddress::ClearColorReg => self.reg_clear_color as _,
            MappedAddress::Vram(addr) => {
                (self.vram[addr as usize] as u16) |
                ((self.vram[addr as usize + 1] as u16) << 8)
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted read halfword from unrecognized VIP address (addr: 0x{:08x})", addr);
                0
            }
        }
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        let addr = addr & 0xfffffffe;
        match map_address(addr) {
            MappedAddress::InterruptPendingReg => {
                println!("WARNING: Attempted write halfword to Interrupt Pending Reg");
            }
            MappedAddress::InterruptEnableReg => {
                println!("WARNING: Write halfword to Interrupt Enable Reg not fully implemented (value: 0x{:04x})", value);
                self.reg_interrupt_enable_left_display_finished = (value & 0x0002) != 0;
                self.reg_interrupt_enable_right_display_finished = (value & 0x0004) != 0;
                self.reg_interrupt_enable_drawing_started = (value & 0x0008) != 0;
                self.reg_interrupt_enable_start_of_frame_processing = (value & 0x0010) != 0;
                self.reg_interrupt_enable_drawing_finished = (value & 0x4000) != 0;
            }
            MappedAddress::InterruptClearReg => {
                println!("WARNING: Write halfword to Interrupt Clear Reg not fully implemented (value: 0x{:04x})", value);
                if (value & 0x0002) != 0 {
                    self.reg_interrupt_pending_left_display_finished = false;
                }
                if (value & 0x0004) != 0 {
                    self.reg_interrupt_pending_right_display_finished = false;
                }
                if (value & 0x0008) != 0 {
                    self.reg_interrupt_pending_drawing_started = false;
                }
                if (value & 0x0010) != 0 {
                    self.reg_interrupt_pending_start_of_frame_processing = false;
                }
                if (value & 0x4000) != 0 {
                    self.reg_interrupt_pending_drawing_finished = false;
                }
            }
            MappedAddress::DisplayControlReadReg => {
                println!("WARNING: Attempted write halfword to Display Control Read Reg");
            }
            MappedAddress::DisplayControlWriteReg => {
                println!("WARNING: Write halfword to Display Control Write Reg not fully implemented (value: 0x{:04x})", value);
                let _reset = (value & 0x01) != 0; // TODO: Soft reset
                self.reg_display_control_display_enable = (value & 0x02) != 0;
                let _mem_refresh = (value & 0x10) != 0; // TODO
                self.reg_display_control_sync_enable = (value & 0x20) != 0;
                let _column_table_addr_lock = (value & 0x40) != 0;

                // TODO
            }
            MappedAddress::LedBrightness1Reg => self.reg_led_brightness_1 = value as _,
            MappedAddress::LedBrightness2Reg => self.reg_led_brightness_2 = value as _,
            MappedAddress::LedBrightness3Reg => self.reg_led_brightness_3 = value as _,
            MappedAddress::LedBrightnessIdleReg => {
                println!("WARNING: Write halfword to LED Brightness Idle Reg not yet implemented (value: 0x{:04x})", value);
            }
            MappedAddress::GameFrameControlReg => {
                println!("Game Frame Control written (value: 0x{:04x})", value);
                self.reg_game_frame_control = (value as usize) + 1;
            }
            MappedAddress::DrawingControlReadReg => {
                println!("WARNING: Attempted write halfword to Drawing Control Read Reg (value: 0x{:04x})", value);
            }
            MappedAddress::DrawingControlWriteReg => {
                println!("WARNING: Write halfword to Drawing Control Write Reg not fully implemented (value: 0x{:04x})", value);
                self.reg_drawing_control_drawing_enable = (value & 0x02) != 0;
            }
            MappedAddress::ObjGroup0PointerReg => self.reg_obj_group_0_ptr = value & 0x03ff,
            MappedAddress::ObjGroup1PointerReg => self.reg_obj_group_1_ptr = value & 0x03ff,
            MappedAddress::ObjGroup2PointerReg => self.reg_obj_group_2_ptr = value & 0x03ff,
            MappedAddress::ObjGroup3PointerReg => self.reg_obj_group_3_ptr = value & 0x03ff,
            MappedAddress::BgPalette0Reg => self.reg_bg_palette_0 = value as _,
            MappedAddress::BgPalette1Reg => self.reg_bg_palette_1 = value as _,
            MappedAddress::BgPalette2Reg => self.reg_bg_palette_2 = value as _,
            MappedAddress::BgPalette3Reg => self.reg_bg_palette_3 = value as _,
            MappedAddress::ObjPalette0Reg => self.reg_obj_palette_0 = value as _,
            MappedAddress::ObjPalette1Reg => self.reg_obj_palette_1 = value as _,
            MappedAddress::ObjPalette2Reg => self.reg_obj_palette_2 = value as _,
            MappedAddress::ObjPalette3Reg => self.reg_obj_palette_3 = value as _,
            MappedAddress::ClearColorReg => self.reg_clear_color = (value & 0x03) as _,
            MappedAddress::Vram(addr) => {
                self.vram[addr as usize] = value as u8;
                self.vram[addr as usize + 1] = (value >> 8) as u8;
            }
            MappedAddress::Unrecognized(addr) => {
                println!("WARNING: Attempted write halfword to unrecognized VIP address (addr: 0x{:08x}, value: 0x{:04x})", addr, value);
            }
        }
    }

    fn read_vram_halfword(&self, addr: u32) -> u16 {
        (self.vram[addr as usize] as u16) |
        ((self.vram[addr as usize + 1] as u16) << 8)
    }

    pub fn cycles(&mut self, cycles: usize, video_driver: &mut VideoDriver) -> bool {
        let mut raise_interrupt = false;

        for _ in 0..cycles {
            self.frame_clock_counter += CPU_CYCLE_PERIOD_NS;
            if self.frame_clock_counter >= FRAME_CLOCK_PERIOD_NS {
                self.frame_clock_counter -= FRAME_CLOCK_PERIOD_NS;
                self.frame_clock(&mut raise_interrupt);
            }

            if let DrawingState::Drawing = self.drawing_state {
                self.drawing_counter += CPU_CYCLE_PERIOD_NS;
                if self.drawing_counter >= DRAWING_PERIOD_NS {
                    self.end_drawing_process();
                    self.reg_interrupt_pending_drawing_finished = true;
                    if self.reg_interrupt_enable_drawing_finished {
                        raise_interrupt = true;
                    }
                }
            }

            self.display_counter += CPU_CYCLE_PERIOD_NS;
            match self.display_state {
                DisplayState::Idle => {
                    if self.display_counter >= DISPLAY_PROCESSING_DELAY_PERIOD_NS {
                        self.display_counter -= DISPLAY_PROCESSING_DELAY_PERIOD_NS;
                        self.start_left_framebuffer_display_process();
                    }
                }
                DisplayState::LeftFramebuffer => {
                    if self.display_counter >= DISPLAY_PROCESSING_BUFFER_PERIOD_NS {
                        self.display_counter -= DISPLAY_PROCESSING_BUFFER_PERIOD_NS;
                        self.start_right_framebuffer_display_process();
                        self.reg_interrupt_pending_left_display_finished = true;
                        if self.reg_interrupt_enable_left_display_finished {
                            raise_interrupt = true;
                        }
                    }
                }
                DisplayState::RightFramebuffer => {
                    if self.display_counter >= DISPLAY_PROCESSING_BUFFER_PERIOD_NS {
                        self.display_counter -= DISPLAY_PROCESSING_BUFFER_PERIOD_NS;
                        self.end_display_processing(video_driver);
                        self.reg_interrupt_pending_right_display_finished = true;
                        if self.reg_interrupt_enable_right_display_finished {
                            raise_interrupt = true;
                        }
                    }
                }
            }
        }

        raise_interrupt
    }

    fn frame_clock(&mut self, raise_interrupt: &mut bool) {
        println!("Frame clock rising edge");

        self.reg_interrupt_pending_start_of_frame_processing = true;
        if self.reg_interrupt_enable_start_of_frame_processing {
            *raise_interrupt = true;
        }

        self.game_frame_clock_counter += 1;
        if self.game_frame_clock_counter >= self.reg_game_frame_control {
            self.game_frame_clock_counter = 0;
            self.game_clock(raise_interrupt);
        }
    }

    fn game_clock(&mut self, raise_interrupt: &mut bool) {
        println!("Game clock rising edge");

        self.reg_interrupt_pending_drawing_started = true;
        if self.reg_interrupt_enable_drawing_started {
            *raise_interrupt = true;
        }

        if self.reg_drawing_control_drawing_enable {
            self.display_first_framebuffers = !self.display_first_framebuffers;

            self.begin_drawing_process();
        } else {
            self.reg_interrupt_pending_drawing_finished = true;
            if self.reg_interrupt_enable_drawing_finished {
                *raise_interrupt = true;
            }
        }
    }

    fn begin_drawing_process(&mut self) {
        println!("Begin drawing process");
        self.drawing_state = DrawingState::Drawing;
        self.drawing_counter = 0;
    }

    fn end_drawing_process(&mut self) {
        self.draw();

        println!("End drawing process");
        self.drawing_state = DrawingState::Idle;
    }

    fn start_left_framebuffer_display_process(&mut self) {
        println!("Start left framebuffer display process");
        self.display_state = DisplayState::LeftFramebuffer;
    }

    fn start_right_framebuffer_display_process(&mut self) {
        println!("Start right framebuffer display process");
        self.display_state = DisplayState::RightFramebuffer;
    }

    fn end_display_processing(&mut self, video_driver: &mut VideoDriver) {
        self.display(video_driver);

        println!("End display process");
        self.display_state = DisplayState::Idle;
    }

    fn draw(&mut self) {
        let draw_to_first_framebuffers = !self.display_first_framebuffers;
        let left_framebuffer_offset = if draw_to_first_framebuffers { 0x00000000 } else { 0x00008000 };
        let right_framebuffer_offset = left_framebuffer_offset + 0x00010000;

        let clear_pixels = (self.reg_clear_color << 6) | (self.reg_clear_color << 4) | (self.reg_clear_color << 2) | self.reg_clear_color;
        for i in 0..FRAMEBUFFER_RESOLUTION_X * FRAMEBUFFER_RESOLUTION_Y / 4 {
            self.vram[left_framebuffer_offset + i] = clear_pixels;
            self.vram[right_framebuffer_offset + i] = clear_pixels;
        }

        let mut current_obj_group = Some(ObjGroup::Group3);

        const WINDOW_ENTRY_LENGTH: u32 = 32;
        let mut window_offset = WINDOW_ATTRIBS_END + 1 - WINDOW_ENTRY_LENGTH;
        let mut window_index = 31;
        for _ in 0..32 {
            println!("Window {}", window_index);

            let header = self.read_vram_halfword(window_offset);
            let base = (header & 0x000f) as u32;
            let stop = (header & 0x0040) != 0;
            let out_of_bounds = (header & 0x0080) != 0;
            let bg_height = ((header >> 8) & 0x03) as usize;
            let bg_width = ((header >> 10) & 0x03) as usize;
            let mode = ((header >> 12) & 0x03) as usize;
            let right_on = (header & 0x4000) != 0;
            let left_on = (header & 0x8000) != 0;
            println!(" Header: 0x{:04x}", header);
            println!("  base: 0x{:02x}", base);
            println!("  stop: {}", stop);
            println!("  out of bounds: {}", out_of_bounds);
            println!("  w, h: {}, {}", bg_width, bg_height);
            println!("  mode: {}", mode);
            println!("  l, r: {}, {}", left_on, right_on);

            let x = self.read_vram_halfword(window_offset + 2) as i16;
            let parallax = self.read_vram_halfword(window_offset + 4) as i16;
            let y = self.read_vram_halfword(window_offset + 6) as i16;
            let bg_x = self.read_vram_halfword(window_offset + 8) as i16;
            let bg_parallax = self.read_vram_halfword(window_offset + 10) as i16;
            let bg_y = self.read_vram_halfword(window_offset + 12) as i16;
            let width = self.read_vram_halfword(window_offset + 14);
            let height = self.read_vram_halfword(window_offset + 16);
            let param_base = self.read_vram_halfword(window_offset + 18) as u32;
            let out_of_bounds_char = self.read_vram_halfword(window_offset + 20);
            println!(" X: {}", x);
            println!(" Parallax: {}", parallax);
            println!(" Y: {}", y);
            println!(" BG X: {}", bg_x);
            println!(" BG Parallax: {}", bg_parallax);
            println!(" BG Y: {}", bg_y);
            println!(" Width: {}", width);
            println!(" Height: {}", height);
            println!(" Param base: 0x{:04x}", param_base);
            println!(" Out of bounds char: 0x{:04x}", out_of_bounds_char);

            if stop {
                break;
            }

            let width = (width as u32) + 1;
            let height = (height as u32) + 1;
            let segment_offset = 0x00020000 + base * 0x00002000;
            let param_offset = 0x00020000 + param_base * 2;

            let mode = match mode {
                0 => WindowMode::Normal,
                1 => WindowMode::LineShift,
                2 => WindowMode::Affine,
                _ => WindowMode::Obj
            };

            for i in 0..2 {
                let eye = match i {
                    0 => Eye::Left,
                    _ => Eye::Right,
                };

                match eye {
                    Eye::Left => {
                        if !left_on {
                            continue;
                        }
                    }
                    Eye::Right => {
                        if !right_on {
                            continue;
                        }
                    }
                }

                let framebuffer_offset = match eye {
                    Eye::Left => left_framebuffer_offset,
                    Eye::Right => right_framebuffer_offset,
                };

                match mode {
                    WindowMode::Obj => {
                        println!("Current obj group: {:?}", current_obj_group);

                        match current_obj_group {
                            Some(obj_group) => {
                                let starting_obj_index = match obj_group {
                                    ObjGroup::Group0 => self.reg_obj_group_0_ptr,
                                    ObjGroup::Group1 => self.reg_obj_group_1_ptr,
                                    ObjGroup::Group2 => self.reg_obj_group_2_ptr,
                                    ObjGroup::Group3 => self.reg_obj_group_3_ptr,
                                };
                                let mut ending_obj_index = match obj_group {
                                    ObjGroup::Group0 => 0,
                                    ObjGroup::Group1 => self.reg_obj_group_0_ptr + 1,
                                    ObjGroup::Group2 => self.reg_obj_group_1_ptr + 1,
                                    ObjGroup::Group3 => self.reg_obj_group_2_ptr + 1,
                                };
                                if ending_obj_index >= starting_obj_index {
                                    ending_obj_index = 0;
                                }
                                for i in (ending_obj_index..starting_obj_index + 1).rev() {
                                    //println!("Current obj: {}", i);

                                    let obj_offset = 0x0003e000 + (i as u32) * 8;

                                    let x = self.read_vram_halfword(obj_offset) as i16;
                                    let l_r_parallax = self.read_vram_halfword(obj_offset + 2);
                                    let l = (l_r_parallax & 0x8000) != 0;
                                    let r = (l_r_parallax & 0x4000) != 0;
                                    let parallax = ((l_r_parallax << 2) as i16) >> 2;
                                    let y = self.read_vram_halfword(obj_offset + 4) as i16;
                                    let pal_hf_vf_char = self.read_vram_halfword(obj_offset + 6);
                                    let pal = pal_hf_vf_char >> 14;
                                    let horizontal_flip = (pal_hf_vf_char & 0x2000) != 0;
                                    let vertical_flip = (pal_hf_vf_char & 0x1000) != 0;
                                    let char_index = (pal_hf_vf_char & 0x07ff) as u32;
                                    /*println!(" X: {}", x);
                                    println!(" L: {}", l);
                                    println!(" R: {}", r);
                                    println!(" Parallax: {}", parallax);
                                    println!(" Y: {}", y);
                                    println!(" Pal: {}", pal);
                                    println!(" Horizontal flip: {}", horizontal_flip);
                                    println!(" Vertical flip: {}", vertical_flip);
                                    println!(" Char index: {}", char_index);*/

                                    match eye {
                                        Eye::Left => {
                                            if !l {
                                                continue;
                                            }
                                        }
                                        Eye::Right => {
                                            if !r {
                                                continue;
                                            }
                                        }
                                    }

                                    let palette = match pal {
                                        0 => self.reg_obj_palette_0,
                                        1 => self.reg_obj_palette_1,
                                        2 => self.reg_obj_palette_2,
                                        _ => self.reg_obj_palette_3
                                    };

                                    for offset_y in 0..8 {
                                        let pixel_y = (y as u32).wrapping_add(offset_y);
                                        if pixel_y >= FRAMEBUFFER_RESOLUTION_Y as u32 {
                                            continue;
                                        }
                                        for offset_x in 0..8 {
                                            let pixel_x = {
                                                let value = (x as u32).wrapping_add(offset_x);
                                                match eye {
                                                    Eye::Left => value.wrapping_sub(parallax as u32),
                                                    Eye::Right => value.wrapping_add(parallax as u32),
                                                }
                                            };
                                            if pixel_x >= FRAMEBUFFER_RESOLUTION_X as u32 {
                                                continue;
                                            }

                                            self.draw_char_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_index, horizontal_flip, vertical_flip, palette);
                                        }
                                    }
                                }
                            }
                            _ => println!("WARNING: Extra obj window found; all obj groups already drawn")
                        }
                    }
                    WindowMode::Affine => {
                        for pixel_y in 0..FRAMEBUFFER_RESOLUTION_Y as u32 {
                            for pixel_x in 0..FRAMEBUFFER_RESOLUTION_X as u32 {
                                let x = {
                                    let value = x as u32;
                                    match eye {
                                        Eye::Left => value.wrapping_sub(parallax as u32),
                                        Eye::Right => value.wrapping_add(parallax as u32),
                                    }
                                };

                                let window_x = pixel_x.wrapping_sub(x as u32);
                                let window_y = pixel_y.wrapping_sub(y as u32);

                                if window_x >= width || window_y >= height {
                                    continue;
                                }

                                let affine_offset = param_offset + window_y * 16;
                                let affine_bg_x = self.read_vram_halfword(affine_offset) as i16;
                                let affine_bg_parallax = self.read_vram_halfword(affine_offset + 2) as i16; // TODO
                                let affine_bg_y = self.read_vram_halfword(affine_offset + 4) as i16;
                                let affine_bg_x_inc = self.read_vram_halfword(affine_offset + 6) as i16;
                                let affine_bg_y_inc = self.read_vram_halfword(affine_offset + 8) as i16;
                                let parallaxed_window_x = match eye {
                                    Eye::Left => {
                                        if affine_bg_parallax < 0 {
                                            window_x.wrapping_sub(affine_bg_parallax as u32)
                                        } else {
                                            window_x
                                        }
                                    }
                                    Eye::Right => {
                                        if affine_bg_parallax > 0 {
                                            window_x.wrapping_add(affine_bg_parallax as u32)
                                        } else {
                                            window_x
                                        }
                                    }
                                };
                                let background_x = (((affine_bg_x as i32) << 6) + ((affine_bg_x_inc as i32) * (parallaxed_window_x as i32)) >> 9) as u32;
                                let background_y = (((affine_bg_y as i32) << 6) + ((affine_bg_y_inc as i32) * (parallaxed_window_x as i32)) >> 9) as u32;

                                let segment_x = (background_x >> 3) & 0x3f;
                                let segment_y = (background_y >> 3) & 0x3f;
                                let offset_x = background_x & 0x07;
                                let offset_y = background_y & 0x07;
                                let segment_addr = segment_offset + (segment_y * 64 + segment_x) * 2;
                                let entry = self.read_vram_halfword(segment_addr as _);
                                let pal = (entry >> 14) & 0x03;
                                let horizontal_flip = (entry & 0x2000) != 0;
                                let vertical_flip = (entry & 0x1000) != 0;
                                let char_index = (entry & 0x07ff) as u32;

                                let palette = match pal {
                                    0 => self.reg_bg_palette_0,
                                    1 => self.reg_bg_palette_1,
                                    2 => self.reg_bg_palette_2,
                                    _ => self.reg_bg_palette_3
                                };

                                self.draw_char_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_index, horizontal_flip, vertical_flip, palette);
                            }
                        }
                    }
                    _ => {
                        for pixel_y in 0..FRAMEBUFFER_RESOLUTION_Y as u32 {
                            for pixel_x in 0..FRAMEBUFFER_RESOLUTION_X as u32 {
                                let x = {
                                    let value = x as u32;
                                    match eye {
                                        Eye::Left => value.wrapping_sub(parallax as u32),
                                        Eye::Right => value.wrapping_add(parallax as u32),
                                    }
                                };

                                let window_x = pixel_x.wrapping_sub(x as u32);
                                let window_y = pixel_y.wrapping_sub(y as u32);

                                if window_x >= width || window_y >= height {
                                    continue;
                                }

                                let line_shift = match mode {
                                    WindowMode::LineShift => {
                                        let line_offset = param_offset + window_y * 4;
                                        let eye_offset = line_offset + match eye {
                                            Eye::Left => 0,
                                            Eye::Right => 2,
                                        };
                                        (self.read_vram_halfword(eye_offset) as i16) as u32
                                    }
                                    _ => 0
                                };

                                let background_x = {
                                    let value = window_x.wrapping_add(bg_x as u32).wrapping_add(line_shift);
                                    match eye {
                                        Eye::Left => value.wrapping_sub(bg_parallax as u32),
                                        Eye::Right => value.wrapping_add(bg_parallax as u32),
                                    }
                                };
                                let background_y = window_y.wrapping_add(bg_y as u32);

                                let segment_x = (background_x >> 3) & 0x3f;
                                let segment_y = (background_y >> 3) & 0x3f;
                                let offset_x = background_x & 0x07;
                                let offset_y = background_y & 0x07;
                                let segment_addr = segment_offset + (segment_y * 64 + segment_x) * 2;
                                let entry = self.read_vram_halfword(segment_addr as _);
                                let pal = (entry >> 14) & 0x03;
                                let horizontal_flip = (entry & 0x2000) != 0;
                                let vertical_flip = (entry & 0x1000) != 0;
                                let char_index = (entry & 0x07ff) as u32;

                                let palette = match pal {
                                    0 => self.reg_bg_palette_0,
                                    1 => self.reg_bg_palette_1,
                                    2 => self.reg_bg_palette_2,
                                    _ => self.reg_bg_palette_3
                                };

                                self.draw_char_pixel(framebuffer_offset, pixel_x, pixel_y, offset_x, offset_y, char_index, horizontal_flip, vertical_flip, palette);
                            }
                        }
                    }
                }
            }

            if let WindowMode::Obj = mode {
                current_obj_group = match current_obj_group {
                    Some(ObjGroup::Group3) => Some(ObjGroup::Group2),
                    Some(ObjGroup::Group2) => Some(ObjGroup::Group1),
                    Some(ObjGroup::Group1) => Some(ObjGroup::Group0),
                    _ => None
                };
            }

            window_offset -= WINDOW_ENTRY_LENGTH;
            window_index -= 1;
        }
    }

    fn draw_char_pixel(&mut self, framebuffer_offset: usize, pixel_x: u32, pixel_y: u32, offset_x: u32, offset_y: u32, char_index: u32, horizontal_flip: bool, vertical_flip: bool, palette: u8) {
        let offset_x = if horizontal_flip { 7 - offset_x } else { offset_x };
        let offset_y = if vertical_flip { 7 - offset_y } else { offset_y };

        let char_offset = if char_index < 0x0200 {
            0x00006000 + char_index * 16
        } else if char_index < 0x0400 {
            0x0000e000 + (char_index - 0x0200) * 16
        } else if char_index < 0x0600 {
            0x00016000 + (char_index - 0x0400) * 16
        } else {
            0x0001e000 + (char_index - 0x0600) * 16
        };

        let char_row_offset = char_offset + offset_y * 2;
        let char_row_data = self.read_vram_halfword(char_row_offset as _);
        let palette_index = ((char_row_data as u32) >> (offset_x * 2)) & 0x03;

        if palette_index == 0 {
            return;
        }

        let color = (palette >> (palette_index * 2)) & 0x03;

        let framebuffer_byte_index = ((pixel_x as usize) * FRAMEBUFFER_RESOLUTION_Y + (pixel_y as usize)) / 4;
        let framebuffer_byte_shift = (pixel_y & 0x03) * 2;
        let framebuffer_byte_mask = 0x03 << framebuffer_byte_shift;
        let mut framebuffer_byte = self.vram[framebuffer_offset + framebuffer_byte_index];
        framebuffer_byte = (framebuffer_byte & !framebuffer_byte_mask) | (color << framebuffer_byte_shift);
        self.vram[framebuffer_offset + framebuffer_byte_index] = framebuffer_byte;
    }

    fn display(&self, video_driver: &mut VideoDriver) {
        let left_framebuffer_offset = if self.display_first_framebuffers { 0x00000000 } else { 0x00008000 };
        let right_framebuffer_offset = left_framebuffer_offset + 0x00010000;

        let mut brightness_1 = (self.reg_led_brightness_1 as u32) * 2;
        let mut brightness_2 = (self.reg_led_brightness_2 as u32) * 2;
        let mut brightness_3 = ((self.reg_led_brightness_1 as u32) + (self.reg_led_brightness_2 as u32) + (self.reg_led_brightness_3 as u32)) * 2;
        if brightness_1 > 255 {
            brightness_1 = 255;
        }
        if brightness_2 > 255 {
            brightness_2 = 255;
        }
        if brightness_3 > 255 {
            brightness_3 = 255;
        }

        let mut left_buffer = vec![0; DISPLAY_RESOLUTION_X * DISPLAY_RESOLUTION_Y];
        let mut right_buffer = vec![0; DISPLAY_RESOLUTION_X * DISPLAY_RESOLUTION_Y];
        for pixel_x in 0..DISPLAY_RESOLUTION_X as usize {
            for pixel_y in 0..DISPLAY_RESOLUTION_Y as usize {
                let framebuffer_byte_index = (pixel_x * FRAMEBUFFER_RESOLUTION_Y + pixel_y) / 4;
                let framebuffer_byte_shift = (pixel_y & 0x03) * 2;
                let left_color = (self.vram[left_framebuffer_offset + framebuffer_byte_index] >> framebuffer_byte_shift) & 0x03;
                let right_color = (self.vram[right_framebuffer_offset + framebuffer_byte_index] >> framebuffer_byte_shift) & 0x03;
                let left_brightness = match left_color {
                    0 => 0,
                    1 => brightness_1,
                    2 => brightness_2,
                    _ => brightness_3
                } as u8;
                let right_brightness = match right_color {
                    0 => 0,
                    1 => brightness_1,
                    2 => brightness_2,
                    _ => brightness_3
                } as u8;
                let buffer_index = pixel_y * DISPLAY_RESOLUTION_X + pixel_x;
                left_buffer[buffer_index] = left_brightness;
                right_buffer[buffer_index] = right_brightness;
            }
        }

        video_driver.output_frame((left_buffer.into_boxed_slice(), right_buffer.into_boxed_slice()));
    }
}
