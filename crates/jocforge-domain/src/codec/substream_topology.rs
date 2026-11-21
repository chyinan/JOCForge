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
            partition: BlockPartition::Six,
        })
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
