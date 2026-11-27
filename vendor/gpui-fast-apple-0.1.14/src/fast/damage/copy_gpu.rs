// SPDX-License-Identifier: Apache-2.0
//! Optional GPU timestamps for the original retained-to-drawable render pass.
use block2::RcBlock;
use objc2::{
    class, msg_send,
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
};
use objc2_foundation::{NSArray, NSData, NSError, NSRange, NSString};
use objc2_metal::{
    MTLCommandBuffer, MTLCounterSamplingPoint, MTLDevice, MTLRenderPassDescriptor, MTLStorageMode,
};
use std::{
    ptr::NonNull,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

thread_local! { static PHASE: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) }; }
pub(super) struct ShiftProbe(Option<&'static str>, bool);
impl ShiftProbe {
    pub(super) fn new(shift: bool) -> Self {
        static ON: OnceLock<bool> = OnceLock::new();
        let enabled =
            shift && *ON.get_or_init(|| std::env::var("RUSTUI_G3B_GPU").as_deref() == Ok("1"));
        Self(PHASE.replace(enabled.then_some("shift_copy")), enabled)
    }
    pub(super) fn active(&self) -> bool {
        self.1
    }
    pub(super) fn finish(&self, cmd: &ProtocolObject<dyn MTLCommandBuffer>, counts: [usize; 3]) {
        if !self.1 {
            return;
        }
        let done = RcBlock::new(move |cb: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            let start: f64 = unsafe { msg_send![cb.as_ref(), GPUStartTime] };
            let end: f64 = unsafe { msg_send![cb.as_ref(), GPUEndTime] };
            super::log::line(format_args!(
                "RUSTUI_SHIFT_GPU_EXPECT {start:.9} {end:.9} {} {} {}",
                counts[0], counts[1], counts[2]
            ));
        });
        unsafe { cmd.addCompletedHandler(RcBlock::as_ptr(&done)) };
    }
    pub(super) fn phase(&self, name: &'static str) {
        if self.1 {
            PHASE.set(Some(name));
        }
    }
}
impl Drop for ShiftProbe {
    fn drop(&mut self) {
        PHASE.set(self.0)
    }
}

enum Pass<'a> {
    Blit(&'a AnyObject),
    Render(&'a MTLRenderPassDescriptor),
}

pub(super) fn blit(
    buffer: &ProtocolObject<dyn MTLCommandBuffer>,
) -> Option<Retained<ProtocolObject<dyn objc2_metal::MTLBlitCommandEncoder>>> {
    if PHASE.get() != Some("shift_copy") {
        return buffer.blitCommandEncoder();
    }
    let desc: Retained<AnyObject> = unsafe { msg_send![class!(MTLBlitPassDescriptor), new] };
    sample(buffer, Pass::Blit(&desc), Some("shift_copy"));
    unsafe { msg_send![buffer,blitCommandEncoderWithDescriptor:&*desc] }
}

pub(crate) fn attach(
    buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: &MTLRenderPassDescriptor,
    probe_copy: bool,
) {
    let phase = PHASE.get();
    static ON: OnceLock<bool> = OnceLock::new();
    if phase.is_none()
        && !(probe_copy
            && *ON.get_or_init(|| std::env::var("RUSTUI_G_R3_COPY_GPU").as_deref() == Ok("1")))
    {
        return;
    }
    sample(buffer, Pass::Render(pass), phase)
}

fn sample(
    buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: Pass<'_>,
    phase: Option<&'static str>,
) {
    let count: usize = if matches!(pass, Pass::Blit(_)) { 2 } else { 4 };
    let device = buffer.device();
    if !device.supportsCounterSampling(MTLCounterSamplingPoint::AtStageBoundary) {
        return;
    }
    // These are the SDK's MTLCounters selectors. Use objc2 dispatch here like
    // the existing GPUStartTime probe, without changing the renderer crate's features.
    #[link(name = "Metal", kind = "framework")]
    unsafe extern "C" {
        static MTLCommonCounterSetTimestamp: &'static NSString;
    }
    let sets: Option<Retained<NSArray<AnyObject>>> = unsafe { msg_send![&*device, counterSets] };
    let Some(sets) = sets else { return };
    let Some(set) = (0..sets.count()).map(|i| sets.objectAtIndex(i)).find(|s| {
        let name: Retained<NSString> = unsafe { msg_send![&**s, name] };
        *name == *unsafe { MTLCommonCounterSetTimestamp }
    }) else {
        return;
    };
    let config: Retained<AnyObject> =
        unsafe { msg_send![class!(MTLCounterSampleBufferDescriptor), new] };
    // SAFETY: These properties have the SDK types id, MTLStorageMode, and NSUInteger.
    let samples: Result<Retained<AnyObject>, Retained<NSError>> = unsafe {
        let _: () = msg_send![&*config,setCounterSet:&*set];
        let _: () = msg_send![&*config,setStorageMode:MTLStorageMode::Shared];
        let _: () = msg_send![&*config,setSampleCount:count];
        msg_send![&*device,newCounterSampleBufferWithDescriptor:&*config,error:_]
    };
    let Ok(samples) = samples else { return };
    unsafe {
        match pass {
            Pass::Render(pass) => {
                let att = pass.sampleBufferAttachments().objectAtIndexedSubscript(0);
                let _: () = msg_send![&*att,setSampleBuffer:&*samples];
                att.setStartOfVertexSampleIndex(0);
                att.setEndOfVertexSampleIndex(1);
                att.setStartOfFragmentSampleIndex(2);
                att.setEndOfFragmentSampleIndex(3);
            }
            Pass::Blit(pass) => {
                let attachments: Retained<AnyObject> = msg_send![pass, sampleBufferAttachments];
                let att: Retained<AnyObject> =
                    msg_send![&*attachments,objectAtIndexedSubscript:0usize];
                let _: () = msg_send![&*att,setSampleBuffer:&*samples];
                let _: () = msg_send![&*att,setStartOfEncoderSampleIndex:0usize];
                let _: () = msg_send![&*att,setEndOfEncoderSampleIndex:1usize];
            }
        }
    }
    let callback = RcBlock::new(
        move |completed: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            // SAFETY: Metal has completed this buffer, and all four timestamp samples exist.
            let data: Option<Retained<NSData>> =
                unsafe { msg_send![&*samples,resolveCounterRange:NSRange::new(0,count)] };
            let Some(data) = data else { return };
            let bytes = unsafe { data.as_bytes_unchecked() };
            if bytes.len() != count * 8 {
                return;
            }
            let ticks: [u64; 4] = std::array::from_fn(|i| {
                let i = i % count;
                u64::from_ne_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap())
            });
            if ticks.iter().any(|&t| t == 0 || t == u64::MAX)
                || ticks[1] < ticks[0]
                || ticks[3] < ticks[2]
            {
                return;
            }
            // Keep raw stage counters. The report calibrates the rate of this
            // counter stream to GPU command-buffer seconds over the capture.
            let first = ticks[0].min(ticks[2]);
            let last = ticks[1].max(ticks[3]);
            let fragment_ticks = ticks[3] - ticks[2];
            // SAFETY: Metal supplies the completed command buffer. Both selectors
            // return CFTimeInterval, as in the adjacent frame GPU recorder.
            let (start_s, end_s): (f64, f64) = unsafe {
                let completed = completed.as_ref();
                (
                    msg_send![completed, GPUStartTime],
                    msg_send![completed, GPUEndTime],
                )
            };
            if !(start_s > 0. && end_s > start_s) {
                return;
            }
            let epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.;
            if let Some(phase) = phase {
                super::log::line(format_args!(
                    "RUSTUI_SHIFT_GPU {epoch:.6} {phase} {first} {last} {start_s:.9} {end_s:.9} {} {} {} {}",
                    ticks[0], ticks[1], ticks[2], ticks[3]
                ));
                return;
            }
            super::log::line(format_args!(
                "RUSTUI_G_R3_COPY_GPU {epoch:.6} {first} {last} {start_s:.9} {end_s:.9} {fragment_ticks}"
            ));
        },
    );
    // SAFETY: Metal copies the completion block, which owns its sample buffer.
    unsafe { buffer.addCompletedHandler(RcBlock::as_ptr(&callback)) };
}
