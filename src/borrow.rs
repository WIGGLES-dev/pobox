pub trait BorrowMaskBitField {
    type BITFIELD;

    fn bitfield(&self) -> (&Self::BITFIELD, &Self::BITFIELD);
    fn bitfield_mut(&mut self) -> (&mut Self::BITFIELD, &mut Self::BITFIELD);

    fn is_borrowed(&self) -> bool {
        false
    }
    fn is_borrowed_mut(&self) -> bool {
        false
    }

    fn borrow(&mut self) {}

    fn borrow_mut(&mut self) {}

    fn unborrow(&mut self) {}
}

pub struct BitfieldState<const LEN: usize>([usize; LEN], [usize; LEN]);

impl<const LEN: usize> BitfieldState<LEN> {
    const fn compare(&self) {}
}

impl<const LEN: usize> BorrowMaskBitField for BitfieldState<LEN> {
    type BITFIELD = [usize; LEN];

    fn bitfield(&self) -> (&Self::BITFIELD, &Self::BITFIELD) {
        (&self.0, &self.1)
    }
    fn bitfield_mut(&mut self) -> (&mut Self::BITFIELD, &mut Self::BITFIELD) {
        (&mut self.0, &mut self.1)
    }
}

pub unsafe trait BorrowMask {
    const BITFIELD_LEN: usize;
    const BITMASK_LEN: usize;
    type Bitfield: BorrowMaskBitField;
}

pub const fn bitfield_len<const FIELDS: usize>() -> usize {
    FIELDS / usize::MAX
}

pub const fn bitfield<const FIELDS: usize>() -> [usize; FIELDS] {
    [0_usize; FIELDS]
}

#[cfg(feature = "simd")]
pub mod simd {
    use std::simd::{LaneCount, Mask, Simd, SupportedLaneCount, cmp::SimdPartialEq};

    /// determine whether or not the resource can be accessed with the current aliasing
    pub fn can_borrow<const BITFIELD_LEN: usize>(
        // current mut borrow
        mask1: Simd<u32, BITFIELD_LEN>,
        // current non mut borrow
        mask2: Simd<u32, BITFIELD_LEN>,
        // requested mut borrow
        mask3: Simd<u32, BITFIELD_LEN>,
        // requested non mut borrow
        mask4: Simd<u32, BITFIELD_LEN>,
    ) -> bool
    where
        LaneCount<BITFIELD_LEN>: SupportedLaneCount,
    {
        let request_any = mask3 | mask4;
        let conflict1 = mask1 & request_any;
        let conflict2 = mask2 & mask3;

        let conflict1_mask: Mask<_, BITFIELD_LEN> = conflict1.simd_ne(Simd::splat(0));
        let conflict2_mask: Mask<_, BITFIELD_LEN> = conflict2.simd_ne(Simd::splat(0));

        conflict1_mask.any() || conflict2_mask.any()
    }
}
