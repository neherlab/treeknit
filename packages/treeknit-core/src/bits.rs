//! Masked set operations on leaf bitsets.
//!
//! Clades and configurations are `FixedBitSet`s over the same index space, so all
//! operations below assume equal lengths and work block-wise.

use fixedbitset::FixedBitSet;

pub type Bits = FixedBitSet;

#[inline]
fn blocks<'a>(a: &'a Bits, b: &'a Bits, m: &'a Bits) -> impl Iterator<Item = (usize, usize, usize)> + 'a {
  debug_assert!(a.len() == b.len() && a.len() == m.len());
  a.as_slice()
    .iter()
    .zip(b.as_slice())
    .zip(m.as_slice())
    .map(|((x, y), z)| (*x, *y, *z))
}

/// `a ∩ m == b ∩ m`
#[inline]
pub fn eq_on(a: &Bits, b: &Bits, m: &Bits) -> bool {
  blocks(a, b, m).all(|(x, y, z)| (x ^ y) & z == 0)
}

/// `a ∩ m ⊆ b`
#[inline]
pub fn subset_on(a: &Bits, b: &Bits, m: &Bits) -> bool {
  blocks(a, b, m).all(|(x, y, z)| x & !y & z == 0)
}

/// `a ∩ b ∩ m == ∅`; the oracle of the clade range methods of the split graph.
#[cfg(test)]
#[inline]
pub fn disjoint_on(a: &Bits, b: &Bits, m: &Bits) -> bool {
  blocks(a, b, m).all(|(x, y, z)| x & y & z == 0)
}

/// `|a ∩ m| < 2`, i.e. `a` is a leaf or empty split on `m`.
#[inline]
pub fn trivial_on(a: &Bits, m: &Bits) -> bool {
  let mut n = 0_u32;
  for (x, z) in a.as_slice().iter().zip(m.as_slice()) {
    n += (x & z).count_ones();
    if n > 1 {
      return false;
    }
  }
  true
}

pub fn from_iter(n: usize, it: impl IntoIterator<Item = usize>) -> Bits {
  let mut b = Bits::with_capacity(n);
  b.extend(it);
  b
}

pub fn full(n: usize) -> Bits {
  let mut b = Bits::with_capacity(n);
  b.insert_range(..);
  b
}
