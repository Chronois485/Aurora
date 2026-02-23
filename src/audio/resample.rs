#[derive(Debug, Clone)]
pub struct LinearResampler {
    in_rate: u32,
    out_rate: u32,
    pos: f64,
    step: f64,
}

impl LinearResampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        let step = in_rate as f64 / out_rate as f64;
        Self {
            in_rate,
            out_rate,
            pos: 0.0,
            step,
        }
    }

    pub fn process(&mut self, input: &[i16]) -> Vec<i16> {
        if input.len() < 2 || self.in_rate == self.out_rate {
            return input.to_vec();
        }

        let estimated = ((input.len() as f64) / self.step).ceil() as usize;
        let mut out = Vec::with_capacity(estimated);

        while (self.pos as usize) + 1 < input.len() {
            let i = self.pos.floor() as usize;
            let frac = self.pos - (i as f64);

            let a = input[i] as f64;
            let b = input[i + 1] as f64;

            let y = a + (b - a) * frac;
            out.push(y.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16);

            self.pos += self.step;
        }

        self.pos -= input.len() as f64;
        if self.pos < 0.0 {
            self.pos = 0.0;
        }

        out
    }
}
