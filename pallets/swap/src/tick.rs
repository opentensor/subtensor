//! The math is adapted from github.com/0xKitsune/uniswap-v3-math
use core::convert::TryFrom;
use core::error::Error;
use core::fmt;
use core::hash::Hash;
use core::ops::{Add, AddAssign, BitOr, Deref, Neg, Shl, Shr, Sub, SubAssign};

use alloy_primitives::{I256, U256};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_math::*;
use substrate_fixed::types::U64F64;

use crate::pallet::{
    Config, CurrentTickIndex, FeeGlobalAlpha, FeeGlobalTao, TickIndexBitmapWords, Ticks,
};
use crate::{NetUid, SqrtPrice};

const U256_1: U256 = U256::from_limbs([1, 0, 0, 0]);
const U256_2: U256 = U256::from_limbs([2, 0, 0, 0]);
const U256_3: U256 = U256::from_limbs([3, 0, 0, 0]);
const U256_4: U256 = U256::from_limbs([4, 0, 0, 0]);
const U256_5: U256 = U256::from_limbs([5, 0, 0, 0]);
const U256_6: U256 = U256::from_limbs([6, 0, 0, 0]);
const U256_7: U256 = U256::from_limbs([7, 0, 0, 0]);
const U256_8: U256 = U256::from_limbs([8, 0, 0, 0]);
const U256_15: U256 = U256::from_limbs([15, 0, 0, 0]);
const U256_16: U256 = U256::from_limbs([16, 0, 0, 0]);
const U256_32: U256 = U256::from_limbs([32, 0, 0, 0]);
const U256_64: U256 = U256::from_limbs([64, 0, 0, 0]);
const U256_127: U256 = U256::from_limbs([127, 0, 0, 0]);
const U256_128: U256 = U256::from_limbs([128, 0, 0, 0]);
const U256_255: U256 = U256::from_limbs([255, 0, 0, 0]);

const U256_256: U256 = U256::from_limbs([256, 0, 0, 0]);
const U256_512: U256 = U256::from_limbs([512, 0, 0, 0]);
const U256_1024: U256 = U256::from_limbs([1024, 0, 0, 0]);
const U256_2048: U256 = U256::from_limbs([2048, 0, 0, 0]);
const U256_4096: U256 = U256::from_limbs([4096, 0, 0, 0]);
const U256_8192: U256 = U256::from_limbs([8192, 0, 0, 0]);
const U256_16384: U256 = U256::from_limbs([16384, 0, 0, 0]);
const U256_32768: U256 = U256::from_limbs([32768, 0, 0, 0]);
const U256_65536: U256 = U256::from_limbs([65536, 0, 0, 0]);
const U256_131072: U256 = U256::from_limbs([131072, 0, 0, 0]);
const U256_262144: U256 = U256::from_limbs([262144, 0, 0, 0]);
const U256_524288: U256 = U256::from_limbs([524288, 0, 0, 0]);

const U256_MAX_TICK: U256 = U256::from_limbs([887272, 0, 0, 0]);

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = -MIN_TICK;

const MIN_SQRT_RATIO: U256 = U256::from_limbs([4295128739, 0, 0, 0]);
const MAX_SQRT_RATIO: U256 =
    U256::from_limbs([6743328256752651558, 17280870778742802505, 4294805859, 0]);

const SQRT_10001: I256 = I256::from_raw(U256::from_limbs([11745905768312294533, 13863, 0, 0]));
const TICK_LOW: I256 = I256::from_raw(U256::from_limbs([
    6552757943157144234,
    184476617836266586,
    0,
    0,
]));
const TICK_HIGH: I256 = I256::from_raw(U256::from_limbs([
    4998474450511881007,
    15793544031827761793,
    0,
    0,
]));

/// Tick is the price range determined by tick index (not part of this struct, but is the key at
/// which the Tick is stored in state hash maps). Tick struct stores liquidity and fee information.
///
///  - Net liquidity
///  - Gross liquidity
///  - Fees (above global) in both currencies
#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct Tick {
    pub liquidity_net: i128,
    pub liquidity_gross: u64,
    pub fees_out_tao: U64F64,
    pub fees_out_alpha: U64F64,
}

/// Struct representing a tick index
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct TickIndex(i32);

impl Add<TickIndex> for TickIndex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // Note: This assumes the result is within bounds.
        // For a safer implementation, consider using checked_add.
        Self::new_unchecked(self.get() + rhs.get())
    }
}

impl Sub<TickIndex> for TickIndex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // Note: This assumes the result is within bounds.
        // For a safer implementation, consider using checked_sub.
        Self::new_unchecked(self.get() - rhs.get())
    }
}

impl AddAssign<TickIndex> for TickIndex {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::new_unchecked(self.get() + rhs.get());
    }
}

impl SubAssign<TickIndex> for TickIndex {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::new_unchecked(self.get() - rhs.get());
    }
}

impl TryFrom<i32> for TickIndex {
    type Error = TickMathError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Deref for TickIndex {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        // Using get() would create an infinite recursion, so this is one place where we need direct
        // field access. This is safe because Self::Target is i32, which is exactly what we're
        // storing
        &self.0
    }
}

/// Extension trait to make working with TryFrom more ergonomic
pub trait TryIntoTickIndex {
    /// Convert an i32 into a TickIndex, with bounds checking
    fn into_tick_index(self) -> Result<TickIndex, TickMathError>;
}

impl TryIntoTickIndex for i32 {
    fn into_tick_index(self) -> Result<TickIndex, TickMathError> {
        TickIndex::try_from(self)
    }
}

impl TickIndex {
    /// Minimum value of the tick index
    /// The tick_math library uses different bitness, so we have to divide by 2.
    /// It's unsafe to change this value to something else.
    pub const MIN: Self = Self(MIN_TICK / 2);

    /// Maximum value of the tick index
    /// The tick_math library uses different bitness, so we have to divide by 2.
    /// It's unsafe to change this value to something else.
    pub const MAX: Self = Self(MAX_TICK / 2);

    /// All tick indexes are offset by this value for storage needs
    /// so that tick indexes are positive, which simplifies bit logic
    const OFFSET: Self = Self(MAX_TICK);

    /// Get fees above a tick
    pub fn fees_above<T: Config>(&self, netuid: NetUid, quote: bool) -> U64F64 {
        let current_tick = CurrentTickIndex::<T>::get(netuid).unwrap_or_default();

        let Some(tick_index) = ActiveTickIndexManager::find_closest_lower::<T>(netuid, *self)
        else {
            return U64F64::from_num(0);
        };

        let tick = Ticks::<T>::get(netuid, tick_index).unwrap_or_default();
        if tick_index <= current_tick {
            if quote {
                FeeGlobalTao::<T>::get(netuid)
                    .unwrap_or_default()
                    .saturating_sub(tick.fees_out_tao)
            } else {
                FeeGlobalAlpha::<T>::get(netuid)
                    .unwrap_or_default()
                    .saturating_sub(tick.fees_out_alpha)
            }
        } else if quote {
            tick.fees_out_tao
        } else {
            tick.fees_out_alpha
        }
    }

    /// Get fees below a tick
    pub fn fees_below<T: Config>(&self, netuid: NetUid, quote: bool) -> U64F64 {
        let current_tick = CurrentTickIndex::<T>::get(netuid).unwrap_or_default();

        let Some(tick_index) = ActiveTickIndexManager::find_closest_lower::<T>(netuid, *self)
        else {
            return U64F64::saturating_from_num(0);
        };

        let tick = Ticks::<T>::get(netuid, tick_index).unwrap_or_default();

        if tick_index <= current_tick {
            if quote {
                tick.fees_out_tao
            } else {
                tick.fees_out_alpha
            }
        } else if quote {
            FeeGlobalTao::<T>::get(netuid)
                .unwrap_or_default()
                .saturating_sub(tick.fees_out_tao)
        } else {
            FeeGlobalAlpha::<T>::get(netuid)
                .unwrap_or_default()
                .saturating_sub(tick.fees_out_alpha)
        }
    }

    /// Converts a sqrt price to a tick index, ensuring it's within valid bounds
    ///
    /// If the price is outside the valid range, this function will return the appropriate boundary
    /// tick index (MIN or MAX) instead of an error.
    ///
    /// # Arguments
    /// * `sqrt_price` - The square root price to convert to a tick index
    ///
    /// # Returns
    /// * `TickIndex` - A tick index that is guaranteed to be within valid bounds
    pub fn from_sqrt_price_bounded(sqrt_price: SqrtPrice) -> Self {
        match Self::try_from_sqrt_price(sqrt_price) {
            Ok(index) => index,
            Err(_) => {
                let max_price = Self::MAX
                    .try_to_sqrt_price()
                    .unwrap_or(SqrtPrice::saturating_from_num(1000));

                if sqrt_price > max_price {
                    Self::MAX
                } else {
                    Self::MIN
                }
            }
        }
    }

    /// Converts a tick index to a sqrt price, ensuring it's within valid bounds
    ///
    /// Unlike try_to_sqrt_price which returns an error for boundary indices, this function
    /// guarantees a valid sqrt price by using fallback values if conversion fails.
    ///
    /// # Returns
    /// * `SqrtPrice` - A sqrt price that is guaranteed to be a valid value
    pub fn to_sqrt_price_bounded(&self) -> SqrtPrice {
        self.try_to_sqrt_price().unwrap_or_else(|_| {
            if *self >= Self::MAX {
                SqrtPrice::saturating_from_num(1000)
            } else {
                SqrtPrice::saturating_from_num(0.000001)
            }
        })
    }

    /// Creates a new TickIndex instance with bounds checking
    pub fn new(value: i32) -> Result<Self, TickMathError> {
        if !(Self::MIN.0..=Self::MAX.0).contains(&value) {
            Err(TickMathError::TickOutOfBounds)
        } else {
            Ok(Self(value))
        }
    }

    /// Creates a new TickIndex without bounds checking
    /// Use this function with caution, only when you're certain the value is valid
    pub fn new_unchecked(value: i32) -> Self {
        Self(value)
    }

    /// Get the inner value
    pub fn get(&self) -> i32 {
        self.0
    }

    /// Creates a TickIndex from an offset representation (u32)
    ///
    /// # Arguments
    /// * `offset_index` - An offset index (u32 value) representing a tick index
    ///
    /// # Returns
    /// * `Result<TickIndex, TickMathError>` - The corresponding TickIndex if within valid bounds
    pub fn from_offset_index(offset_index: u32) -> Result<Self, TickMathError> {
        let signed_index = (offset_index as i64 - Self::OFFSET.get() as i64) as i32;
        Self::new(signed_index)
    }

    /// Get the next tick index (incrementing by 1)
    pub fn next(&self) -> Result<Self, TickMathError> {
        Self::new(self.0 + 1)
    }

    /// Get the previous tick index (decrementing by 1)
    pub fn prev(&self) -> Result<Self, TickMathError> {
        Self::new(self.0 - 1)
    }

    /// Add a value to this tick index with bounds checking
    pub fn checked_add(&self, value: i32) -> Result<Self, TickMathError> {
        Self::new(self.0 + value)
    }

    /// Subtract a value from this tick index with bounds checking
    pub fn checked_sub(&self, value: i32) -> Result<Self, TickMathError> {
        Self::new(self.0 - value)
    }

    /// Add a value to this tick index, saturating at the bounds instead of overflowing
    pub fn saturating_add(&self, value: i32) -> Self {
        match self.checked_add(value) {
            Ok(result) => result,
            Err(_) => {
                if value > 0 {
                    Self::MAX
                } else {
                    Self::MIN
                }
            }
        }
    }

    /// Subtract a value from this tick index, saturating at the bounds instead of overflowing
    pub fn saturating_sub(&self, value: i32) -> Self {
        match self.checked_sub(value) {
            Ok(result) => result,
            Err(_) => {
                if value > 0 {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Divide the tick index by a value with bounds checking
    pub fn checked_div(&self, value: i32) -> Result<Self, TickMathError> {
        if value == 0 {
            return Err(TickMathError::DivisionByZero);
        }
        Self::new(self.0 / value)
    }

    /// Divide the tick index by a value, saturating at the bounds
    pub fn saturating_div(&self, value: i32) -> Self {
        if value == 0 {
            return Self::MAX; // Return MAX for division by zero
        }
        match self.checked_div(value) {
            Ok(result) => result,
            Err(_) => {
                if (self.0 < 0 && value > 0) || (self.0 > 0 && value < 0) {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Multiply the tick index by a value with bounds checking
    pub fn checked_mul(&self, value: i32) -> Result<Self, TickMathError> {
        // Check for potential overflow
        match self.0.checked_mul(value) {
            Some(result) => Self::new(result),
            None => Err(TickMathError::Overflow),
        }
    }

    /// Multiply the tick index by a value, saturating at the bounds
    pub fn saturating_mul(&self, value: i32) -> Self {
        match self.checked_mul(value) {
            Ok(result) => result,
            Err(_) => {
                if (self.0 < 0 && value > 0) || (self.0 > 0 && value < 0) {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Converts tick index into SQRT of lower price of this tick In order to find the higher price
    /// of this tick, call tick_index_to_sqrt_price(tick_idx + 1)
    pub fn try_to_sqrt_price(&self) -> Result<SqrtPrice, TickMathError> {
        // because of u256->u128 conversion we have twice less values for min/max ticks
        if !(Self::MIN..=Self::MAX).contains(self) {
            return Err(TickMathError::TickOutOfBounds);
        }
        get_sqrt_ratio_at_tick(self.0).and_then(u256_q64_96_to_u64f64)
    }

    /// Converts SQRT price to tick index
    /// Because the tick is the range of prices [sqrt_lower_price, sqrt_higher_price), the resulting
    /// tick index matches the price by the following inequality:
    ///    sqrt_lower_price <= sqrt_price < sqrt_higher_price
    pub fn try_from_sqrt_price(sqrt_price: SqrtPrice) -> Result<Self, TickMathError> {
        let tick = get_tick_at_sqrt_ratio(u64f64_to_u256_q64_96(sqrt_price))?;

        // Correct for rounding error during conversions between different fixed-point formats
        if tick == 0 {
            Ok(Self(tick))
        } else {
            match (tick + 1).into_tick_index() {
                Ok(incremented) => Ok(incremented),
                Err(e) => Err(e),
            }
        }
    }
}

pub struct ActiveTickIndexManager;

impl ActiveTickIndexManager {
    pub fn insert<T: Config>(netuid: NetUid, index: TickIndex) {
        // Check the range
        if (index < TickIndex::MIN) || (index > TickIndex::MAX) {
            return;
        }

        // Convert to bitmap representation
        let bitmap = TickIndexBitmap::from(index);

        // Update layer words
        let mut word0_value = TickIndexBitmapWords::<T>::get((
            netuid,
            LayerLevel::Top,
            bitmap.word_at(LayerLevel::Top),
        ));
        let mut word1_value = TickIndexBitmapWords::<T>::get((
            netuid,
            LayerLevel::Middle,
            bitmap.word_at(LayerLevel::Middle),
        ));
        let mut word2_value = TickIndexBitmapWords::<T>::get((
            netuid,
            LayerLevel::Bottom,
            bitmap.word_at(LayerLevel::Bottom),
        ));

        // Set bits in each layer
        word0_value |= bitmap.bit_mask(LayerLevel::Top);
        word1_value |= bitmap.bit_mask(LayerLevel::Middle);
        word2_value |= bitmap.bit_mask(LayerLevel::Bottom);

        // Update the storage
        TickIndexBitmapWords::<T>::set(
            (netuid, LayerLevel::Top, bitmap.word_at(LayerLevel::Top)),
            word0_value,
        );
        TickIndexBitmapWords::<T>::set(
            (
                netuid,
                LayerLevel::Middle,
                bitmap.word_at(LayerLevel::Middle),
            ),
            word1_value,
        );
        TickIndexBitmapWords::<T>::set(
            (
                netuid,
                LayerLevel::Bottom,
                bitmap.word_at(LayerLevel::Bottom),
            ),
            word2_value,
        );
    }

    pub fn remove<T: Config>(netuid: NetUid, index: TickIndex) {
        // Check the range
        if (index < TickIndex::MIN) || (index > TickIndex::MAX) {
            return;
        }

        // Convert to bitmap representation
        let bitmap = TickIndexBitmap::from(index);

        // Update layer words
        let mut word0_value = TickIndexBitmapWords::<T>::get((
            netuid,
            LayerLevel::Top,
            bitmap.word_at(LayerLevel::Top),
        ));
        let mut word1_value = TickIndexBitmapWords::<T>::get((
            netuid,
            LayerLevel::Middle,
            bitmap.word_at(LayerLevel::Middle),
        ));
        let mut word2_value = TickIndexBitmapWords::<T>::get((
            netuid,
            LayerLevel::Bottom,
            bitmap.word_at(LayerLevel::Bottom),
        ));

        // Turn the bit off (& !bit) and save as needed
        word2_value &= !bitmap.bit_mask(LayerLevel::Bottom);
        TickIndexBitmapWords::<T>::set(
            (
                netuid,
                LayerLevel::Bottom,
                bitmap.word_at(LayerLevel::Bottom),
            ),
            word2_value,
        );

        if word2_value == 0 {
            word1_value &= !bitmap.bit_mask(LayerLevel::Middle);
            TickIndexBitmapWords::<T>::set(
                (
                    netuid,
                    LayerLevel::Middle,
                    bitmap.word_at(LayerLevel::Middle),
                ),
                word1_value,
            );
        }

        if word1_value == 0 {
            word0_value &= !bitmap.bit_mask(LayerLevel::Top);
            TickIndexBitmapWords::<T>::set(
                (netuid, LayerLevel::Top, bitmap.word_at(LayerLevel::Top)),
                word0_value,
            );
        }
    }

    pub fn find_closest_lower<T: Config>(netuid: NetUid, index: TickIndex) -> Option<TickIndex> {
        Self::find_closest::<T>(netuid, index, true)
    }

    pub fn find_closest_higher<T: Config>(netuid: NetUid, index: TickIndex) -> Option<TickIndex> {
        Self::find_closest::<T>(netuid, index, false)
    }

    fn find_closest<T: Config>(netuid: NetUid, index: TickIndex, lower: bool) -> Option<TickIndex> {
        // Check the range
        if (index < TickIndex::MIN) || (index > TickIndex::MAX) {
            return None;
        }

        // Convert to bitmap representation
        let bitmap = TickIndexBitmap::from(index);
        let mut found = false;
        let mut result: u32 = 0;

        // Layer positions from bitmap
        let layer0_word = bitmap.word_at(LayerLevel::Top);
        let layer0_bit = bitmap.bit_at(LayerLevel::Top);
        let layer1_word = bitmap.word_at(LayerLevel::Middle);
        let layer1_bit = bitmap.bit_at(LayerLevel::Middle);
        let layer2_word = bitmap.word_at(LayerLevel::Bottom);
        let layer2_bit = bitmap.bit_at(LayerLevel::Bottom);

        // Find the closest active bits in layer 0, then 1, then 2

        ///////////////
        // Level 0
        let word0 = TickIndexBitmapWords::<T>::get((netuid, LayerLevel::Top, layer0_word));
        let closest_bits_l0 =
            TickIndexBitmap::find_closest_active_bit_candidates(word0, layer0_bit, lower);

        for closest_bit_l0 in closest_bits_l0.iter() {
            ///////////////
            // Level 1
            let word1_index = TickIndexBitmap::layer_to_index(BitmapLayer::new(0, *closest_bit_l0));

            // Layer 1 words are different, shift the bit to the word edge
            let start_from_l1_bit = if word1_index < layer1_word {
                127
            } else if word1_index > layer1_word {
                0
            } else {
                layer1_bit
            };
            let word1_value =
                TickIndexBitmapWords::<T>::get((netuid, LayerLevel::Middle, word1_index));
            let closest_bits_l1 = TickIndexBitmap::find_closest_active_bit_candidates(
                word1_value,
                start_from_l1_bit,
                lower,
            );

            for closest_bit_l1 in closest_bits_l1.iter() {
                ///////////////
                // Level 2
                let word2_index =
                    TickIndexBitmap::layer_to_index(BitmapLayer::new(word1_index, *closest_bit_l1));

                // Layer 2 words are different, shift the bit to the word edge
                let start_from_l2_bit = if word2_index < layer2_word {
                    127
                } else if word2_index > layer2_word {
                    0
                } else {
                    layer2_bit
                };

                let word2_value =
                    TickIndexBitmapWords::<T>::get((netuid, LayerLevel::Bottom, word2_index));

                let closest_bits_l2 = TickIndexBitmap::find_closest_active_bit_candidates(
                    word2_value,
                    start_from_l2_bit,
                    lower,
                );

                if closest_bits_l2.len() > 0 {
                    // The active tick is found, restore its full index and return
                    let offset_found_index = TickIndexBitmap::layer_to_index(BitmapLayer::new(
                        word2_index,
                        closest_bits_l2[0],
                    ));

                    if lower {
                        if (offset_found_index > result) || (!found) {
                            result = offset_found_index;
                            found = true;
                        }
                    } else if (offset_found_index < result) || (!found) {
                        result = offset_found_index;
                        found = true;
                    }
                }
            }
        }

        if !found {
            return None;
        }

        // Convert the result offset_index back to a tick index
        TickIndex::from_offset_index(result).ok()
    }
}

/// Represents the three layers in the Uniswap V3 bitmap structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum LayerLevel {
    /// Top layer (highest level of the hierarchy)
    Top = 0,
    /// Middle layer
    Middle = 1,
    /// Bottom layer (contains the actual ticks)
    Bottom = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
struct BitmapLayer {
    word: u32,
    bit: u32,
}

impl BitmapLayer {
    pub fn new(word: u32, bit: u32) -> Self {
        Self { word, bit }
    }
}

/// A bitmap representation of a tick index position across the three-layer structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickIndexBitmap {
    /// The position in layer 0 (top layer)
    layer0: BitmapLayer,
    /// The position in layer 1 (middle layer)
    layer1: BitmapLayer,
    /// The position in layer 2 (bottom layer)
    layer2: BitmapLayer,
}

impl TickIndexBitmap {
    /// Helper function to convert a bitmap index to a (word, bit) tuple in a bitmap layer using
    /// safe methods
    ///
    /// Note: This function operates on bitmap navigation indices, NOT tick indices.
    /// It converts a flat index within the bitmap structure to a (word, bit) position.
    fn index_to_layer(index: u32) -> BitmapLayer {
        let word = index.safe_div(128);
        let bit = index.checked_rem(128).unwrap_or_default();
        BitmapLayer { word, bit }
    }

    /// Converts a position (word, bit) within a layer to a word index in the next layer down
    /// Note: This returns a bitmap navigation index, NOT a tick index
    pub fn layer_to_index(layer: BitmapLayer) -> u32 {
        layer.word.saturating_mul(128).saturating_add(layer.bit)
    }

    /// Get the mask for a bit in the specified layer
    pub fn bit_mask(&self, layer: LayerLevel) -> u128 {
        match layer {
            LayerLevel::Top => 1u128 << self.layer0.bit,
            LayerLevel::Middle => 1u128 << self.layer1.bit,
            LayerLevel::Bottom => 1u128 << self.layer2.bit,
        }
    }

    /// Get the word for the specified layer
    pub fn word_at(&self, layer: LayerLevel) -> u32 {
        match layer {
            LayerLevel::Top => self.layer0.word,
            LayerLevel::Middle => self.layer1.word,
            LayerLevel::Bottom => self.layer2.word,
        }
    }

    /// Get the bit for the specified layer
    pub fn bit_at(&self, layer: LayerLevel) -> u32 {
        match layer {
            LayerLevel::Top => self.layer0.bit,
            LayerLevel::Middle => self.layer1.bit,
            LayerLevel::Bottom => self.layer2.bit,
        }
    }

    /// Finds the closest active bit in a bitmap word, and if the active bit exactly matches the
    /// requested bit, then it finds the next one as well
    ///
    /// # Arguments
    /// * `word` - The bitmap word to search within
    /// * `bit` - The bit position to start searching from
    /// * `lower` - If true, search for lower bits (decreasing bit position),
    ///             if false, search for higher bits (increasing bit position)
    ///
    /// # Returns
    /// * Exact match: Vec with [next_bit, bit]
    /// * Non-exact match: Vec with [closest_bit]
    /// * No match: Empty Vec
    pub fn find_closest_active_bit_candidates(word: u128, bit: u32, lower: bool) -> Vec<u32> {
        let mut result = vec![];
        let mut mask: u128 = 1_u128.wrapping_shl(bit);
        let mut active_bit: u32 = bit;

        while mask > 0 {
            if mask & word != 0 {
                result.push(active_bit);
                if active_bit != bit {
                    break;
                }
            }

            mask = if lower {
                active_bit = active_bit.saturating_sub(1);
                mask.wrapping_shr(1)
            } else {
                active_bit = active_bit.saturating_add(1);
                mask.wrapping_shl(1)
            };
        }

        result
    }
}

impl From<TickIndex> for TickIndexBitmap {
    fn from(tick_index: TickIndex) -> Self {
        // Convert to offset index (internal operation only)
        let offset_index = (tick_index.get().saturating_add(TickIndex::OFFSET.get())) as u32;

        // Calculate layer positions
        let layer2 = Self::index_to_layer(offset_index);
        let layer1 = Self::index_to_layer(layer2.word);
        let layer0 = Self::index_to_layer(layer1.word);

        Self {
            layer0,
            layer1,
            layer2,
        }
    }
}

fn get_sqrt_ratio_at_tick(tick: i32) -> Result<U256, TickMathError> {
    let abs_tick = if tick < 0 {
        U256::from(tick.neg())
    } else {
        U256::from(tick)
    };

    if abs_tick > U256_MAX_TICK {
        return Err(TickMathError::TickOutOfBounds);
    }

    let mut ratio = if abs_tick & (U256_1) != U256::ZERO {
        U256::from_limbs([12262481743371124737, 18445821805675392311, 0, 0])
    } else {
        U256::from_limbs([0, 0, 1, 0])
    };

    if !(abs_tick & U256_2).is_zero() {
        ratio = (ratio * U256::from_limbs([6459403834229662010, 18444899583751176498, 0, 0])) >> 128
    }
    if !(abs_tick & U256_4).is_zero() {
        ratio =
            (ratio * U256::from_limbs([17226890335427755468, 18443055278223354162, 0, 0])) >> 128
    }
    if !(abs_tick & U256_8).is_zero() {
        ratio = (ratio * U256::from_limbs([2032852871939366096, 18439367220385604838, 0, 0])) >> 128
    }
    if !(abs_tick & U256_16).is_zero() {
        ratio =
            (ratio * U256::from_limbs([14545316742740207172, 18431993317065449817, 0, 0])) >> 128
    }
    if !(abs_tick & U256_32).is_zero() {
        ratio = (ratio * U256::from_limbs([5129152022828963008, 18417254355718160513, 0, 0])) >> 128
    }
    if !(abs_tick & U256_64).is_zero() {
        ratio = (ratio * U256::from_limbs([4894419605888772193, 18387811781193591352, 0, 0])) >> 128
    }
    if !(abs_tick & U256_128).is_zero() {
        ratio = (ratio * U256::from_limbs([1280255884321894483, 18329067761203520168, 0, 0])) >> 128
    }
    if !(abs_tick & U256_256).is_zero() {
        ratio =
            (ratio * U256::from_limbs([15924666964335305636, 18212142134806087854, 0, 0])) >> 128
    }
    if !(abs_tick & U256_512).is_zero() {
        ratio = (ratio * U256::from_limbs([8010504389359918676, 17980523815641551639, 0, 0])) >> 128
    }
    if !(abs_tick & U256_1024).is_zero() {
        ratio =
            (ratio * U256::from_limbs([10668036004952895731, 17526086738831147013, 0, 0])) >> 128
    }
    if !(abs_tick & U256_2048).is_zero() {
        ratio = (ratio * U256::from_limbs([4878133418470705625, 16651378430235024244, 0, 0])) >> 128
    }
    if !(abs_tick & U256_4096).is_zero() {
        ratio = (ratio * U256::from_limbs([9537173718739605541, 15030750278693429944, 0, 0])) >> 128
    }
    if !(abs_tick & U256_8192).is_zero() {
        ratio = (ratio * U256::from_limbs([9972618978014552549, 12247334978882834399, 0, 0])) >> 128
    }
    if !(abs_tick & U256_16384).is_zero() {
        ratio = (ratio * U256::from_limbs([10428997489610666743, 8131365268884726200, 0, 0])) >> 128
    }
    if !(abs_tick & U256_32768).is_zero() {
        ratio = (ratio * U256::from_limbs([9305304367709015974, 3584323654723342297, 0, 0])) >> 128
    }
    if !(abs_tick & U256_65536).is_zero() {
        ratio = (ratio * U256::from_limbs([14301143598189091785, 696457651847595233, 0, 0])) >> 128
    }
    if !(abs_tick & U256_131072).is_zero() {
        ratio = (ratio * U256::from_limbs([7393154844743099908, 26294789957452057, 0, 0])) >> 128
    }
    if !(abs_tick & U256_262144).is_zero() {
        ratio = (ratio * U256::from_limbs([2209338891292245656, 37481735321082, 0, 0])) >> 128
    }
    if !(abs_tick & U256_524288).is_zero() {
        ratio = (ratio * U256::from_limbs([10518117631919034274, 76158723, 0, 0])) >> 128
    }

    if tick > 0 {
        ratio = U256::MAX / ratio;
    }

    Ok((ratio >> 32)
        + if (ratio.wrapping_rem(U256_1 << 32)).is_zero() {
            U256::ZERO
        } else {
            U256_1
        })
}

fn get_tick_at_sqrt_ratio(sqrt_price_x_96: U256) -> Result<i32, TickMathError> {
    if !(sqrt_price_x_96 >= MIN_SQRT_RATIO && sqrt_price_x_96 < MAX_SQRT_RATIO) {
        return Err(TickMathError::SqrtPriceOutOfBounds);
    }

    let ratio: U256 = sqrt_price_x_96.shl(32);
    let mut r = ratio;
    let mut msb = U256::ZERO;

    let mut f = if r > U256::from_limbs([18446744073709551615, 18446744073709551615, 0, 0]) {
        U256_1.shl(U256_7)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256::from_limbs([18446744073709551615, 0, 0, 0]) {
        U256_1.shl(U256_6)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256::from_limbs([4294967295, 0, 0, 0]) {
        U256_1.shl(U256_5)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256::from_limbs([65535, 0, 0, 0]) {
        U256_1.shl(U256_4)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256_255 {
        U256_1.shl(U256_3)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256_15 {
        U256_1.shl(U256_2)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256_3 {
        U256_1.shl(U256_1)
    } else {
        U256::ZERO
    };
    msb = msb.bitor(f);
    r = r.shr(f);

    f = if r > U256_1 { U256_1 } else { U256::ZERO };

    msb = msb.bitor(f);

    r = if msb >= U256_128 {
        ratio.shr(msb - U256_127)
    } else {
        ratio.shl(U256_127 - msb)
    };

    let mut log_2: I256 = (I256::from_raw(msb) - I256::from_limbs([128, 0, 0, 0])).shl(64);

    for i in (51..=63).rev() {
        r = r.overflowing_mul(r).0.shr(U256_127);
        let f: U256 = r.shr(128);
        log_2 = log_2.bitor(I256::from_raw(f.shl(i)));

        r = r.shr(f);
    }

    r = r.overflowing_mul(r).0.shr(U256_127);
    let f: U256 = r.shr(128);
    log_2 = log_2.bitor(I256::from_raw(f.shl(50)));

    let log_sqrt10001 = log_2.wrapping_mul(SQRT_10001);

    let tick_low = ((log_sqrt10001 - TICK_LOW) >> 128_u8).low_i32();

    let tick_high = ((log_sqrt10001 + TICK_HIGH) >> 128_u8).low_i32();

    let tick = if tick_low == tick_high {
        tick_low
    } else if get_sqrt_ratio_at_tick(tick_high)? <= sqrt_price_x_96 {
        tick_high
    } else {
        tick_low
    };

    Ok(tick)
}

/// Convert U256 to U64F64
///
/// # Arguments
/// * `value` - The U256 value in Q64.96 format
///
/// # Returns
/// * `Result<U64F64, &'static str>` - Converted value or error if too large
fn u256_to_u64f64(value: U256, source_fractional_bits: u32) -> Result<U64F64, TickMathError> {
    if value > U256::from(u128::MAX) {
        return Err(TickMathError::ConversionError);
    }

    let mut value: u128 = value
        .try_into()
        .map_err(|_| TickMathError::ConversionError)?;

    // Adjust to 64 fractional bits (U64F64 format)
    if source_fractional_bits < 64 {
        // Shift left to add more fractional bits
        value = value
            .checked_shl(64 - source_fractional_bits)
            .ok_or(TickMathError::Overflow)?;
    } else if source_fractional_bits > 64 {
        // Shift right to remove excess fractional bits
        value = value >> (source_fractional_bits - 64);
    }

    Ok(U64F64::from_bits(value))
}

/// Convert U64F64 to U256
///
/// # Arguments
/// * `value` - The U64F64 value to convert
/// * `target_fractional_bits` - Number of fractional bits in the target U256 format
///
/// # Returns
/// * `U256` - Converted value
fn u64f64_to_u256(value: U64F64, target_fractional_bits: u32) -> U256 {
    let mut bits = value.to_bits();

    // Adjust to target fractional bits
    if target_fractional_bits < 64 {
        // Shift right to remove excess fractional bits
        bits = bits >> (64 - target_fractional_bits);
    } else if target_fractional_bits > 64 {
        // Shift left to add more fractional bits
        bits = bits << (target_fractional_bits - 64);
    }

    // Create U256
    U256::from(bits)
}

/// Convert U256 in Q64.96 format (Uniswap's sqrt price format) to U64F64
fn u256_q64_96_to_u64f64(value: U256) -> Result<U64F64, TickMathError> {
    u256_to_u64f64(value, 96)
}

/// Convert U64F64 to U256 in Q64.96 format (Uniswap's sqrt price format)
fn u64f64_to_u256_q64_96(value: U64F64) -> U256 {
    u64f64_to_u256(value, 96)
}

#[derive(Debug, PartialEq, Eq)]
pub enum TickMathError {
    TickOutOfBounds,
    SqrtPriceOutOfBounds,
    ConversionError,
    Overflow,
    DivisionByZero,
}

impl fmt::Display for TickMathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
			Self::TickOutOfBounds => f.write_str("The given tick is outside of the minimum/maximum values."),
			Self::SqrtPriceOutOfBounds =>f.write_str("Second inequality must be < because the price can never reach the price at the max tick"),
			Self::ConversionError => f.write_str("Error converting from one number type into another"),
			Self::Overflow => f.write_str("Number overflow in arithmetic operation"),
			Self::DivisionByZero => f.write_str("Division by zero is not allowed")
		}
    }
}

impl Error for TickMathError {}

#[cfg(test)]
mod tests {
    use std::{ops::Sub, str::FromStr};

    use safe_math::FixedExt;

    use super::*;

    #[test]
    fn test_get_sqrt_ratio_at_tick_bounds() {
        // the function should return an error if the tick is out of bounds
        if let Err(err) = get_sqrt_ratio_at_tick(MIN_TICK - 1) {
            assert!(matches!(err, TickMathError::TickOutOfBounds));
        } else {
            panic!("get_qrt_ratio_at_tick did not respect lower tick bound")
        }
        if let Err(err) = get_sqrt_ratio_at_tick(MAX_TICK + 1) {
            assert!(matches!(err, TickMathError::TickOutOfBounds));
        } else {
            panic!("get_qrt_ratio_at_tick did not respect upper tick bound")
        }
    }

    #[test]
    fn test_get_sqrt_ratio_at_tick_values() {
        // test individual values for correct results
        assert_eq!(
            get_sqrt_ratio_at_tick(MIN_TICK).unwrap(),
            U256::from(4295128739u64),
            "sqrt ratio at min incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(MIN_TICK + 1).unwrap(),
            U256::from(4295343490u64),
            "sqrt ratio at min + 1 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(MAX_TICK - 1).unwrap(),
            U256::from_str("1461373636630004318706518188784493106690254656249").unwrap(),
            "sqrt ratio at max - 1 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(MAX_TICK).unwrap(),
            U256::from_str("1461446703485210103287273052203988822378723970342").unwrap(),
            "sqrt ratio at max incorrect"
        );
        // checking hard coded values against solidity results
        assert_eq!(
            get_sqrt_ratio_at_tick(50).unwrap(),
            U256::from(79426470787362580746886972461u128),
            "sqrt ratio at 50 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(100).unwrap(),
            U256::from(79625275426524748796330556128u128),
            "sqrt ratio at 100 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(250).unwrap(),
            U256::from(80224679980005306637834519095u128),
            "sqrt ratio at 250 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(500).unwrap(),
            U256::from(81233731461783161732293370115u128),
            "sqrt ratio at 500 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(1000).unwrap(),
            U256::from(83290069058676223003182343270u128),
            "sqrt ratio at 1000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(2500).unwrap(),
            U256::from(89776708723587163891445672585u128),
            "sqrt ratio at 2500 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(3000).unwrap(),
            U256::from(92049301871182272007977902845u128),
            "sqrt ratio at 3000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(4000).unwrap(),
            U256::from(96768528593268422080558758223u128),
            "sqrt ratio at 4000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(5000).unwrap(),
            U256::from(101729702841318637793976746270u128),
            "sqrt ratio at 5000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(50000).unwrap(),
            U256::from(965075977353221155028623082916u128),
            "sqrt ratio at 50000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(150000).unwrap(),
            U256::from(143194173941309278083010301478497u128),
            "sqrt ratio at 150000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(250000).unwrap(),
            U256::from(21246587762933397357449903968194344u128),
            "sqrt ratio at 250000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(500000).unwrap(),
            U256::from_str("5697689776495288729098254600827762987878").unwrap(),
            "sqrt ratio at 500000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(738203).unwrap(),
            U256::from_str("847134979253254120489401328389043031315994541").unwrap(),
            "sqrt ratio at 738203 incorrect"
        );
    }

    #[test]
    fn test_get_tick_at_sqrt_ratio() {
        //throws for too low
        let result = get_tick_at_sqrt_ratio(MIN_SQRT_RATIO.sub(U256_1));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Second inequality must be < because the price can never reach the price at the max tick"
        );

        //throws for too high
        let result = get_tick_at_sqrt_ratio(MAX_SQRT_RATIO);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Second inequality must be < because the price can never reach the price at the max tick"
        );

        //ratio of min tick
        let result = get_tick_at_sqrt_ratio(MIN_SQRT_RATIO).unwrap();
        assert_eq!(result, MIN_TICK);

        //ratio of min tick + 1
        let result = get_tick_at_sqrt_ratio(U256::from_str("4295343490").unwrap()).unwrap();
        assert_eq!(result, MIN_TICK + 1);
    }

    #[test]
    fn test_roundtrip() {
        for tick_index in [
            MIN_TICK + 1, // we can't use extremes because of rounding during roundtrip conversion
            -1000,
            -100,
            -10,
            -4,
            -2,
            0,
            2,
            4,
            10,
            100,
            1000,
            MAX_TICK - 1,
        ]
        .iter()
        {
            let sqrt_price = get_sqrt_ratio_at_tick(*tick_index).unwrap();
            let round_trip_tick_index = get_tick_at_sqrt_ratio(sqrt_price).unwrap();
            assert_eq!(round_trip_tick_index, *tick_index);
        }
    }

    #[test]
    fn test_u256_to_u64f64_q64_96() {
        // Test tick 0 (sqrt price = 1.0 * 2^96)
        let tick0_sqrt_price = U256::from(1u128 << 96);
        let fixed_price = u256_q64_96_to_u64f64(tick0_sqrt_price).unwrap();

        // Should be 1.0 in U64F64
        assert_eq!(fixed_price, U64F64::from_num(1.0));

        // Round trip back to U256 Q64.96
        let back_to_u256 = u64f64_to_u256_q64_96(fixed_price);
        assert_eq!(back_to_u256, tick0_sqrt_price);
    }

    #[test]
    fn test_u256_with_other_formats() {
        // Test with a value that has 32 fractional bits
        let value_32frac = U256::from(123456789u128 << 32); // 123456789.0 in Q96.32
        let fixed_value = u256_to_u64f64(value_32frac, 32).unwrap();

        // Should be 123456789.0 in U64F64
        assert_eq!(fixed_value, U64F64::from_num(123456789.0));

        // Round trip back to U256 with 32 fractional bits
        let back_to_u256 = u64f64_to_u256(fixed_value, 32);
        assert_eq!(back_to_u256, value_32frac);
    }
    #[test]
    fn test_tick_index_to_sqrt_price() {
        let tick_spacing = SqrtPrice::from_num(1.0001);

        // check tick bounds
        assert_eq!(
            TickIndex(MIN_TICK).try_to_sqrt_price(),
            Err(TickMathError::TickOutOfBounds)
        );

        assert_eq!(
            TickIndex(MAX_TICK).try_to_sqrt_price(),
            Err(TickMathError::TickOutOfBounds),
        );

        // At tick index 0, the sqrt price should be 1.0
        let sqrt_price = TickIndex(0).try_to_sqrt_price().unwrap();
        assert_eq!(sqrt_price, SqrtPrice::from_num(1.0));

        let sqrt_price = TickIndex(2).try_to_sqrt_price().unwrap();
        assert!(sqrt_price.abs_diff(tick_spacing) < SqrtPrice::from_num(1e-10));

        let sqrt_price = TickIndex(4).try_to_sqrt_price().unwrap();
        // Calculate the expected value: (1 + TICK_SPACING/1e9 + 1.0)^2
        let expected = tick_spacing * tick_spacing;
        assert!(sqrt_price.abs_diff(expected) < SqrtPrice::from_num(1e-10));

        // Test with tick index 10
        let sqrt_price = TickIndex(10).try_to_sqrt_price().unwrap();
        // Calculate the expected value: (1 + TICK_SPACING/1e9 + 1.0)^5
        let expected = tick_spacing.checked_pow(5).unwrap();
        assert!(
            sqrt_price.abs_diff(expected) < SqrtPrice::from_num(1e-10),
            "diff: {}",
            sqrt_price.abs_diff(expected),
        );
    }

    #[test]
    fn test_sqrt_price_to_tick_index() {
        let tick_spacing = SqrtPrice::from_num(1.0001);
        let tick_index = TickIndex::try_from_sqrt_price(SqrtPrice::from_num(1.0)).unwrap();
        assert_eq!(tick_index, TickIndex::new_unchecked(0));

        // Test with sqrt price equal to tick_spacing_tao (should be tick index 2)
        let tick_index = TickIndex::try_from_sqrt_price(tick_spacing).unwrap();
        assert_eq!(tick_index, TickIndex::new_unchecked(2));

        // Test with sqrt price equal to tick_spacing_tao^2 (should be tick index 4)
        let sqrt_price = tick_spacing * tick_spacing;
        let tick_index = TickIndex::try_from_sqrt_price(sqrt_price).unwrap();
        assert_eq!(tick_index, TickIndex::new_unchecked(4));

        // Test with sqrt price equal to tick_spacing_tao^5 (should be tick index 10)
        let sqrt_price = tick_spacing.checked_pow(5).unwrap();
        let tick_index = TickIndex::try_from_sqrt_price(sqrt_price).unwrap();
        assert_eq!(tick_index, TickIndex::new_unchecked(10));
    }

    #[test]
    fn test_roundtrip_tick_index_sqrt_price() {
        for i32_value in [
            MIN_TICK / 2,
            -1000,
            -100,
            -10,
            -4,
            -2,
            0,
            2,
            4,
            10,
            100,
            1000,
            MAX_TICK / 2,
        ]
        .iter()
        {
            let tick_index = TickIndex(*i32_value);
            let sqrt_price = tick_index.try_to_sqrt_price().unwrap();
            let round_trip_tick_index = TickIndex::try_from_sqrt_price(sqrt_price).unwrap();
            assert_eq!(round_trip_tick_index, tick_index);
        }
    }

    #[test]
    fn test_from_offset_index() {
        // Test various tick indices
        for i32_value in [
            MIN_TICK / 2,
            -1000,
            -100,
            -10,
            0,
            10,
            100,
            1000,
            MAX_TICK / 2,
        ] {
            let original_tick = TickIndex::new_unchecked(i32_value);

            // Calculate the offset index (adding OFFSET)
            let offset_index = (i32_value + TickIndex::OFFSET.get()) as u32;

            // Convert back from offset index to tick index
            let roundtrip_tick = TickIndex::from_offset_index(offset_index).unwrap();

            // Check that we get the same tick index back
            assert_eq!(original_tick, roundtrip_tick);
        }

        // Test out of bounds values
        let too_large = (TickIndex::MAX.get() + TickIndex::OFFSET.get() + 1) as u32;
        assert!(TickIndex::from_offset_index(too_large).is_err());
    }
}
