// 麦克风 probe — 仅用 cpal 抓默认输入设备，记录每包 chunk size、累计 RMS、采样参数。
//
// Phase 0 不做 16 kHz / 16-bit / mono PCM 重采样 — 那是 Slice 1 接火山 ASR 时的事。
// 这里只证明：
//   1. 能枚举默认输入设备
//   2. 能拿到 callback 持续推 PCM
//   3. 能提供 level 给后续 capsule UI
//
// cpal::Stream 不是 Send/Sync，所以放在专门的线程里持有，主线程只通过原子标志和
// channel 跟它交互。停止时 set 标志、join 线程，最终把统计数据带出来。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Default, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicProbeStats {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub total_chunks: u64,
    pub total_bytes: u64,
    pub last_chunk_bytes: u64,
    pub last_level: f32,
    pub duration_ms: u128,
}

struct Probe {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<MicProbeStats>,
}

static PROBE: Lazy<Mutex<Option<Probe>>> = Lazy::new(|| Mutex::new(None));

#[tauri::command]
pub fn start_microphone_probe() -> Result<String, String> {
    let mut slot = PROBE.lock();
    if slot.is_some() {
        return Err("mic probe already running".into());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_in_thread = stop.clone();

    let (ready_tx, ready_rx) = mpsc::channel::<Result<String, String>>();

    let handle = thread::spawn(move || run_probe(stop_in_thread, ready_tx));

    let started = ready_rx
        .recv_timeout(Duration::from_secs(3))
        .map_err(|err| format!("等待 mic 线程就绪超时: {err}"))?;

    match started {
        Ok(msg) => {
            *slot = Some(Probe { stop, handle });
            Ok(msg)
        }
        Err(err) => {
            // 线程已经因为错误退出，drop 自然就清了。
            let _ = handle.join();
            Err(err)
        }
    }
}

#[tauri::command]
pub fn stop_microphone_probe() -> Result<MicProbeStats, String> {
    let probe = PROBE.lock().take().ok_or("mic probe not running")?;
    probe.stop.store(true, Ordering::SeqCst);
    probe
        .handle
        .join()
        .map_err(|_| "mic 线程 join 失败".to_string())
}

fn run_probe(stop: Arc<AtomicBool>, ready: mpsc::Sender<Result<String, String>>) -> MicProbeStats {
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(device) => device,
        None => {
            let _ = ready.send(Err("没找到默认麦克风设备".into()));
            return MicProbeStats::default();
        }
    };

    let device_name = device.name().unwrap_or_else(|_| "<unknown>".to_string());

    let config = match device.default_input_config() {
        Ok(config) => config,
        Err(err) => {
            let _ = ready.send(Err(format!("拿默认输入配置失败: {err}")));
            return MicProbeStats {
                device: device_name,
                ..Default::default()
            };
        }
    };

    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.clone().into();

    let stats = Arc::new(Mutex::new(MicProbeStats {
        device: device_name.clone(),
        sample_rate: stream_config.sample_rate.0,
        channels: stream_config.channels,
        ..Default::default()
    }));

    let stats_in_cb = stats.clone();
    let err_fn = |err| eprintln!("[mic] stream error: {err}");

    let stream_result = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| update_stats(&stats_in_cb, data),
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| update_stats(&stats_in_cb, data),
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| update_stats(&stats_in_cb, data),
            err_fn,
            None,
        ),
        other => {
            let _ = ready.send(Err(format!("暂不支持的 sample format: {other:?}")));
            return stats.lock().clone();
        }
    };

    let stream = match stream_result {
        Ok(stream) => stream,
        Err(err) => {
            let _ = ready.send(Err(format!("build_input_stream 失败: {err}")));
            return stats.lock().clone();
        }
    };

    if let Err(err) = stream.play() {
        let _ = ready.send(Err(format!("stream.play 失败: {err}")));
        return stats.lock().clone();
    }

    let _ = ready.send(Ok(format!(
        "started device=\"{}\" sampleRate={} channels={} sampleFormat={:?}",
        device_name, stream_config.sample_rate.0, stream_config.channels, sample_format
    )));

    let started_at = Instant::now();
    while !stop.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(50));
    }

    drop(stream);

    let mut final_stats = stats.lock().clone();
    final_stats.duration_ms = started_at.elapsed().as_millis();
    final_stats
}

fn update_stats<T: Sample + cpal::SizedSample + Into<f32> + Copy>(
    stats: &Arc<Mutex<MicProbeStats>>,
    data: &[T],
) where
    f32: cpal::FromSample<T>,
{
    let bytes = std::mem::size_of_val(data) as u64;
    let mut sum_sq: f64 = 0.0;
    for sample in data.iter() {
        let f: f32 = f32::from_sample(*sample);
        sum_sq += (f as f64) * (f as f64);
    }
    let level = if data.is_empty() {
        0.0
    } else {
        ((sum_sq / data.len() as f64).sqrt()) as f32
    };

    let mut s = stats.lock();
    s.total_chunks += 1;
    s.total_bytes += bytes;
    s.last_chunk_bytes = bytes;
    s.last_level = level;
}
