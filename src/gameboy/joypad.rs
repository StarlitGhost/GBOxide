use num_traits::FromPrimitive;

use crate::gameboy::interrupt::{Interrupt, InterruptHandler};

bitfield!{
    struct Buttons(u8);
    impl Debug;
    start, _: 3;
    select, _: 2;
    b, _: 1;
    a, _: 0;
    from into u8, bits, set_bits: 3,0;
}
impl From<u8> for Buttons {
    fn from(value: u8) -> Buttons {
        Buttons(value & 0b0000_1111)
    }
}
impl From<Buttons> for u8 {
    fn from(value: Buttons) -> u8 {
        value.bits() & 0b0000_1111
    }
}

bitfield!{
    struct Directions(u8);
    impl Debug;
    down, _: 3;
    up, _: 2;
    left, _: 1;
    right, _: 0;
    from into u8, bits, set_bits: 3,0;
}
impl From<u8> for Directions {
    fn from(value: u8) -> Directions {
        Directions(value & 0b0000_1111)
    }
}
impl From<Directions> for u8 {
    fn from(value: Directions) -> u8 {
        value.bits() & 0b0000_1111
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
enum JoypadSelection {
    Buttons = 0b0010_0000,
    Directions = 0b0001_0000,
    Both = 0b0011_0000,
    Neither = 0b0000_0000,
}

pub struct Joypad {
    buttons: Buttons,
    directions: Directions,

    selection: JoypadSelection,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            buttons: Buttons(0b0000),
            directions: Directions(0b0000),
            selection: JoypadSelection::Neither,
        }
    }

    pub fn set_from_controls(&mut self, controls: Controls, ih: &mut InterruptHandler) {
        // store previous values so we can check if we need to set an interrupt
        let prev_buttons = self.buttons.bits();
        let prev_directions = self.directions.bits();

        let buttons = (controls.a as u8)
                    | (controls.b as u8) << 1
                    | (controls.select as u8) << 2
                    | (controls.start as u8) << 3;
        self.buttons.set_bits(buttons);

        let directions = (controls.right as u8)
                       | (controls.left as u8) << 1
                       | (controls.up as u8) << 2
                       | (controls.down as u8) << 3;
        self.directions.set_bits(directions);

        // check if any bits went from 0 to 1 in the set of buttons the current selection points to
        // (1 to 0 in actual hardware - we invert the values when reading/writing)
        use JoypadSelection::*;
        let new_pressed = match self.selection {
            Buttons => self.buttons.bits() & !prev_buttons > 0,
            Directions => self.directions.bits() & !prev_directions > 0,
            Both => (self.buttons.bits() | self.directions.bits()) & !(prev_buttons | prev_directions) > 0,
            Neither => false,
        };
        if new_pressed {
            ih.set_interrupt(Interrupt::Joypad);
        }
    }

    pub fn write_select_bits(&mut self, value: u8) {
        // only the selection bits can be written to, so mask the input to them
        // we also invert the input value since in actual hardware, 0 is selected and 1 is not
        let bits = !value & 0b0011_0000;
        self.selection = FromPrimitive::from_u8(bits).expect("invalid selection bits");
    }

    pub fn as_u8(&self) -> u8 {
        use JoypadSelection::*;
        // invert the whole u8 since select/pressed are denoted by 0, but we're storing as 1
        !(0b0000_0000 |
        // OR in our selection bits
        self.selection as u8 |
        // OR in the set of control bits indicated by the selection bits
        match self.selection {
            Buttons => self.buttons.bits(),
            Directions => self.directions.bits(),
            Both => self.buttons.bits() | self.directions.bits(),
            Neither => 0b0000,
        })
    }
}

pub struct Controls {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}