use std::u32;

pub struct Bits<'d> {
	bytes: &'d [u8],
	available: u8,
	bits: u8
}

impl<'d> Bits<'d> {
	pub fn from_bytes(bytes: &'d [u8]) -> Self {
		if bytes.len() == 0 {
			Bits {
				bytes: &[],
				available: 0,
				bits: 0
			}
		} else {
			Bits {
				bytes: &bytes[1..],
				available: 8,
				bits: bytes[0]
			}
		}
	}

	/// Counts the remaining bits. Prefer has_remaining to avoid overflow panics.
	pub fn remaining(&self) -> usize {
		self.bytes.len() * 8 + (self.available as usize)
	}

	pub fn has_remaining(&self, amount: usize) -> bool {
		let min_remaining = self.bytes.len().saturating_mul(8).saturating_add(self.available as usize);

		min_remaining >= amount
	}

	pub fn read_bit(&mut self) -> bool {
		assert_ne!(self.available, 0, "Out of bytes, you should have checked Bits::remaining!");

		// The Speex bit stream gets the highest bit first, and then works its way down to the low bits.
		let bit = (self.bits >> (self.available - 1)) & 1;

		//println!("read bit: {}", bit);

		self.available -= 1;

		if self.available == 0 && self.bytes.len() > 0 {
			self.available = 8;
			self.bits = self.bytes[0];
			self.bytes = &self.bytes[1..];
		}

		bit != 0
	}

	pub fn read_bits(&mut self, bits: u8) -> u32 {
		assert!(bits <= 32, "Cannot read more than 32 bits at a time");

		let mut out = 0;

		for _ in 0..bits {
			out <<= 1;
			out |= self.read_bit() as u32;
		}

		out
	}

	pub fn read_bits_signed(&mut self, bits: u8) -> i32 {
		let mut out = self.read_bits(bits);
		let signed = out>>(bits-1) != 0;

		out |= if signed { u32::MAX << bits } else { 0 };

		out as i32
	}

	pub fn try_read_bit(&mut self) -> Option<bool> {
		if self.available != 0 {
			Some(self.read_bit())
		} else {
			None
		}
	}

	pub fn try_read_bits(&mut self, bits: u8) -> Option<u32> {
		if self.has_remaining(bits as usize) {
			Some(self.read_bits(bits))
		} else {
			None
		}
	}

	pub fn try_read_bits_signed(&mut self, bits: u8) -> Option<i32> {
		if self.has_remaining(bits as usize) {
			Some(self.read_bits_signed(bits))
		} else {
			None
		}
	}

	pub fn skip_bits(&mut self, bits: usize) {
		if bits < 16 {
			self.read_bits(bits as u8);
		}

		// First, take away the bits currently in the buffer.
		let bits = bits - (self.available as usize);

		// Skip all the full bytes.
		self.bytes = &self.bytes[bits / 8..];

		// Skip the trailing bits.
		let trailing_bits = (bits % 8) as u8;
		self.available = 8 - trailing_bits;

		// Return the bit buffer to a valid state.
		self.bits = self.bytes[0];
		self.bytes = &self.bytes[1..];
	}
}