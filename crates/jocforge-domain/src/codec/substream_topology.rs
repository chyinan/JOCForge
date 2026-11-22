// pattern: Functional Core
//! Channel ownership of six-block E-AC-3 access units.
use crate::profile::JocReconstructionLayout;
/// Number of audio blocks per syncframe within a six-block programme interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPartition {
    Six,
    Three,
    Two,
    One,
}
impl std::str::FromStr for BlockPartition {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "6" => Ok(Self::Six),
            "3+3" => Ok(Self::Three),
            "2+2+2" => Ok(Self::Two),
            "1x6" | "1+1+1+1+1+1" => Ok(Self::One),
            _ => Err("expected 6, 3+3, 2+2+2, or 1x6 (1+1+1+1+1+1)"),
        }
    }
}
impl BlockPartition {
    pub(super) const fn blocks(self) -> usize {
        match self {
            Self::Six => 6,
            Self::Three => 3,
            Self::Two => 2,
            Self::One => 1,
        }
    }
    pub(super) const fn code(self) -> u8 {
        match self {
            Self::Six => 3,
            Self::Three => 2,
            Self::Two => 1,
            Self::One => 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmdfLocation {
    SkipField,
    FrameEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LfeOwner {
    None,
    Independent,
    Dependent,
    ReplaceIndependent,
}

#[derive(Clone, Debug)]
pub(super) struct FramePlan {
    pub(super) channels: Vec<usize>,
    pub(super) map: u16,
    pub(super) lfe: bool,
}

#[derive(Clone, Debug)]
pub struct SubstreamPlan {
    ac3_core: bool,
    cmaf_rate: Option<u32>,
    pub(super) emdf_location: EmdfLocation,
    pub(super) frames: Vec<FramePlan>,
    pub(super) partition: BlockPartition,
    layout: JocReconstructionLayout,
}

impl SubstreamPlan {
    /// Canonical Table 47 speaker groups in Table E.1.4 chanmap order.
    pub fn new(layout: JocReconstructionLayout, count: u8, lfe: LfeOwner) -> Result<Self, String> {
        if !(layout.uses_five_carriers() || layout.uses_seven_carriers()) {
            return Err("unsupported JOC layout".into());
        }
        let extra = if layout.uses_height_pair() {
            0x0010
        } else if layout.uses_seven_carriers() {
            0x0200
        } else {
            0
        };
        let max = if extra == 0 { 5 } else { 6 };
        let min = if extra == 0 { 0 } else { 1 };
        if count == 0 && matches!(lfe, LfeOwner::Dependent | LfeOwner::ReplaceIndependent) {
            return Err("dependent LFE requires at least one dependent substream".into());
        }
        if !(min..=max).contains(&count) {
            return Err(format!(
                "this profile supports {min}..={max} dependent substreams"
            ));
        }
        let singles = [0x8000, 0x4000, 0x2000, 0x1000, 0x0800];
        let maps = match (extra, count) {
            (0, 1) => vec![0xc000],
            (0, 2) => vec![0xc000, 0x3000],
            (0, n) => singles[..usize::from(n)].to_vec(),
            (pair, 1) => vec![pair],
            (pair, 2) => vec![0xc000, pair],
            (pair, 3) => vec![0xc000, 0x3000, pair],
            (pair, n) => {
                let mut maps = singles[..usize::from(n - 1)].to_vec();
                maps.push(pair);
                maps
            }
        };
        let mut frames = vec![FramePlan {
            channels: vec![0, 1, 2, 3, 4],
            map: 0,
            lfe: matches!(lfe, LfeOwner::Independent | LfeOwner::ReplaceIndependent),
        }];
        for (id, map) in maps.into_iter().enumerate() {
            let channels = if extra != 0 && map == extra {
                vec![5, 6]
            } else {
                singles
                    .iter()
                    .enumerate()
                    .filter_map(|(lane, mask)| (mask & map != 0).then_some(lane))
                    .collect()
            };
            let has_lfe =
                id == 0 && matches!(lfe, LfeOwner::Dependent | LfeOwner::ReplaceIndependent);
            frames.push(FramePlan {
                channels,
                map: map | u16::from(has_lfe),
                lfe: has_lfe,
            });
        }
        Ok(Self {
            frames,
            layout,
            ac3_core: false,
            cmaf_rate: None,
            emdf_location: EmdfLocation::SkipField,
            partition: BlockPartition::Six,
        })
    }

    pub fn cmaf(layout: JocReconstructionLayout, rate: u32) -> Result<Self, String> {
        Self::cmaf_with_lfe(layout, rate, LfeOwner::None)
    }

    pub fn cmaf_with_lfe(
        layout: JocReconstructionLayout,
        rate: u32,
        lfe: LfeOwner,
    ) -> Result<Self, String> {
        if lfe == LfeOwner::ReplaceIndependent {
            return Err("CMAF supports absent, independent or supplemental dependent LFE".into());
        }
        if !matches!(rate, 1024 | 1280 | 1408 | 1536 | 1664) {
            return Err("CMAF total rate must be 1024, 1280, 1408, 1536 or 1664 kbps".into());
        }
        let mut plan = Self::new(layout, 1, lfe)?.with_emdf_location(EmdfLocation::FrameEnd)?;
        plan.cmaf_rate = Some(rate);
        Ok(plan)
    }
    pub fn cmaf_rate(&self) -> Option<u32> {
        self.cmaf_rate
    }
    pub fn with_ac3_core(mut self) -> Result<Self, String> {
        if self.cmaf_rate.is_some()
            || self.frames.len() != 2
            || self.partition != BlockPartition::Six
            || self.emdf_location != EmdfLocation::FrameEnd
        {
            return Err("legacy-ac3 requires six blocks, one D0 and frame-end EMDF".into());
        }
        self.ac3_core = true;
        Ok(self)
    }

    pub fn ac3_core_lfe(&self) -> Option<bool> {
        self.ac3_core.then_some(self.frames[0].lfe)
    }
    pub fn with_emdf_location(mut self, location: EmdfLocation) -> Result<Self, String> {
        if location == EmdfLocation::FrameEnd && self.frames.len() < 2 {
            return Err("ETSI frame-end carriage requires a dependent substream".into());
        }
        self.emdf_location = location;
        Ok(self)
    }

    pub fn with_partition(mut self, partition: BlockPartition) -> Self {
        self.partition = partition;
        self
    }

    pub fn layout(&self) -> JocReconstructionLayout {
        self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac3_core_requires_one_six_block_dependent_with_frame_end_metadata() {
        for (layout, channel_map) in [
            (JocReconstructionLayout::FivePointOneWithoutPhase, 0xc000),
            (
                JocReconstructionLayout::ExistingSelectedFivePointOne,
                0xc000,
            ),
            (JocReconstructionLayout::Flat7x, 0x0200),
            (JocReconstructionLayout::FivePointOnePlusHeight, 0x0010),
            (JocReconstructionLayout::HeightPairWithPhase, 0x0010),
        ] {
            for owner in [
                LfeOwner::None,
                LfeOwner::Independent,
                LfeOwner::Dependent,
                LfeOwner::ReplaceIndependent,
            ] {
                let plan = SubstreamPlan::new(layout, 1, owner).unwrap();
                assert!(plan.clone().with_ac3_core().is_err());
                let plan = plan.with_emdf_location(EmdfLocation::FrameEnd).unwrap();
                assert!(
                    plan.clone()
                        .with_partition(BlockPartition::Three)
                        .with_ac3_core()
                        .is_err()
                );
                let plan = plan.with_ac3_core().unwrap();
                assert_eq!(
                    plan.ac3_core_lfe(),
                    Some(matches!(
                        owner,
                        LfeOwner::Independent | LfeOwner::ReplaceIndependent
                    ))
                );
                assert_eq!(
                    plan.frames[1].map,
                    channel_map
                        | u16::from(matches!(
                            owner,
                            LfeOwner::Dependent | LfeOwner::ReplaceIndependent
                        ))
                );
            }
        }
    }

    #[test]
    fn all_canonical_topologies_have_disjoint_dependent_maps_and_one_final_owner() {
        for layout in [
            JocReconstructionLayout::FivePointOneWithoutPhase,
            JocReconstructionLayout::Flat7x,
            JocReconstructionLayout::FivePointOnePlusHeight,
            JocReconstructionLayout::ExistingSelectedFivePointOne,
            JocReconstructionLayout::HeightPairWithPhase,
        ] {
            let max = if layout.uses_seven_carriers() { 6 } else { 5 };
            for count in 0..=7 {
                for owner in [
                    LfeOwner::None,
                    LfeOwner::Independent,
                    LfeOwner::Dependent,
                    LfeOwner::ReplaceIndependent,
                ] {
                    let in_d0 = matches!(owner, LfeOwner::Dependent | LfeOwner::ReplaceIndependent);
                    let valid =
                        count <= max && (count > 0 || (!layout.uses_seven_carriers() && !in_d0));
                    let plan = SubstreamPlan::new(layout, count, owner);
                    assert_eq!(plan.is_ok(), valid);
                    if let Ok(plan) = plan {
                        assert_eq!(plan.frames.len(), usize::from(count) + 1);
                        assert_eq!(
                            plan.frames[0].lfe,
                            matches!(owner, LfeOwner::Independent | LfeOwner::ReplaceIndependent)
                        );
                        let mut seen = 0;
                        for (id, frame) in plan.frames[1..].iter().enumerate() {
                            assert_eq!(frame.lfe, id == 0 && in_d0);
                            assert_eq!(frame.map & 1, u16::from(frame.lfe));
                            assert_eq!(frame.map & seen, 0);
                            seen |= frame.map;
                        }
                        if layout.uses_seven_carriers() {
                            assert_eq!(plan.frames.last().unwrap().channels, [5, 6]);
                        }
                    }
                }
            }
        }
    }
}
