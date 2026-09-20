//! Attach chunk-encode % onto pipeline steps.

use shared::{PipelineStep, PipelineStepName, PipelineStepState};

use crate::ports::pipeline::EncodeProgress;

#[derive(Debug, Clone)]
pub struct PipelineStepView {
    pub step: PipelineStep,
    pub progress_pct: Option<u8>,
    pub chunks_done: Option<i32>,
    pub chunks_total: Option<i32>,
}

pub fn attach_progress(steps: Vec<PipelineStep>, progress: &EncodeProgress) -> Vec<PipelineStepView> {
    steps
        .into_iter()
        .map(|step| {
            let p = progress_for(step.step, step.state, progress);
            PipelineStepView {
                progress_pct: p.map(|x| x.0),
                chunks_done: p.map(|x| x.1),
                chunks_total: p.map(|x| x.2),
                step,
            }
        })
        .collect()
}

fn progress_for(
    name: PipelineStepName,
    state: PipelineStepState,
    progress: &EncodeProgress,
) -> Option<(u8, i32, i32)> {
    if !tracks_encode(name) {
        return None;
    }
    // A rung IMS has disabled (IMS_ENABLE_720P/1080P) never gets `track_abr`
    // called on it, so its step row stays `pending` forever. Don't attach
    // the packed ladder's shared chunk progress to a rung that was never
    // actually part of the encode.
    if is_abr_rung(name) && state == PipelineStepState::Pending {
        return None;
    }
    if progress.rungs.is_empty() {
        return match state {
            PipelineStepState::Done => Some((100, 1, 1)),
            _ => None,
        };
    }
    let (done, total) = counts_for(name, progress)?;
    if total <= 0 {
        return None;
    }
    let pct = ((done as u64 * 100) / total as u64) as u8;
    Some((pct, done, total))
}

fn tracks_encode(name: PipelineStepName) -> bool {
    matches!(
        name,
        PipelineStepName::Process
            | PipelineStepName::Hls360
            | PipelineStepName::Hls720
            | PipelineStepName::Hls1080
    )
}

fn is_abr_rung(name: PipelineStepName) -> bool {
    matches!(
        name,
        PipelineStepName::Hls360 | PipelineStepName::Hls720 | PipelineStepName::Hls1080
    )
}

fn counts_for(name: PipelineStepName, progress: &EncodeProgress) -> Option<(i32, i32)> {
    if progress.pack_ladder || name == PipelineStepName::Process {
        let done: i32 = progress.rungs.iter().map(|r| r.done).sum();
        let total: i32 = progress.rungs.iter().map(|r| r.total).sum();
        return Some((done, total));
    }
    let rung = match name {
        PipelineStepName::Hls360 => 0,
        PipelineStepName::Hls720 => 1,
        PipelineStepName::Hls1080 => 2,
        _ => return None,
    };
    progress
        .rungs
        .iter()
        .find(|r| r.rung == rung)
        .map(|r| (r.done, r.total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::pipeline::RungProgress;
    use shared::VideoId;
    use uuid::Uuid;

    fn step(name: PipelineStepName, state: PipelineStepState) -> PipelineStep {
        PipelineStep {
            video_id: VideoId::from(Uuid::nil()),
            step: name,
            state,
            detail: None,
            error: None,
            started_at: None,
            finished_at: None,
        }
    }

    #[test]
    fn packed_ladder_same_pct_on_all_rungs() {
        let progress = EncodeProgress {
            pack_ladder: true,
            rungs: vec![RungProgress {
                rung: 0,
                done: 25,
                total: 100,
            }],
        };
        let views = attach_progress(
            vec![
                step(PipelineStepName::Hls360, PipelineStepState::Running),
                step(PipelineStepName::Hls720, PipelineStepState::Running),
                step(PipelineStepName::Hls1080, PipelineStepState::Running),
                step(PipelineStepName::Process, PipelineStepState::Running),
            ],
            &progress,
        );
        for v in views {
            assert_eq!(v.progress_pct, Some(25));
            assert_eq!(v.chunks_done, Some(25));
            assert_eq!(v.chunks_total, Some(100));
        }
    }

    #[test]
    fn per_rung_jobs_differ() {
        let progress = EncodeProgress {
            pack_ladder: false,
            rungs: vec![
                RungProgress {
                    rung: 0,
                    done: 10,
                    total: 10,
                },
                RungProgress {
                    rung: 1,
                    done: 5,
                    total: 10,
                },
                RungProgress {
                    rung: 2,
                    done: 0,
                    total: 10,
                },
            ],
        };
        let views = attach_progress(
            vec![
                step(PipelineStepName::Hls360, PipelineStepState::Done),
                step(PipelineStepName::Hls720, PipelineStepState::Running),
                step(PipelineStepName::Hls1080, PipelineStepState::Running),
            ],
            &progress,
        );
        assert_eq!(views[0].progress_pct, Some(100));
        assert_eq!(views[1].progress_pct, Some(50));
        assert_eq!(views[2].progress_pct, Some(0));
    }

    #[test]
    fn disabled_rung_stuck_pending_gets_no_progress() {
        let progress = EncodeProgress {
            pack_ladder: true,
            rungs: vec![RungProgress {
                rung: 0,
                done: 3,
                total: 49,
            }],
        };
        let views = attach_progress(
            vec![
                step(PipelineStepName::Hls360, PipelineStepState::Running),
                step(PipelineStepName::Hls720, PipelineStepState::Running),
                step(PipelineStepName::Hls1080, PipelineStepState::Pending),
            ],
            &progress,
        );
        assert_eq!(views[0].progress_pct, Some(6));
        assert_eq!(views[1].progress_pct, Some(6));
        assert_eq!(views[2].progress_pct, None);
        assert_eq!(views[2].chunks_done, None);
        assert_eq!(views[2].chunks_total, None);
    }
}
