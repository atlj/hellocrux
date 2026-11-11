use std::time::Duration;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Progress {
    pub frame: usize,
    pub fps: f64,
    pub bitrate: String,
    #[serde(rename = "total_size")]
    pub total_size_bytes: usize,
    // TODO parse from millis
    #[serde(rename = "out_time_ms")]
    pub out_time: Duration,
    pub dup_frames: usize,
    pub drop_frames: usize,
    // TODO parse from string
    pub speed: f64,
    pub progress: ProgressState,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ProgressState {
    Continue,
    End,
}

#[cfg(test)]
mod tests {
    const FIRST_LINE: &str = "frame=0¬
fps=0.00¬
stream_0_0_q=0.0¬
bitrate=N/A¬
total_size=0¬
out_time_us=N/A¬
out_time_ms=N/A¬
out_time=N/A¬
dup_frames=0¬
drop_frames=0¬
speed=N/A¬
progress=continue";

    const CONTINUE_LINE: &str = "frame=943
fps=35.57
stream_0_0_q=36.0
bitrate=   0.0kbits/s
total_size=44
out_time_us=37640000
out_time_ms=37640000
out_time=00:00:37.640000
dup_frames=0
drop_frames=0
speed=1.42x
progress=continue";

    const END_LINE: &str = "frame=12500
fps=36.48
stream_0_0_q=36.0
bitrate=  23.8kbits/s
total_size=1485771
out_time_us=499920000
out_time_ms=499920000
out_time=00:08:19.920000
dup_frames=0
drop_frames=0
speed=1.46x
progress=end";
}
