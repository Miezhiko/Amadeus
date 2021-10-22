use rand::Rng;

static BASE_DIVERGENCE: f32 = 80f32;

fn unfloat(v: f32) -> u8 {
  if v >= 1.0 { 255u8 } else {
    (v * 256.0) as u8
  }
}

fn hsv(huy: f32, sat: f32, val: f32) -> (u8, u8, u8) {
  set!{ chrome = val * sat
      , huyhuy = huy / 60.0
      , tmpcol = chrome * (1.0 - ((huyhuy % 2.0) - 1.0).abs())
      , midval = val - chrome };
  let c = match huyhuy {
    h if (0.0..1.0).contains(&h) => (chrome, tmpcol, 0.0),
    h if (1.0..2.0).contains(&h) => (tmpcol, chrome, 0.0),
    h if (2.0..3.0).contains(&h) => (0.0, chrome, tmpcol),
    h if (3.0..4.0).contains(&h) => (0.0, tmpcol, chrome),
    h if (4.0..5.0).contains(&h) => (tmpcol, 0.0, chrome),
    h if (5.0..6.0).contains(&h) => (chrome, 0.0, tmpcol),
    _                            => (0.0, 0.0, 0.0) };
  ( unfloat (c.0 + midval)
  , unfloat (c.1 + midval)
  , unfloat (c.2 + midval) )
}

fn rnd_cos(huy: &mut f32, sat: &mut f32, val: &mut f32, i: f32, d: f32) {
  set!{ fix = (i * 30.0).cos().abs()
      , div = if d < 15.0 { 15.0 } else { d } };
  *huy = (*huy + div + fix).abs() % 360.0;
  *sat = ((i * 0.35).cos() / 4.0).abs();
  *val = 0.5 + (i.cos() / 2.0).abs();
}

fn rnd_tan(huy: &mut f32, sat: &mut f32, val: &mut f32, i: f32, d: f32) {
  set!{ fix = (i * 55.0).tan().abs()
      , div = if d < 15.0 { 15.0 } else { d } };
  *huy = (*huy + div + fix).abs() % 360.0;
  *sat = (i * 0.35).sin().abs();
  *val = ((6.33 * i) * 0.5).cos().abs();
  if *sat < 0.4 { *sat = 0.4; }
  if *val < 0.2 {
    *val = 0.2;
  } else if *val > 0.85 {
    *val = 0.85;
  }
}

pub fn gen_colors(n: usize) -> Vec<(u8, u8, u8)> {
  setm!{ rng = rand::thread_rng()
       , huy = rng.gen_range(0.0..360.0)
       , sat = rng.gen_range(0.35..1.0)
       , val = rng.gen_range(0.55..1.0)
       , palette = Vec::with_capacity(n) };
  set!{ rand_bool   = rng.gen_range(0..2)
      , use_cos_f   = rand_bool == 1
      , divergence  = BASE_DIVERGENCE - n as f32 / 2.6 };
  for i in 0..n {
    let rgb = hsv(huy, sat, val);
    if use_cos_f {
      rnd_cos(&mut huy, &mut sat, &mut val, i as f32, divergence);
    } else {
      rnd_tan(&mut huy, &mut sat, &mut val, i as f32, divergence);
    }
    palette.push(rgb);
  } palette
}
