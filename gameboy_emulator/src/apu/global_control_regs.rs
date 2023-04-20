pub struct GlobalControlRegisters {
    // 0xff26 - sound on/off
    // bit 7 is all sound on/off
    // bits 3-0 are for channels 4-1 respectively
    pub nr52: u8,

    // 0xff25 - sound panning
    /*  Bit 7 - Mix channel 4 into left output
        Bit 6 - Mix channel 3 into left output
        Bit 5 - Mix channel 2 into left output
        Bit 4 - Mix channel 1 into left output
        Bit 3 - Mix channel 4 into right output
        Bit 2 - Mix channel 3 into right output
        Bit 1 - Mix channel 2 into right output
        Bit 0 - Mix channel 1 into right output  */
    pub nr51: u8,

    // 0xff24 - master volume & VIN panning
    // For volume bits, a value of 0 is a volume of 1 and a value of 7 is a volume of 8.
    /*  Bit 7   - Mix VIN into left output  (1=Enable)
        Bit 6-4 - Left output volume        (0-7)
        Bit 3   - Mix VIN into right output (1=Enable)
        Bit 2-0 - Right output volume       (0-7)  */
    pub nr50: u8,
}

impl GlobalControlRegisters {
    pub fn sound_on(&self) -> bool {
        (self.nr52 >> 7) & 1 == 1
    }
}