#![macro_use]
use super::{Core, Cycles, Result};
use super::Exception::IllegalInstruction;
mod common;
pub mod handlers;

pub mod fake {
	use super::super::{Core, Cycles, Result};

	pub fn set_d0(core: &mut Core) -> Result<Cycles> {
		core.dar[0] = 0xabcd;
		Ok(Cycles(2))
	}

	pub fn set_d1(core: &mut Core) -> Result<Cycles> {
		core.dar[1] = 0xbcde;
		Ok(Cycles(2))
	}

	pub fn set_dx(core: &mut Core) -> Result<Cycles> {
		dx!(core) = 0xcdef;
		Ok(Cycles(2))
	}

	use super::super::InstructionSet;
	use super::illegal;
	const SET_DX_0: usize = 0b0100_0000_0000_0000;

	pub fn instruction_set() -> InstructionSet {
		// Covers all possible IR values (64k entries)
		let mut handler: InstructionSet = Vec::with_capacity(0x10000);
		for _ in 0..0x10000 { handler.push(illegal); }
		handler[0xA] = set_d0;
		handler[0xB] = set_d1;
		for i in 0..8 {
			let opcode = SET_DX_0 | (i << 9);
			// println!("{:x}", opcode);
			handler[opcode] = set_dx;
		}
		handler
	}
}

macro_rules! impl_op {
	(8, $common:ident, $name:ident, $src:ident, dx, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let dst = dx!(core);
			let res = common::$common(core, dst, src);
			dx!(core) = mask_out_below_8!(dst) | res;
			Ok(Cycles($cycles))
		});
	(8, $common:ident, $name:ident, $src:ident, dy, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let dst = dy!(core);
			let res = common::$common(core, dst, src);
			dy!(core) = mask_out_below_8!(dst) | res;
			Ok(Cycles($cycles))
		});
	(16, $common:ident, $name:ident, $src:ident, dx, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let dst = dx!(core);
			let res = common::$common(core, dst, src);
			dx!(core) = mask_out_below_16!(dst) | res;
			Ok(Cycles($cycles))
		});
	(16, $common:ident, $name:ident, $src:ident, dy, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let dst = dy!(core);
			let res = common::$common(core, dst, src);
			dy!(core) = mask_out_below_16!(dst) | res;
			Ok(Cycles($cycles))
		});
	(32, $common:ident, $name:ident, $src:ident, dx, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let dst = dx!(core);
			let res = common::$common(core, dst, src);
			dx!(core) = res;
			Ok(Cycles($cycles))
		});
	(32, $common:ident, $name:ident, $src:ident, dy, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let dst = dy!(core);
			let res = common::$common(core, dst, src);
			dy!(core) = res;
			Ok(Cycles($cycles))
		});
	(8, $common:ident, $name:ident, $src:ident, $dst:ident, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let (dst, ea) = try!(operator::$dst(core));
			let res = common::$common(core, dst, src);
			core.write_data_byte(ea, mask_out_below_8!(dst) | res);
			Ok(Cycles($cycles))
		});
	(16, $common:ident, $name:ident, $src:ident, $dst:ident, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let (dst, ea) = try!(operator::$dst(core));
			let res = common::$common(core, dst, src);
			core.write_data_word(ea, mask_out_below_16!(dst) | res);
			Ok(Cycles($cycles))
		});
	(32, $common:ident, $name:ident, $src:ident, $dst:ident, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let src = try!(operator::$src(core));
			let (dst, ea) = try!(operator::$dst(core));
			let res = common::$common(core, dst, src);
			core.write_data_long(ea, res);
			Ok(Cycles($cycles))
		})
}
macro_rules! impl_shift_op {
	(8, $common:ident, $name:ident, $shift_src:ident, dy, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let shift = try!(operator::$shift_src(core)) & 0x3f; // mod 64
			let dst = dy!(core);
			let res = common::$common(core, dst, shift);
			dy!(core) = mask_out_below_8!(dst) | res;
			Ok(Cycles($cycles + 2 * shift as i32))
		});
	(16, $common:ident, $name:ident, $shift_src:ident, dy, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let shift = try!(operator::$shift_src(core)) & 0x3f; // mod 64
			let dst = dy!(core);
			let res = common::$common(core, dst, shift);
			dy!(core) = mask_out_below_16!(dst) | res;
			Ok(Cycles($cycles + 2 * shift as i32))
		});
	(32, $common:ident, $name:ident, $shift_src:ident, dy, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			let shift = try!(operator::$shift_src(core)) & 0x3f; // mod 64
			let dst = dy!(core);
			let res = common::$common(core, dst, shift);
			dy!(core) = res;
			Ok(Cycles($cycles + 2 * shift as i32))
		});
}

pub fn illegal(core: &mut Core) -> Result<Cycles> {
	Err(IllegalInstruction(core.ir, core.pc-2))
}
use super::InstructionSet;
pub fn instruction_set() -> InstructionSet {
	handlers::generate()
}
use std::num::Wrapping;
use super::operator;

impl_op!(8, abcd_8, abcd_8_rr, dy, dx, 6);
impl_op!(8, abcd_8, abcd_8_mm, ay_pd_8, ea_ax_pd_8, 18);

macro_rules! add_8_er {
	($name:ident, $src:ident, $cycles:expr) => (impl_op!(8, add_8, $name, $src, dx, $cycles);)
}
macro_rules! add_8_re {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(8, add_8, $name, dx, $dst, $cycles);)
}
macro_rules! add_16_er {
	($name:ident, $src:ident, $cycles:expr) => (impl_op!(16, add_16, $name, $src, dx, $cycles);)
}
macro_rules! add_16_re {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(16, add_16, $name, dx, $dst, $cycles);)
}
macro_rules! add_32_er {
	($name:ident, $src:ident, $cycles:expr) => (impl_op!(32, add_32, $name, $src, dx, $cycles);)
}
macro_rules! add_32_re {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(32, add_32, $name, dx, $dst, $cycles);)
}
add_8_er!(add_8_er_d, dy, 4);
// add_8_er!(..., ay) not present - for word and long only
add_8_er!(add_8_er_ai, ay_ai_8,   8);
add_8_er!(add_8_er_pi, ay_pi_8,   8);
add_8_er!(add_8_er_pd, ay_pd_8,  10);
add_8_er!(add_8_er_di, ay_di_8,  12);
add_8_er!(add_8_er_ix, ay_ix_8,  14);
add_8_er!(add_8_er_aw, aw_8,     12);
add_8_er!(add_8_er_al, al_8,     16);
add_8_er!(add_8_er_pcdi, pcdi_8, 12);
add_8_er!(add_8_er_pcix, pcix_8, 14);
add_8_er!(add_8_er_imm, imm_8,   10);

// add_8_re!(..., dy) not present
// add_8_re!(..., ay) not present
add_8_re!(add_8_re_ai, ea_ay_ai_8,  12);
add_8_re!(add_8_re_pi, ea_ay_pi_8,  12);
add_8_re!(add_8_re_pd, ea_ay_pd_8,  14);
add_8_re!(add_8_re_di, ea_ay_di_8,  16);
add_8_re!(add_8_re_ix, ea_ay_ix_8,  18);
add_8_re!(add_8_re_aw, ea_aw_8,     16);
add_8_re!(add_8_re_al, ea_al_8,     20);
// add_8_re!(..., pcdi) not present
// add_8_re!(..., pcix) not present
// add_8_re!(..., imm) not present

add_16_er!(add_16_er_d, dy,          4);
add_16_er!(add_16_er_a, ay,          4);
add_16_er!(add_16_er_ai, ay_ai_16,   8);
add_16_er!(add_16_er_pi, ay_pi_16,   8);
add_16_er!(add_16_er_pd, ay_pd_16,  10);
add_16_er!(add_16_er_di, ay_di_16,  12);
add_16_er!(add_16_er_ix, ay_ix_16,  14);
add_16_er!(add_16_er_aw, aw_16,     12);
add_16_er!(add_16_er_al, al_16,     16);
add_16_er!(add_16_er_pcdi, pcdi_16, 12);
add_16_er!(add_16_er_pcix, pcix_16, 14);
add_16_er!(add_16_er_imm, imm_16,   10);

// add_16_re!(..., dy) not present
// add_16_re!(..., ay) not present
add_16_re!(add_16_re_ai, ea_ay_ai_16,  12);
add_16_re!(add_16_re_pi, ea_ay_pi_16,  12);
add_16_re!(add_16_re_pd, ea_ay_pd_16,  14);
add_16_re!(add_16_re_di, ea_ay_di_16,  16);
add_16_re!(add_16_re_ix, ea_ay_ix_16,  18);
add_16_re!(add_16_re_aw, ea_aw_16,     16);
add_16_re!(add_16_re_al, ea_al_16,     20);
// add_16_re!(..., pcdi) not present
// add_16_re!(..., pcix) not present
// add_16_re!(..., imm) not present

add_32_er!(add_32_er_d, dy,          6);
add_32_er!(add_32_er_a, ay,          6);
add_32_er!(add_32_er_ai, ay_ai_32,  14);
add_32_er!(add_32_er_pi, ay_pi_32,  14);
add_32_er!(add_32_er_pd, ay_pd_32,  16);
add_32_er!(add_32_er_di, ay_di_32,  18);
add_32_er!(add_32_er_ix, ay_ix_32,  20);
add_32_er!(add_32_er_aw, aw_32,     18);
add_32_er!(add_32_er_al, al_32,     22);
add_32_er!(add_32_er_pcdi, pcdi_32, 18);
add_32_er!(add_32_er_pcix, pcix_32, 20);
add_32_er!(add_32_er_imm, imm_32,   16);

// add_32_re!(..., dy) not present
// add_32_re!(..., ay) not present
add_32_re!(add_32_re_ai, ea_ay_ai_32,  12+8);
add_32_re!(add_32_re_pi, ea_ay_pi_32,  12+8);
add_32_re!(add_32_re_pd, ea_ay_pd_32,  14+8);
add_32_re!(add_32_re_di, ea_ay_di_32,  16+8);
add_32_re!(add_32_re_ix, ea_ay_ix_32,  18+8);
add_32_re!(add_32_re_aw, ea_aw_32,     16+8);
add_32_re!(add_32_re_al, ea_al_32,     20+8);
// add_32_re!(..., pcdi) not present
// add_32_re!(..., pcix) not present
// add_32_re!(..., imm) not present

macro_rules! adda_16 {
	($name:ident, $src:ident, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			// we must load original value from AX before the src
			// as the PI/PD addressing modes will change AX (if AX=AY)
			let dst = try!(operator::ax(core));
			let src = try!(operator::$src(core));
			ax!(core) = (Wrapping(dst) + Wrapping(src as i16 as u32)).0;
			Ok(Cycles($cycles))
		})
}
macro_rules! adda_32 {
	($name:ident, $src:ident, $cycles:expr) => (
		pub fn $name(core: &mut Core) -> Result<Cycles> {
			// we must load original value from AX before the src
			// as the PI/PD addressing modes will change AX (if AX=AY)
			let dst = try!(operator::ax(core));
			let src = try!(operator::$src(core));
			ax!(core) = (Wrapping(dst) + Wrapping(src)).0;
			Ok(Cycles($cycles))
		})
}
adda_16!(adda_16_d, dy,          4+4);
adda_16!(adda_16_a, ay,          4+4);
adda_16!(adda_16_ai, ay_ai_16,   8+4);
adda_16!(adda_16_pi, ay_pi_16,   8+4);
adda_16!(adda_16_pd, ay_pd_16,  10+4);
adda_16!(adda_16_di, ay_di_16,  12+4);
adda_16!(adda_16_ix, ay_ix_16,  14+4);
adda_16!(adda_16_aw, aw_16,     12+4);
adda_16!(adda_16_al, al_16,     16+4);
adda_16!(adda_16_pcdi, pcdi_16, 12+4);
adda_16!(adda_16_pcix, pcix_16, 14+4);
adda_16!(adda_16_imm, imm_16,   10+4);

adda_32!(adda_32_d, dy,          6);
adda_32!(adda_32_a, ay,          6);
adda_32!(adda_32_ai, ay_ai_32,  14);
adda_32!(adda_32_pi, ay_pi_32,  14);
adda_32!(adda_32_pd, ay_pd_32,  16);
adda_32!(adda_32_di, ay_di_32,  18);
adda_32!(adda_32_ix, ay_ix_32,  20);
adda_32!(adda_32_aw, aw_32,     18);
adda_32!(adda_32_al, al_32,     22);
adda_32!(adda_32_pcdi, pcdi_32, 18);
adda_32!(adda_32_pcix, pcix_32, 20);
adda_32!(adda_32_imm, imm_32,   16);

macro_rules! addi_8 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(8, add_8, $name, imm_8, $dst, $cycles);)
}
macro_rules! addi_16 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(16, add_16, $name, imm_16, $dst, $cycles);)
}
macro_rules! addi_32 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(32, add_32, $name, imm_32, $dst, $cycles);)
}
addi_8!(addi_8_d, dy,  8);
// addi_8_re!(..., ay) not present
addi_8!(addi_8_ai, ea_ay_ai_8,  12+4);
addi_8!(addi_8_pi, ea_ay_pi_8,  12+4);
addi_8!(addi_8_pd, ea_ay_pd_8,  12+6);
addi_8!(addi_8_di, ea_ay_di_8,  12+8);
addi_8!(addi_8_ix, ea_ay_ix_8,  12+10);
addi_8!(addi_8_aw, ea_aw_8,     12+8);
addi_8!(addi_8_al, ea_al_8,     12+12);
// addi_8!(..., pcdi) not present
// addi_8!(..., pcix) not present
// addi_8!(..., imm) not present

addi_16!(addi_16_d, dy,  8);
// addi_16_re!(..., ay) not present
addi_16!(addi_16_ai, ea_ay_ai_16,  12+4);
addi_16!(addi_16_pi, ea_ay_pi_16,  12+4);
addi_16!(addi_16_pd, ea_ay_pd_16,  12+6);
addi_16!(addi_16_di, ea_ay_di_16,  12+8);
addi_16!(addi_16_ix, ea_ay_ix_16,  12+10);
addi_16!(addi_16_aw, ea_aw_16,     12+8);
addi_16!(addi_16_al, ea_al_16,     12+12);
// addi_16!(..., pcdi) not present
// addi_16!(..., pcix) not present
// addi_16!(..., imm) not present

addi_32!(addi_32_d, dy,  16);
// addi_32_re!(..., ay) not present
addi_32!(addi_32_ai, ea_ay_ai_32,  20+8);
addi_32!(addi_32_pi, ea_ay_pi_32,  20+8);
addi_32!(addi_32_pd, ea_ay_pd_32,  20+10);
addi_32!(addi_32_di, ea_ay_di_32,  20+12);
addi_32!(addi_32_ix, ea_ay_ix_32,  20+14);
addi_32!(addi_32_aw, ea_aw_32,     20+12);
addi_32!(addi_32_al, ea_al_32,     20+16);
// addi_32!(..., pcdi) not present
// addi_32!(..., pcix) not present
// addi_32!(..., imm) not present

macro_rules! addq_8 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(8, add_8, $name, quick, $dst, $cycles);)
}
macro_rules! addq_16 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(16, add_16, $name, quick, $dst, $cycles);)
}
macro_rules! addq_32 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(32, add_32, $name, quick, $dst, $cycles);)
}

addq_8!(addq_8_d, dy, 4);
// addq_8!(..., ay) not present - word and long only
addq_8!(addq_8_ai, ea_ay_ai_8,  8+4);
addq_8!(addq_8_pi, ea_ay_pi_8,  8+4);
addq_8!(addq_8_pd, ea_ay_pd_8,  8+6);
addq_8!(addq_8_di, ea_ay_di_8,  8+8);
addq_8!(addq_8_ix, ea_ay_ix_8,  8+10);
addq_8!(addq_8_aw, ea_aw_8,     8+8);
addq_8!(addq_8_al, ea_al_8,     8+12);
// addq_8!(..., pcdi) not present
// addq_8!(..., pcix) not present
// addq_8!(..., imm) not present

addq_16!(addq_16_d, dy,  4);
pub fn addq_16_a(core: &mut Core) -> Result<Cycles> {
	let src = try!(operator::quick(core));
	let dst = ay!(core);
	// When adding to address registers, the condition codes are not
	// altered, and the entire destination address register is used
	// regardless of the operation size.
	ay!(core) = (Wrapping(dst) + Wrapping(src)).0;
	Ok(Cycles(4))
}
addq_16!(addq_16_ai, ea_ay_ai_16,  8+4);
addq_16!(addq_16_pi, ea_ay_pi_16,  8+4);
addq_16!(addq_16_pd, ea_ay_pd_16,  8+6);
addq_16!(addq_16_di, ea_ay_di_16,  8+8);
addq_16!(addq_16_ix, ea_ay_ix_16,  8+10);
addq_16!(addq_16_aw, ea_aw_16,     8+8);
addq_16!(addq_16_al, ea_al_16,     8+12);
// addq_16!(..., pcdi) not present
// addq_16!(..., pcix) not present
// addq_16!(..., imm) not present

addq_32!(addq_32_d, dy,  8);
pub fn addq_32_a(core: &mut Core) -> Result<Cycles> {
	let src = try!(operator::quick(core));
	let dst = ay!(core);
	// When adding to address registers, the condition codes are not
	// altered, and the entire destination address register is used
	// regardless of the operation size.
	ay!(core) = (Wrapping(dst) + Wrapping(src)).0;
	Ok(Cycles(8))
}
addq_32!(addq_32_ai, ea_ay_ai_32,  12+8);
addq_32!(addq_32_pi, ea_ay_pi_32,  12+8);
addq_32!(addq_32_pd, ea_ay_pd_32,  12+10);
addq_32!(addq_32_di, ea_ay_di_32,  12+12);
addq_32!(addq_32_ix, ea_ay_ix_32,  12+14);
addq_32!(addq_32_aw, ea_aw_32,     12+12);
addq_32!(addq_32_al, ea_al_32,     12+16);
// addq_32!(..., pcdi) not present
// addq_32!(..., pcix) not present
// addq_32!(..., imm) not present

impl_op!( 8, addx_8,  addx_8_rr, dy, dx, 4);
impl_op!( 8, addx_8,  addx_8_mm, ay_pd_8, ea_ax_pd_8, 18);
impl_op!(16, addx_16, addx_16_rr, dy, dx, 4);
impl_op!(16, addx_16, addx_16_mm, ay_pd_16, ea_ax_pd_16, 18);
impl_op!(32, addx_32, addx_32_rr, dy, dx, 8);
impl_op!(32, addx_32, addx_32_mm, ay_pd_32, ea_ax_pd_32, 30);

macro_rules! and_8_er {
	($name:ident, $src:ident, $cycles:expr) => (impl_op!(8, and_8, $name, $src, dx, $cycles);)
}
macro_rules! and_8_re {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(8, and_8, $name, dx, $dst, $cycles);)
}
macro_rules! and_16_er {
	($name:ident, $src:ident, $cycles:expr) => (impl_op!(16, and_16, $name, $src, dx, $cycles);)
}
macro_rules! and_16_re {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(16, and_16, $name, dx, $dst, $cycles);)
}
macro_rules! and_32_er {
	($name:ident, $src:ident, $cycles:expr) => (impl_op!(32, and_32, $name, $src, dx, $cycles);)
}
macro_rules! and_32_re {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(32, and_32, $name, dx, $dst, $cycles);)
}

and_8_er!(and_8_er_d, dy, 4);
// and_8_er!(..., ay) not present
and_8_er!(and_8_er_ai, ay_ai_8,   8);
and_8_er!(and_8_er_pi, ay_pi_8,   8);
and_8_er!(and_8_er_pd, ay_pd_8,  10);
and_8_er!(and_8_er_di, ay_di_8,  12);
and_8_er!(and_8_er_ix, ay_ix_8,  14);
and_8_er!(and_8_er_aw, aw_8,     12);
and_8_er!(and_8_er_al, al_8,     16);
and_8_er!(and_8_er_pcdi, pcdi_8, 12);
and_8_er!(and_8_er_pcix, pcix_8, 14);
and_8_er!(and_8_er_imm, imm_8,   10);

// and_8_re!(..., dy) not present
// and_8_re!(..., ay) not present
and_8_re!(and_8_re_ai, ea_ay_ai_8,  12);
and_8_re!(and_8_re_pi, ea_ay_pi_8,  12);
and_8_re!(and_8_re_pd, ea_ay_pd_8,  14);
and_8_re!(and_8_re_di, ea_ay_di_8,  16);
and_8_re!(and_8_re_ix, ea_ay_ix_8,  18);
and_8_re!(and_8_re_aw, ea_aw_8,     16);
and_8_re!(and_8_re_al, ea_al_8,     20);
// and_8_re!(..., pcdi) not present
// and_8_re!(..., pcix) not present
// and_8_re!(..., imm) not present

and_16_er!(and_16_er_d, dy,          4);
// and_16_er!(..., ay) not present
and_16_er!(and_16_er_ai, ay_ai_16,   8);
and_16_er!(and_16_er_pi, ay_pi_16,   8);
and_16_er!(and_16_er_pd, ay_pd_16,  10);
and_16_er!(and_16_er_di, ay_di_16,  12);
and_16_er!(and_16_er_ix, ay_ix_16,  14);
and_16_er!(and_16_er_aw, aw_16,     12);
and_16_er!(and_16_er_al, al_16,     16);
and_16_er!(and_16_er_pcdi, pcdi_16, 12);
and_16_er!(and_16_er_pcix, pcix_16, 14);
and_16_er!(and_16_er_imm, imm_16,   10);

// and_16_re!(..., dy) not present
// and_16_re!(..., ay) not present
and_16_re!(and_16_re_ai, ea_ay_ai_16,  12);
and_16_re!(and_16_re_pi, ea_ay_pi_16,  12);
and_16_re!(and_16_re_pd, ea_ay_pd_16,  14);
and_16_re!(and_16_re_di, ea_ay_di_16,  16);
and_16_re!(and_16_re_ix, ea_ay_ix_16,  18);
and_16_re!(and_16_re_aw, ea_aw_16,     16);
and_16_re!(and_16_re_al, ea_al_16,     20);
// and_16_re!(..., pcdi) not present
// and_16_re!(..., pcix) not present
// and_16_re!(..., imm) not present

and_32_er!(and_32_er_d, dy,          6);
// and_32_er!(..., ay) not present
and_32_er!(and_32_er_ai, ay_ai_32,  14);
and_32_er!(and_32_er_pi, ay_pi_32,  14);
and_32_er!(and_32_er_pd, ay_pd_32,  16);
and_32_er!(and_32_er_di, ay_di_32,  18);
and_32_er!(and_32_er_ix, ay_ix_32,  20);
and_32_er!(and_32_er_aw, aw_32,     18);
and_32_er!(and_32_er_al, al_32,     22);
and_32_er!(and_32_er_pcdi, pcdi_32, 18);
and_32_er!(and_32_er_pcix, pcix_32, 20);
and_32_er!(and_32_er_imm, imm_32,   16);

// and_32_re!(..., dy) not present
// and_32_re!(..., ay) not present
and_32_re!(and_32_re_ai, ea_ay_ai_32,  12+8);
and_32_re!(and_32_re_pi, ea_ay_pi_32,  12+8);
and_32_re!(and_32_re_pd, ea_ay_pd_32,  14+8);
and_32_re!(and_32_re_di, ea_ay_di_32,  16+8);
and_32_re!(and_32_re_ix, ea_ay_ix_32,  18+8);
and_32_re!(and_32_re_aw, ea_aw_32,     16+8);
and_32_re!(and_32_re_al, ea_al_32,     20+8);
// and_32_re!(..., pcdi) not present
// and_32_re!(..., pcix) not present
// and_32_re!(..., imm) not present

macro_rules! andi_8 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(8, and_8, $name, imm_8, $dst, $cycles);)
}
macro_rules! andi_16 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(16, and_16, $name, imm_16, $dst, $cycles);)
}
macro_rules! andi_32 {
	($name:ident, $dst:ident, $cycles:expr) => (impl_op!(32, and_32, $name, imm_32, $dst, $cycles);)
}
andi_8!(andi_8_d, dy,  8);
// andi_8_re!(..., ay) not present
andi_8!(andi_8_ai, ea_ay_ai_8,  12+4);
andi_8!(andi_8_pi, ea_ay_pi_8,  12+4);
andi_8!(andi_8_pd, ea_ay_pd_8,  12+6);
andi_8!(andi_8_di, ea_ay_di_8,  12+8);
andi_8!(andi_8_ix, ea_ay_ix_8,  12+10);
andi_8!(andi_8_aw, ea_aw_8,     12+8);
andi_8!(andi_8_al, ea_al_8,     12+12);
// andi_8!(..., pcdi) not present
// andi_8!(..., pcix) not present
// andi_8!(..., imm) not present

andi_16!(andi_16_d, dy,  8);
// andi_16_re!(..., ay) not present
andi_16!(andi_16_ai, ea_ay_ai_16,  12+4);
andi_16!(andi_16_pi, ea_ay_pi_16,  12+4);
andi_16!(andi_16_pd, ea_ay_pd_16,  12+6);
andi_16!(andi_16_di, ea_ay_di_16,  12+8);
andi_16!(andi_16_ix, ea_ay_ix_16,  12+10);
andi_16!(andi_16_aw, ea_aw_16,     12+8);
andi_16!(andi_16_al, ea_al_16,     12+12);
// andi_16!(..., pcdi) not present
// andi_16!(..., pcix) not present
// andi_16!(..., imm) not present

andi_32!(andi_32_d, dy,  14);
// andi_32_re!(..., ay) not present
andi_32!(andi_32_ai, ea_ay_ai_32,  20+8);
andi_32!(andi_32_pi, ea_ay_pi_32,  20+8);
andi_32!(andi_32_pd, ea_ay_pd_32,  20+10);
andi_32!(andi_32_di, ea_ay_di_32,  20+12);
andi_32!(andi_32_ix, ea_ay_ix_32,  20+14);
andi_32!(andi_32_aw, ea_aw_32,     20+12);
andi_32!(andi_32_al, ea_al_32,     20+16);
// andi_32!(..., pcdi) not present
// andi_32!(..., pcix) not present
// andi_32!(..., imm) not present

pub fn andi_16_toc(core: &mut Core) -> Result<Cycles> {
	let dst = core.condition_code_register();
	let src = mask_out_above_8!(try!(operator::imm_16(core)));
	core.ccr_to_flags(dst & src);
	Ok(Cycles(20))
}


macro_rules! asr_8 {
	($name:ident, $src:ident, $dst:ident, $cycles:expr) => (impl_shift_op!(8, asr_8, $name, $src, $dst, $cycles);)
}
macro_rules! asr_16 {
	($name:ident, $src:ident, $dst:ident, $cycles:expr) => (impl_shift_op!(16, asr_16, $name, $src, $dst, $cycles);)
}
macro_rules! asr_32 {
	($name:ident, $src:ident, $dst:ident, $cycles:expr) => (impl_shift_op!(32, asr_32, $name, $src, $dst, $cycles);)
}

macro_rules! asl_8 {
	($name:ident, $src:ident, $dst:ident, $cycles:expr) => (impl_shift_op!(8, asl_8, $name, $src, $dst, $cycles);)
}
macro_rules! asl_16 {
	($name:ident, $src:ident, $dst:ident, $cycles:expr) => (impl_shift_op!(16, asl_16, $name, $src, $dst, $cycles);)
}
macro_rules! asl_32 {
	($name:ident, $src:ident, $dst:ident, $cycles:expr) => (impl_shift_op!(32, asl_32, $name, $src, $dst, $cycles);)
}

asr_8!(asr_8_s, quick, dy, 6);
asr_16!(asr_16_s, quick, dy, 6);
asr_32!(asr_32_s, quick, dy, 8);
asr_8!(asr_8_r, dx, dy, 6);
asr_16!(asr_16_r, dx, dy, 6);
asr_32!(asr_32_r, dx, dy, 8);

asl_8!(asl_8_s, quick, dy, 6);
asl_16!(asl_16_s, quick, dy, 6);
asl_32!(asl_32_s, quick, dy, 8);
asl_8!(asl_8_r, dx, dy, 6);
asl_16!(asl_16_r, dx, dy, 6);
asl_32!(asl_32_r, dx, dy, 8);