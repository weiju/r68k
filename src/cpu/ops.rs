#![macro_use]
use super::Core;
macro_rules! ir_dx {
	($e:ident) => (($e.ir >> 9 & 7) as usize);
}
macro_rules! ir_dy {
	($e:ident) => (($e.ir & 7) as usize);
}
macro_rules! ir_ax {
	($e:ident) => (8+($e.ir >> 9 & 7) as usize);
}
macro_rules! ir_ay {
	($e:ident) => (8+($e.ir & 7) as usize);
}
macro_rules! dx {
	($e:ident) => ($e.dar[ir_dx!($e)]);
}
macro_rules! dy {
	($e:ident) => ($e.dar[ir_dy!($e)]);
}
macro_rules! ax {
	($e:ident) => ($e.dar[ir_ax!($e)]);
}
macro_rules! ay {
	($e:ident) => ($e.dar[ir_ay!($e)]);
}
macro_rules! mask_out_above_8 {
	($e:expr) => ($e & 0xff)
}
macro_rules! mask_out_below_8 {
	($e:expr) => ($e & !0xff)
}
macro_rules! mask_out_above_16 {
	($e:expr) => ($e & 0xffff)
}
macro_rules! mask_out_below_16 {
	($e:expr) => ($e & !0xffff)
}
macro_rules! low_nibble {
	($e:expr) => ($e & 0x0f);
}
macro_rules! high_nibble {
	($e:expr) => ($e & 0xf0);
}
macro_rules! true1 {
	($e:expr) => (if $e {1} else {0})
}
macro_rules! not1 {
	($e:expr) => (true1!($e == 0))
}

pub mod fake {
	use super::super::Core;

	pub fn set_d0(core: &mut Core) {
		core.dar[0] = 0xabcd;
	}

	pub fn set_d1(core: &mut Core) {
		core.dar[1] = 0xbcde;
	}

	pub fn set_dx(core: &mut Core) {
		dx!(core) = 0xcdef;
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

pub fn illegal(core: &mut Core) {
	panic!("Illegal instruction {:04x} at {:08x}", core.ir, core.pc-2);
}

use std::num::Wrapping;
use super::operator;

// All instructions are ported from https://github.com/kstenerud/Musashi
pub fn abcd_8_common(core: &mut Core, dst: u32, src: u32) -> u32 {
	// unsigned int res = ((src) & 0x0f) + ((dst) & 0x0f) + ((m68ki_cpu.x_flag>>8)&1);
	let mut res = low_nibble!(src) + low_nibble!(dst) + core.x_flag_as_1();

	// m68ki_cpu.v_flag = ~res;
	core.v_flag = !res;

	// if(res > 9)
	//  res += 6;
	if res > 9 {
		res += 6;
	}
	// res += ((src) & 0xf0) + ((dst) & 0xf0);
	res += high_nibble!(src) + high_nibble!(dst);
	// m68ki_cpu.x_flag = m68ki_cpu.c_flag = (res > 0x99) << 8;
	core.c_flag = true1!(res > 0x99) << 8;
	core.x_flag = core.c_flag;

	if core.c_flag > 0 {
		res = (Wrapping(res) - Wrapping(0xa0)).0;
	}

	// m68ki_cpu.v_flag &= res;
	// m68ki_cpu.n_flag = (res);
	core.v_flag &= res;
	core.n_flag = res;

	// res = ((res) & 0xff);
	// m68ki_cpu.not_z_flag |= res;
	res = mask_out_above_8!(res);
	core.not_z_flag |= res;
	res
}
pub fn abcd_8_rr(core: &mut Core) {
	let dst = operator::dx(core);
	let src = operator::dy(core);
	let res = abcd_8_common(core, dst, src);
	dx!(core) = mask_out_below_8!(dst) | res;
}
pub fn abcd_8_mm(core: &mut Core) {
	let src = operator::ay_pd_8(core);
	let (dst, ea) = operator::ax_pd_8(core);
	let res = abcd_8_common(core, dst, src);
	core.write_data_byte(ea, res);
}

fn add_8_common(core: &mut Core, dst: u32, src: u32) -> u32 {
	let dst = mask_out_above_8!(dst);
	let src = mask_out_above_8!(src);

	let res = dst + src;
	// m68ki_cpu.n_flag = (res);
	core.n_flag = res;
	// m68ki_cpu.v_flag = ((src^res) & (dst^res));
	core.v_flag = (src ^ res) & (dst ^ res);
	// m68ki_cpu.x_flag = m68ki_cpu.c_flag = (res);
	core.c_flag = res;
	core.x_flag = res;
	// m68ki_cpu.not_z_flag = ((res) & 0xff);
	let res8 = mask_out_above_8!(res);
	core.not_z_flag = res8;
	res8
}
macro_rules! add_8_er {
    ($name:ident, $src:ident) => (
    	pub fn $name(core: &mut Core) {
			let dst = operator::dx(core);
			let src = operator::$src(core);
			let res = add_8_common(core, dst, src);
			dx!(core) = mask_out_below_8!(dst) | res;
    	})
}
add_8_er!(add_8_er_d, dy);
add_8_er!(add_8_er_ai, ay_ai_8);
add_8_er!(add_8_er_pi, ay_pi_8);
add_8_er!(add_8_er_pd, ay_pd_8);
add_8_er!(add_8_er_di, ay_di_8);
add_8_er!(add_8_er_ix, ay_ix_8);
add_8_er!(add_8_er_aw, aw_8);
add_8_er!(add_8_er_al, al_8);
add_8_er!(add_8_er_pcdi, pcdi_8);
add_8_er!(add_8_er_pcix, pcix_8);
add_8_er!(add_8_er_imm, imm_8);

use super::Handler;
#[allow(dead_code)]
struct OpcodeHandler {
	mask: u32,
	matching: u32,
	name: String,
	handler: Handler
}

use super::InstructionSet;
macro_rules! op_entry {
    ($mask:expr, $matching:expr, $handler:ident) => (OpcodeHandler { mask: $mask, matching: $matching, handler: $handler, name: stringify!($handler).to_string() })
}
pub const MASK_OUT_X_Y: u32 = 0b1111000111111000; // masks out X and Y register bits (????xxx??????yyy)
pub const MASK_OUT_X: u32 = 0b1111000111111111; // masks out X register bits (????xxx?????????)
pub const OP_ABCD_8_RR: u32 = 0xc100;
pub const OP_ABCD_8_MM: u32 = 0xc108;
pub const OP_ADD_8_ER_D: u32 = 0xd000;
pub const OP_ADD_8_ER_AI: u32 = 0xd010;
pub const OP_ADD_8_ER_PI: u32 = 0xd018;
pub const OP_ADD_8_ER_PD: u32 = 0xd020;
pub const OP_ADD_8_ER_DI: u32 = 0xd028;
pub const OP_ADD_8_ER_IX: u32 = 0xd030;
pub const OP_ADD_8_ER_AW: u32 = 0xd038;
pub const OP_ADD_8_ER_AL: u32 = 0xd039;
pub const OP_ADD_8_ER_PCDI: u32 = 0xd03a;
pub const OP_ADD_8_ER_PCIX: u32 = 0xd03b;
pub const OP_ADD_8_ER_IMM: u32 = 0xd03c;

pub fn instruction_set() -> InstructionSet {
	// Covers all possible IR values (64k entries)
	let mut handler: InstructionSet = Vec::with_capacity(0x10000);
	for _ in 0..0x10000 { handler.push(illegal); }
	//let handler = [illegal].iter().cycle().take(0x10000).collect::<InstructionSet>();
	// (0..0x10000).map(|_| illegal).collect::<InstructionSet>();
	// the optable contains opcode mask, matching mask and the corresponding handler + name
	let optable = vec![
		op_entry!(MASK_OUT_X_Y, OP_ABCD_8_RR, abcd_8_rr),
		op_entry!(MASK_OUT_X_Y, OP_ABCD_8_MM, abcd_8_mm),
		op_entry!(MASK_OUT_X_Y, OP_ADD_8_ER_D, add_8_er_d),
		op_entry!(MASK_OUT_X_Y, OP_ADD_8_ER_AI, add_8_er_ai),
		op_entry!(MASK_OUT_X_Y, OP_ADD_8_ER_PI, add_8_er_pi),
		op_entry!(MASK_OUT_X_Y, OP_ADD_8_ER_PD, add_8_er_pd),
		op_entry!(MASK_OUT_X_Y, OP_ADD_8_ER_DI, add_8_er_di),
		op_entry!(MASK_OUT_X_Y, OP_ADD_8_ER_IX, add_8_er_ix),
		op_entry!(MASK_OUT_X, OP_ADD_8_ER_AW, add_8_er_aw),
		op_entry!(MASK_OUT_X, OP_ADD_8_ER_AL, add_8_er_al),
		op_entry!(MASK_OUT_X, OP_ADD_8_ER_PCDI, add_8_er_pcdi),
		op_entry!(MASK_OUT_X, OP_ADD_8_ER_PCIX, add_8_er_pcix),
		op_entry!(MASK_OUT_X, OP_ADD_8_ER_IMM, add_8_er_imm),
	];
	for op in optable {
		for opcode in 0..0x10000 {
			if (opcode & op.mask) == op.matching {
				// println!("{:16b}: {}", opcode, op.name);
				handler[opcode as usize] = op.handler;
			}
		}
	}
	handler
}

#[cfg(test)]
mod tests {
	use super::super::Core;

	#[test]
	fn low_nibble() {
		assert_eq!(0x0a, low_nibble!(0xba));
	}
	#[test]
	fn high_nibble() {
		assert_eq!(0xb0, high_nibble!(0xba));
	}
	#[test]
	fn mask_out_below_8() {
		assert_eq!(0x2bcdef00, mask_out_below_8!(0x2bcdef73));
	}
	#[test]
	fn mask_out_above_8() {
		assert_eq!(0xf1, mask_out_above_8!(0x2bcdeff1));
	}
	#[test]
	fn dx_and_dy() {
		let mut core = Core::new(0x40);
		core.dar[0] = 0x00;
		core.dar[1] = 0x11;
		core.dar[2] = 0x22;
		core.dar[3] = 0x33;
		core.dar[4] = 0x44;
		core.dar[5] = 0x55;
		core.dar[6] = 0x66;
		core.dar[7] = 0x77;

		core.ir = 0b1111_1001_1111_1010; // X=4, Y=2
		assert_eq!(0x22, dy!(core));
		assert_eq!(0x44, dx!(core));

		core.ir = 0b1111_1011_1111_1110; // X=5, Y=6
		assert_eq!(0x66, dy!(core));
		assert_eq!(0x55, dx!(core));
	}
}