//! Physical layer interactions.
//!
//! Don't expect to see much code here: Most of this layer is hardware.
//!
//! BLE data is transmitted on 40 different RF channels numbered from 0 to 39 with ascending
//! frequencies. Channels 0, 12 and 39 are reserved for advertising, all others are used for data
//! transmission. BLE internally uses so-called "Channel Indices" which reorder the RF channels so
//! that indices 0..=36 refer to data channels and 37..=39 refer to the advertising channels
//! (presumably to simplify channel hopping). The Link-Layer is only interested in these channel
//! indices, so only those are implemented here.

/// Returns the center frequency in MHz corresponding to an RF channel.
fn rf_channel_freq(rf_channel: u8) -> u16 {
    2402 + u16::from(rf_channel) * 2
}

fn whitening_iv(channel_idx: u8) -> u8 {
    debug_assert!(channel_idx <= 39);
    0b01000000 | channel_idx
}

/// Represents one of the three advertising channels (channel indices 37, 38 or 39).
#[derive(Copy, Clone)]
pub struct AdvertisingChannelIndex(u8);

impl AdvertisingChannelIndex {
    /// Returns the first (lowest-numbered) advertising channel.
    pub fn first() -> Self {
        AdvertisingChannelIndex(37)
    }

    /// Returns the next advertising channel, or the first one if `self` is the last channel.
    pub fn cycle(&self) -> Self {
        if self.0 == 39 {
            AdvertisingChannelIndex(37)
        } else {
            AdvertisingChannelIndex(self.0 + 1)
        }
    }

    /// Returns the RF channel corresponding to this advertising channel index.
    ///
    /// RF channels 0, 12 and 39 are used for advertising.
    pub fn rf_channel(&self) -> u8 {
        match self.0 {
            37 => 0,
            38 => 12,
            39 => 39,
            _ => unreachable!(),
        }
    }

    /// Returns the frequency of this channel in MHz.
    pub fn freq(&self) -> u16 {
        rf_channel_freq(self.rf_channel())
    }

    /// Calculates the initial value of the LFSR to use for data whitening.
    ///
    /// The value is a 7-bit value. The MSb will always be 0, and the 2nd MSb always 1 (Position 0).
    /// The LSb contains Position 6. Refer to the specification for details about the bit positions.
    ///
    /// The polynomial is always `x^7 + x^4 + 1`.
    ///
    /// Whitening is applied to PDU and CRC.
    pub fn whitening_iv(&self) -> u8 {
        whitening_iv(self.0)
    }
}

/// A data channel index uniquely identifies an RF channel on which only data PDUs are sent.
///
/// (channel indices 0..=36)
#[derive(Copy, Clone)]
pub struct DataChannelIndex(u8);

impl DataChannelIndex {
    /// Creates a `DataChannelIndex` from a raw index.
    ///
    /// # Panics
    ///
    /// This will panic if `index` is not a valid data channel index. Valid indices are 0..=36.
    pub fn new(index: u8) -> Self {
        assert!(index <= 36);
        DataChannelIndex(index)
    }

    /// Returns the RF channel corresponding to this data channel index.
    ///
    /// RF channels 1-11 and 13-38 are used for data transmission.
    pub fn rf_channel(&self) -> u8 {
        match self.0 {
            ch @ 0...10 => ch + 1,
            ch @ 11...36 => ch + 2,
            _ => unreachable!(),
        }
    }

    /// Returns the frequency of this channel in MHz.
    pub fn freq(&self) -> u16 {
        rf_channel_freq(self.rf_channel())
    }

    /// Calculates the initial value of the LFSR to use for data whitening.
    ///
    /// The value is a 7-bit value. The MSb will always be 0, and the 2nd MSb always 1 (Position 0).
    /// The LSb contains Position 6. Refer to the specification for details about the bit positions.
    ///
    /// The polynomial is always `x^7 + x^4 + 1`.
    ///
    /// Whitening is applied to PDU and CRC.
    pub fn whitening_iv(&self) -> u8 {
        whitening_iv(self.0)
    }
}

/// Trait for raw 2.4 GHz non-BLE-specific radios.
///
/// You probably won't need to implement this trait, unless you're working with hardware that has
/// absolutely no special support for BLE. Usually, the Link-Layer `Transmitter` should be
/// implemented.
pub trait Radio {
    /// Transmit every Byte in `buf` over the air, LSb first, at `freq` MHz.
    fn transmit(&mut self, buf: &mut [u8], freq: u16);
}

// FIXME Add helpers for parsing adv/data PDU into their headers and payload so they can be passed to the LinkLayer