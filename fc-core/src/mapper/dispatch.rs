use super::*;

macro_rules! dispatch {
    ($self:ident, $m:ident => $body:expr) => {
        match $self {
            Mapper::Nrom($m) => $body,
            Mapper::Mmc1($m) => $body,
            Mapper::Unrom($m) => $body,
            Mapper::Cnrom($m) => $body,
            Mapper::Axrom($m) => $body,
            Mapper::FfeMapper($m) => $body,
            Mapper::Mapper8($m) => $body,
            Mapper::Bnrom($m) => $body,
            Mapper::Nina01($m) => $body,
            Mapper::Cprom($m) => $body,
            Mapper::Mapper15($m) => $body,
            Mapper::BandaiFcg($m) => $body,
            Mapper::Mapper18($m) => $body,
            Mapper::Namco163($m) => $body,
            Mapper::Vrc6($m) => $body,
            Mapper::IremG101($m) => $body,
            Mapper::Action53($m) => $body,
            Mapper::Mapper29($m) => $body,
            Mapper::Mapper31($m) => $body,
            Mapper::TaitoTc0190($m) => $body,
            Mapper::Bandai74161($m) => $body,
            Mapper::JalecoJf16($m) => $body,
            Mapper::JalecoJfxx($m) => $body,
            Mapper::Sunsoft184($m) => $body,
            Mapper::UnlPci556($m) => $body,
            Mapper::Caltron41($m) => $body,
            Mapper::ColorDreams46($m) => $body,
            Mapper::Mapper35($m) => $body,
            Mapper::Mapper36($m) => $body,
            Mapper::Mapper40($m) => $body,
            Mapper::Mapper42($m) => $body,
            Mapper::Mapper43($m) => $body,
            Mapper::Mapper50($m) => $body,
            Mapper::Mapper51($m) => $body,
            Mapper::Mapper53($m) => $body,
            Mapper::Mapper57($m) => $body,
            Mapper::Mapper60($m) => $body,
            Mapper::Mapper63($m) => $body,
            Mapper::Rambo1($m) => $body,
            Mapper::Mapper65($m) => $body,
            Mapper::Mapper67($m) => $body,
            Mapper::Sunsoft4($m) => $body,
            Mapper::Mapper72($m) => $body,
            Mapper::Mapper73($m) => $body,
            Mapper::Mapper79($m) => $body,
            Mapper::Mapper81($m) => $body,
            Mapper::TaitoX1005($m) => $body,
            Mapper::TaitoX1017($m) => $body,
            Mapper::Vrc1($m) => $body,
            Mapper::Mapper83($m) => $body,
            Mapper::JyAsic($m) => $body,
            Mapper::Mapper91($m) => $body,
            Mapper::Mapper92($m) => $body,
            Mapper::Mapper96($m) => $body,
            Mapper::Mapper99($m) => $body,
            Mapper::AddrLatch16k($m) => $body,
            Mapper::Mapper103($m) => $body,
            Mapper::Mapper104($m) => $body,
            Mapper::Mapper106($m) => $body,
            Mapper::Mapper108($m) => $body,
            Mapper::Mapper111($m) => $body,
            Mapper::Mapper116($m) => $body,
            Mapper::Mapper117($m) => $body,
            Mapper::Mapper120($m) => $body,
            Mapper::Mapper122($m) => $body,
            Mapper::Mapper128($m) => $body,
            Mapper::TxcMapper($m) => $body,
            Mapper::Sachen133($m) => $body,
            Mapper::SachenSa0161m($m) => $body,
            Mapper::Sachen149($m) => $body,
            Mapper::Sachen8259($m) => $body,
            Mapper::Sachen74Ls374N($m) => $body,
            Mapper::Mapper142($m) => $body,
            Mapper::Mapper156($m) => $body,
            Mapper::NanjingMapper($m) => $body,
            Mapper::Subor166($m) => $body,
            Mapper::Mapper168($m) => $body,
            Mapper::Mapper170($m) => $body,
            Mapper::Mapper171($m) => $body,
            Mapper::Mapper175($m) => $body,
            Mapper::Mapper177($m) => $body,
            Mapper::Mapper178($m) => $body,
            Mapper::Mapper181($m) => $body,
            Mapper::Mapper183($m) => $body,
            Mapper::Mapper185($m) => $body,
            Mapper::Mapper186($m) => $body,
            Mapper::Mapper188($m) => $body,
            Mapper::Mapper190($m) => $body,
            Mapper::Mapper193($m) => $body,
            Mapper::Mapper212($m) => $body,
            Mapper::Mapper218($m) => $body,
            Mapper::Mapper222($m) => $body,
            Mapper::Mapper226($m) => $body,
            Mapper::Mapper230($m) => $body,
            Mapper::Mapper233($m) => $body,
            Mapper::Mapper234($m) => $body,
            Mapper::Mapper235($m) => $body,
            Mapper::Mapper236($m) => $body,
            Mapper::Mapper237($m) => $body,
            Mapper::Mapper240($m) => $body,
            Mapper::Mapper241($m) => $body,
            Mapper::Mapper244($m) => $body,
            Mapper::Mapper246($m) => $body,
            Mapper::Mapper252($m) => $body,
            Mapper::Mapper253($m) => $body,
            Mapper::IremLrog017($m) => $body,
            Mapper::Namco108Mapper154($m) => $body,
            Mapper::Namco108Mapper95($m) => $body,
            Mapper::Namco108Mapper206($m) => $body,
            Mapper::Namco118($m) => $body,
            Mapper::ActionEnterprises($m) => $body,
            Mapper::Bf9096($m) => $body,
            Mapper::JalecoJf13($m) => $body,
            Mapper::Sunsoft89($m) => $body,
            Mapper::UnromVariant($m) => $body,
            Mapper::IremTamS1($m) => $body,
            Mapper::Mapper107($m) => $body,
            Mapper::Ntdec112($m) => $body,
            Mapper::Nina03_06($m) => $body,
            Mapper::JalecoJf11_14($m) => $body,
            Mapper::Mapper151($m) => $body,
            Mapper::Mapper203($m) => $body,
            Mapper::Mmc3($m) => $body,
            Mapper::Mmc5($m) => $body,
            Mapper::Mmc2($m) => $body,
            Mapper::Mmc4($m) => $body,
            Mapper::ColorDreams($m) => $body,
            Mapper::Gxrom($m) => $body,
            Mapper::Fme7($m) => $body,
            Mapper::Codemasters($m) => $body,
            Mapper::Vrc7($m) => $body,
            Mapper::Vrc4($m) => $body,
        }
    };
}

impl MapperOps for Mapper {
    fn prg_index(&self, addr: u16) -> usize {
        dispatch!(self, m => m.prg_index(addr))
    }
    fn chr_index(&self, addr: u16) -> usize {
        dispatch!(self, m => m.chr_index(addr))
    }
    fn chr_index_for(&self, addr: u16, access: ChrAccess) -> usize {
        dispatch!(self, m => m.chr_index_for(addr, access))
    }
    fn chr_read(&self, addr: u16, access: ChrAccess) -> Option<u8> {
        dispatch!(self, m => m.chr_read(addr, access))
    }
    fn has_chr_read(&self) -> bool {
        dispatch!(self, m => m.has_chr_read())
    }
    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        dispatch!(self, m => m.chr_write(addr, value))
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_register(addr, value))
    }
    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        dispatch!(self, m => m.read_register(addr, prg_value))
    }
    fn read_register_with_open_bus(
        &mut self,
        addr: u16,
        prg_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        dispatch!(self, m => m.read_register_with_open_bus(addr, prg_value, open_bus))
    }
    fn peek_register(&self, addr: u16, prg_value: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_register(addr, prg_value))
    }
    fn peek_register_with_open_bus(&self, addr: u16, prg_value: u8, open_bus: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_register_with_open_bus(addr, prg_value, open_bus))
    }
    fn has_bus_conflicts(&self) -> bool {
        dispatch!(self, m => m.has_bus_conflicts())
    }
    fn apply_bus_conflict(&self, value: u8, prg_value: u8) -> u8 {
        dispatch!(self, m => m.apply_bus_conflict(value, prg_value))
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        dispatch!(self, m => m.write_low_register(addr, value))
    }
    fn low_register_write_falls_through(&self, addr: u16) -> bool {
        dispatch!(self, m => m.low_register_write_falls_through(addr))
    }
    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        dispatch!(self, m => m.low_prg_index(addr))
    }
    fn low_prg_ram_read_enabled(&self, addr: u16) -> bool {
        dispatch!(self, m => m.low_prg_ram_read_enabled(addr))
    }
    fn low_prg_ram_write_enabled(&self, addr: u16) -> bool {
        dispatch!(self, m => m.low_prg_ram_write_enabled(addr))
    }
    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.read_low_register(addr))
    }
    fn read_low_register_with_prg_ram(&mut self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        dispatch!(self, m => m.read_low_register_with_prg_ram(addr, prg_ram_value))
    }
    fn read_low_register_with_open_bus(
        &mut self,
        addr: u16,
        prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        dispatch!(self, m => m.read_low_register_with_open_bus(addr, prg_ram_value, open_bus))
    }
    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.peek_low_register(addr))
    }
    fn peek_low_register_with_prg_ram(&self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_low_register_with_prg_ram(addr, prg_ram_value))
    }
    fn peek_low_register_with_open_bus(
        &self,
        addr: u16,
        prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        dispatch!(self, m => m.peek_low_register_with_open_bus(addr, prg_ram_value, open_bus))
    }
    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.read_expansion(addr))
    }
    fn read_expansion_with_open_bus(&mut self, addr: u16, open_bus: u8) -> Option<u8> {
        dispatch!(self, m => m.read_expansion_with_open_bus(addr, open_bus))
    }
    fn expansion_prg_index(&self, addr: u16) -> Option<usize> {
        dispatch!(self, m => m.expansion_prg_index(addr))
    }
    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.peek_expansion(addr))
    }
    fn peek_expansion_with_open_bus(&self, addr: u16, open_bus: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_expansion_with_open_bus(addr, open_bus))
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_expansion(addr, value))
    }
    fn write_controller_strobe(&mut self, value: u8) -> bool {
        dispatch!(self, m => m.write_controller_strobe(value))
    }
    fn notify_ppudata_write(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.notify_ppudata_write(addr, value))
    }
    fn supports_barcode_input(&self) -> bool {
        dispatch!(self, m => m.supports_barcode_input())
    }
    fn input_barcode(&mut self, digits: &str) -> Result<(), String> {
        dispatch!(self, m => m.input_barcode(digits))
    }
    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        dispatch!(self, m => m.nametable_read(addr, ciram))
    }
    fn nametable_chr_index(&self, addr: u16) -> Option<usize> {
        dispatch!(self, m => m.nametable_chr_index(addr))
    }
    fn has_nametable_chr_mapping(&self) -> bool {
        dispatch!(self, m => m.has_nametable_chr_mapping())
    }
    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        dispatch!(self, m => m.peek_nametable(addr, ciram))
    }
    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        dispatch!(self, m => m.nametable_write(addr, value, ciram))
    }
    fn mirroring(&self) -> Mirroring {
        dispatch!(self, m => m.mirroring())
    }
    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        dispatch!(self, m => m.notify_a12(addr, cycle))
    }
    fn watches_ppu_bus(&self) -> bool {
        dispatch!(self, m => m.watches_ppu_bus())
    }
    fn cpu_clock(&mut self) {
        dispatch!(self, m => m.cpu_clock())
    }
    fn expansion_audio(&self) -> f32 {
        dispatch!(self, m => m.expansion_audio())
    }
    fn has_expansion_audio(&self) -> bool {
        dispatch!(self, m => m.has_expansion_audio())
    }
    fn hblank_clock(&mut self, scanline: u16, dot: u16) {
        dispatch!(self, m => m.hblank_clock(scanline, dot))
    }
    fn clocks_hblank(&self) -> bool {
        dispatch!(self, m => m.clocks_hblank())
    }
    fn clocks_cpu(&self) -> bool {
        dispatch!(self, m => m.clocks_cpu())
    }
    fn irq(&self) -> bool {
        dispatch!(self, m => m.irq())
    }
    fn clear_irq(&mut self) {
        dispatch!(self, m => m.clear_irq())
    }
    fn reset(&mut self, soft: bool) {
        dispatch!(self, m => m.reset(soft))
    }
}
