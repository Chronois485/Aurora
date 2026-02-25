#[derive(Debug, Clone)]
pub struct DcBlocker {
    mean: f32,
    alpha: f32,
}

impl DcBlocker {
    pub fn new(alpha: f32) -> Self {
        Self { mean: 0.0, alpha }
    }

    #[inline]
    pub fn process_i16(&mut self, x: i16) -> i16 {
        let xf = x as f32;
        self.mean = self.alpha * self.mean + (1.0 - self.alpha) * xf;
        let y = xf - self.mean;
        y.clamp(i16::MIN as f32, i16::MAX as f32) as i16
    }

    pub fn process_buf(&mut self, buf: &mut [i16]) {
        for s in buf.iter_mut() {
            *s = self.process_i16(*s);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Agc {
    target_rms: f32,
    max_gain: f32,
    gain: f32,
    smooth: f32,
}

impl Agc {
    pub fn new(target_rms: f32, max_gain: f32, smooth: f32) -> Self {
        Self {
            target_rms,
            max_gain,
            gain: 1.0,
            smooth,
        }
    }

    pub fn process(&mut self, buf: &mut [i16]) {
        if buf.is_empty() {
            return;
        }

        let mut sum = 0.0f32;
        for &s in buf.iter() {
            let x = s as f32 / i16::MAX as f32;
            sum += x * x;
        }
        let rms = (sum / buf.len() as f32).sqrt().max(1e-6);

        let desired = (self.target_rms / rms).clamp(1.0 / self.max_gain, self.max_gain);

        self.gain = self.smooth * self.gain + (1.0 - self.smooth) * desired;

        for s in buf.iter_mut() {
            let x = *s as f32 * self.gain;
            *s = x.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        }
    }
}

#[inline]
pub fn soft_clip_i16(x: i16, threshold: i16) -> i16 {
    let t = threshold as f32;
    let xf = x as f32;

    if xf.abs() <= t {
        return x;
    }

    let sign = xf.signum();
    let over = xf.abs() - t;
    let y = t + over / (1.0 + over / t);
    (sign * y).clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

pub fn rms_i16(buf: &[i16]) -> f32 {
    if buf.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for &s in buf {
        let x = s as f32 / i16::MAX as f32;
        sum += x * x;
    }
    (sum / buf.len() as f32).sqrt()
}

#[derive(Debug, Clone)]
pub struct AudioNormalizer {
    dc: DcBlocker,
    agc: Agc,
    clip_threshold: i16,
    min_rms: f32,
}

impl AudioNormalizer {
    pub fn new() -> Self {
        Self {
            dc: DcBlocker::new(0.999),
            agc: Agc::new(0.12, 8.0, 0.95),
            clip_threshold: (i16::MAX as f32 * 0.95) as i16,
            min_rms: 0.008,
        }
    }

    pub fn process(&mut self, buf: &mut [i16]) -> bool {
        self.dc.process_buf(buf);
        self.agc.process(buf);

        for s in buf.iter_mut() {
            *s = soft_clip_i16(*s, self.clip_threshold);
        }

        rms_i16(buf) >= self.min_rms
    }
}