use bits::Bits;

const WB_SKIPS: [usize; 8] = [0, 32, 108, 188, 348, 0, 0, 0];
const NB_SKIPS: [usize; 9] = [0, 38, 114, 155, 215, 295, 359, 487, 74];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NbSubmode {
	Null,
	Nb2150,
	Nb5950,
	Nb8000,
	Nb11000,
	Nb15000,
	Nb18200,
	Nb24600,
	Nb3950,
}

struct Mode(u8);

pub struct State {
	encode_submode: bool
}

impl State {
	pub fn skip_wideband(bits: &mut Bits) {
		let submode = bits.read_bits(3);
		bits.skip_bits(WB_SKIPS[submode as usize]);
	}

	pub fn find_next_submode(bits: &mut Bits) -> NbSubmode {
		loop {
			if !bits.has_remaining(5) {
				panic!("not enough bits");
			}

			if bits.read_bit() {
				// Wideband

				State::skip_wideband(bits);

				if !bits.has_remaining(5) {
					panic!("not enough bits");
				}

				if bits.read_bit() {
					State::skip_wideband(bits);

					if bits.read_bit() {
						panic!("Cannot skip more than 2 wideband layers in narrowband decoding, stream corrupted!");
					}
				}
			}

			let mode = bits.try_read_bits(4).expect("not enough bits") as u8;

			return match mode {
				0 => NbSubmode::Null,
				1 => NbSubmode::Nb2150,
				2 => NbSubmode::Nb5950,
				3 => NbSubmode::Nb8000,
				4 => NbSubmode::Nb11000,
				5 => NbSubmode::Nb15000,
				6 => NbSubmode::Nb18200,
				7 => NbSubmode::Nb24600,
				8 => NbSubmode::Nb3950,
				9...12 => panic!("invalid mode, stream corrupted!"),
				13 => unimplemented!("user inband request"),
				14 => unimplemented!("speex inband request"),
				15 => unimplemented!("terminator"),
				_ => unreachable!()
			};
		}
	}

	fn reset(&mut self) {

	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LspLbr {
	pub main: u8,
	pub low:  u8,
	pub high: u8
}

impl LspLbr {
	pub fn read(bits: &mut Bits) -> Self {
		LspLbr {
			main: bits.read_bits(6) as u8,
			low:  bits.read_bits(6) as u8,
			high: bits.read_bits(6) as u8
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LspNb {
	pub main:  u8,
	pub low:  (u8, u8),
	pub high: (u8, u8)
}

impl LspNb {
	pub fn read(bits: &mut Bits) -> Self {
		LspNb {
			main: bits.read_bits(6) as u8,
			low:  (
				bits.read_bits(6) as u8,
				bits.read_bits(6) as u8
			),
			high: (
				bits.read_bits(6) as u8,
				bits.read_bits(6) as u8
			)
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LtpForcedPitch;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LtpG {
	pub gain_index: u8
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LtpGp {
	pub pitch:      u8,
	pub gain_index: u8
}



// -- Format --

enum LspKind { Lbr, Nb }
struct LtpOptions {
	/// False => 0 bits, True => 7 bits
	pitch:   bool,
	/// False => 5 bits, True => 7 bits
	gain_nb: bool
}

struct Format {
	lsp: LspKind,
	lbr_pitch: bool,
	forced_pitch_gain: bool,
	ltp: LtpOptions,
	subframe_gain: u8, // TODO: 3, 1, 0 (None)
	innov_bits:  u8, // TODO
	innov_len:   u8,
	innov2_bits: u8, // TODO
	innov2_len:  u8
}

struct Frame {
	qlsp: Result<LspLbr, LspNb>,
	lbr_pitch: Option<u8>, // 7 bits
	forced_pitch_gain: Option<u8>, // 4 bits
	qe: u8, // 5 bits
	dtx: bool, // read as 4 bits in Nb2150, true if all bits are set, always false in other modes
	ltp_pitch:     Option<[u8; 4]>,
	gain_indices:  Option<[u8; 4]>,
	subframe_gain: Option<[u8; 4]>, // 3 or 1 bits
	innov:  [u8; 8], // TODO
	innov2: [u8; 8] // TODO
}

/*
let qlsp = [spx_lsp_t; NB_ORDER];
lsp_unquant(qlsp, NB_ORDER, bits)

...

if lbr_pitch != -1 { READ:u7 }

...

if forced_pitch_gain { READ:u4 }

...

qe = READ: u5

...

extra = if submode == NbSubmode::Nb2150 { READ: u4 }

...

for _ in 0..NB_NB_SUBFRAMES {
	READ: LTP_UNQUANT

	...

	if have_subframe_gain == 3 { READ:3 } else if have_subframe_gain==1 { READ:1 }

	....

	READ: INNOVATION_UNQUANT

	...

	if double_codebook { READ: INNOVATION_UNQUANT }
}
*/