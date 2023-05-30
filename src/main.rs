const PADDING: [u8; 64] = [
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

const K0: u32 = 0x00000000;
const K1: u32 = 0x5A827999;
const K2: u32 = 0x6ED9EBA1;
const K3: u32 = 0x8F1BBCDC;
const K4: u32 = 0xA953FD4E;
const KK0: u32 = 0x50A28BE6;
const KK1: u32 = 0x5C4DD124;
const KK2: u32 = 0x6D703EF3;
const KK3: u32 = 0x7A6D76E9;
const KK4: u32 = 0x00000000;

struct RMDContext {
    state: [u32; 5],
    count: u64,
    buffer: [u8; 64],
}

impl RMDContext {
    fn new() -> RMDContext {
        RMDContext {
            state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            count: 0,
            buffer: [0; 64],
        }
    }
}

fn ripemd160(input: &[u8]) -> [u8; 20] {
    let mut ctx = RMDContext::new();
    rmd160_update(&mut ctx, input);
    rmd160_final(&mut ctx)
}

fn rmd160_update(ctx: &mut RMDContext, input: &[u8]) {
    let mut have = (ctx.count / 8) % 64;
    let inplen = input.len() as u64;
    let need = 64 - have;
    ctx.count += 8 * inplen;
    let mut off = 0;

    if inplen >= need {
        if have > 0 {
            for i in 0..need {
                ctx.buffer[(have + i) as usize] = input[i as usize];
            }
            rmd160_transform(&mut ctx.state, &ctx.buffer);
            off = need;
            have = 0;
        }
        while off + 64 <= inplen {
            rmd160_transform(&mut ctx.state, &input[off as usize..]);
            off += 64;
        }
    }
    if off < inplen {
        for i in 0..(inplen - off) {
            ctx.buffer[(have + i) as usize] = input[(off + i) as usize];
        }
    }
}

fn rmd160_final(ctx: &mut RMDContext) -> [u8; 20] {
    let size = ctx.count.to_le_bytes();
    let mut padlen = 64 - ((ctx.count / 8) % 64);
    if padlen < 1 + 8 {
        padlen += 64;
    }
    rmd160_update(ctx, &PADDING[..padlen as usize - 8]);
    rmd160_update(ctx, &size);
    let mut result = [0u8; 20];
    for (i, chunk) in ctx.state.iter().enumerate() {
        result[i * 4..(i + 1) * 4].copy_from_slice(&chunk.to_le_bytes());
    }
    result
}

fn rol(n: u32, x: u32) -> u32 {
    (x << n) | (x >> (32 - n))
}

fn f0(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn f1(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | ((!x) & z)
}

fn f2(x: u32, y: u32, z: u32) -> u32 {
    (x | !y) ^ z
}

fn f3(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (!z & y)
}

fn f4(x: u32, y: u32, z: u32) -> u32 {
    x ^ (y | !z)
}

fn r(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    fj: fn(u32, u32, u32) -> u32,
    kj: u32,
    sj: u32,
    rj: usize,
    x: &[u32],
) -> (u32, u32) {
    let a = rol(
        sj,
        a.wrapping_add(fj(b, c, d))
            .wrapping_add(x[rj])
            .wrapping_add(kj),
    ).wrapping_add(e);
    let c = rol(10, c);
    (a, c)
}

fn rmd160_transform(state: &mut [u32; 5], block: &[u8]) {
    assert_eq!(block.len(), 64);
    let mut x = [0u32; 16];
    for (i, chunk) in block.chunks(4).enumerate() {
        x[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];

    /* Round 1 */
    (a, c) = r(a, b, c, d, e, f0, K0, 11,  0, &x);
    (e, b) = r(e, a, b, c, d, f0, K0, 14,  1, &x);
    (d, a) = r(d, e, a, b, c, f0, K0, 15,  2, &x);
    (c, e) = r(c, d, e, a, b, f0, K0, 12,  3, &x);
    (b, d) = r(b, c, d, e, a, f0, K0,  5,  4, &x);
    (a, c) = r(a, b, c, d, e, f0, K0,  8,  5, &x);
    (e, b) = r(e, a, b, c, d, f0, K0,  7,  6, &x);
    (d, a) = r(d, e, a, b, c, f0, K0,  9,  7, &x);
    (c, e) = r(c, d, e, a, b, f0, K0, 11,  8, &x);
    (b, d) = r(b, c, d, e, a, f0, K0, 13,  9, &x);
    (a, c) = r(a, b, c, d, e, f0, K0, 14, 10, &x);
    (e, b) = r(e, a, b, c, d, f0, K0, 15, 11, &x);
    (d, a) = r(d, e, a, b, c, f0, K0,  6, 12, &x);
    (c, e) = r(c, d, e, a, b, f0, K0,  7, 13, &x);
    (b, d) = r(b, c, d, e, a, f0, K0,  9, 14, &x);
    (a, c) = r(a, b, c, d, e, f0, K0,  8, 15, &x); /* #15 */

    /* Round 2 */
    (e, b) = r(e, a, b, c, d, f1, K1,  7,  7, &x);
    (d, a) = r(d, e, a, b, c, f1, K1,  6,  4, &x);
    (c, e) = r(c, d, e, a, b, f1, K1,  8, 13, &x);
    (b, d) = r(b, c, d, e, a, f1, K1, 13,  1, &x);
    (a, c) = r(a, b, c, d, e, f1, K1, 11, 10, &x);
    (e, b) = r(e, a, b, c, d, f1, K1,  9,  6, &x);
    (d, a) = r(d, e, a, b, c, f1, K1,  7, 15, &x);
    (c, e) = r(c, d, e, a, b, f1, K1, 15,  3, &x);
    (b, d) = r(b, c, d, e, a, f1, K1,  7, 12, &x);
    (a, c) = r(a, b, c, d, e, f1, K1, 12,  0, &x);
    (e, b) = r(e, a, b, c, d, f1, K1, 15,  9, &x);
    (d, a) = r(d, e, a, b, c, f1, K1,  9,  5, &x);
    (c, e) = r(c, d, e, a, b, f1, K1, 11,  2, &x);
    (b, d) = r(b, c, d, e, a, f1, K1,  7, 14, &x);
    (a, c) = r(a, b, c, d, e, f1, K1, 13, 11, &x);
    (e, b) = r(e, a, b, c, d, f1, K1, 12,  8, &x); /* #31 */

    /* Round 3 */
    (d, a) = r(d, e, a, b, c, f2, K2, 11,  3, &x);
    (c, e) = r(c, d, e, a, b, f2, K2, 13, 10, &x);
    (b, d) = r(b, c, d, e, a, f2, K2,  6, 14, &x);
    (a, c) = r(a, b, c, d, e, f2, K2,  7,  4, &x);
    (e, b) = r(e, a, b, c, d, f2, K2, 14,  9, &x);
    (d, a) = r(d, e, a, b, c, f2, K2,  9, 15, &x);
    (c, e) = r(c, d, e, a, b, f2, K2, 13,  8, &x);
    (b, d) = r(b, c, d, e, a, f2, K2, 15,  1, &x);
    (a, c) = r(a, b, c, d, e, f2, K2, 14,  2, &x);
    (e, b) = r(e, a, b, c, d, f2, K2,  8,  7, &x);
    (d, a) = r(d, e, a, b, c, f2, K2, 13,  0, &x);
    (c, e) = r(c, d, e, a, b, f2, K2,  6,  6, &x);
    (b, d) = r(b, c, d, e, a, f2, K2,  5, 13, &x);
    (a, c) = r(a, b, c, d, e, f2, K2, 12, 11, &x);
    (e, b) = r(e, a, b, c, d, f2, K2,  7,  5, &x);
    (d, a) = r(d, e, a, b, c, f2, K2,  5, 12, &x); /* #47 */

    /* Round 4 */
    (c, e) = r(c, d, e, a, b, f3, K3, 11,  1, &x);
    (b, d) = r(b, c, d, e, a, f3, K3, 12,  9, &x);
    (a, c) = r(a, b, c, d, e, f3, K3, 14, 11, &x);
    (e, b) = r(e, a, b, c, d, f3, K3, 15, 10, &x);
    (d, a) = r(d, e, a, b, c, f3, K3, 14,  0, &x);
    (c, e) = r(c, d, e, a, b, f3, K3, 15,  8, &x);
    (b, d) = r(b, c, d, e, a, f3, K3,  9, 12, &x);
    (a, c) = r(a, b, c, d, e, f3, K3,  8,  4, &x);
    (e, b) = r(e, a, b, c, d, f3, K3,  9, 13, &x);
    (d, a) = r(d, e, a, b, c, f3, K3, 14,  3, &x);
    (c, e) = r(c, d, e, a, b, f3, K3,  5,  7, &x);
    (b, d) = r(b, c, d, e, a, f3, K3,  6, 15, &x);
    (a, c) = r(a, b, c, d, e, f3, K3,  8, 14, &x);
    (e, b) = r(e, a, b, c, d, f3, K3,  6,  5, &x);
    (d, a) = r(d, e, a, b, c, f3, K3,  5,  6, &x);
    (c, e) = r(c, d, e, a, b, f3, K3, 12,  2, &x); /* #63 */

    /* Round 5 */
    (b, d) = r(b, c, d, e, a, f4, K4,  9,  4, &x);
    (a, c) = r(a, b, c, d, e, f4, K4, 15,  0, &x);
    (e, b) = r(e, a, b, c, d, f4, K4,  5,  5, &x);
    (d, a) = r(d, e, a, b, c, f4, K4, 11,  9, &x);
    (c, e) = r(c, d, e, a, b, f4, K4,  6,  7, &x);
    (b, d) = r(b, c, d, e, a, f4, K4,  8, 12, &x);
    (a, c) = r(a, b, c, d, e, f4, K4, 13,  2, &x);
    (e, b) = r(e, a, b, c, d, f4, K4, 12, 10, &x);
    (d, a) = r(d, e, a, b, c, f4, K4,  5, 14, &x);
    (c, e) = r(c, d, e, a, b, f4, K4, 12,  1, &x);
    (b, d) = r(b, c, d, e, a, f4, K4, 13,  3, &x);
    (a, c) = r(a, b, c, d, e, f4, K4, 14,  8, &x);
    (e, b) = r(e, a, b, c, d, f4, K4, 11, 11, &x);
    (d, a) = r(d, e, a, b, c, f4, K4,  8,  6, &x);
    (c, e) = r(c, d, e, a, b, f4, K4,  5, 15, &x);
    (b, d) = r(b, c, d, e, a, f4, K4,  6, 13, &x); /* #79 */

    let aa = a;
    let bb = b;
    let cc = c;
    let dd = d;
    let ee = e;

    a = state[0];
    b = state[1];
    c = state[2];
    d = state[3];
    e = state[4];
    
    /* Parallel round 1 */
    (a, c) = r(a, b, c, d, e, f4, KK0,  8,  5, &x);
    (e, b) = r(e, a, b, c, d, f4, KK0,  9, 14, &x);
    (d, a) = r(d, e, a, b, c, f4, KK0,  9,  7, &x);
    (c, e) = r(c, d, e, a, b, f4, KK0, 11,  0, &x);
    (b, d) = r(b, c, d, e, a, f4, KK0, 13,  9, &x);
    (a, c) = r(a, b, c, d, e, f4, KK0, 15,  2, &x);
    (e, b) = r(e, a, b, c, d, f4, KK0, 15, 11, &x);
    (d, a) = r(d, e, a, b, c, f4, KK0,  5,  4, &x);
    (c, e) = r(c, d, e, a, b, f4, KK0,  7, 13, &x);
    (b, d) = r(b, c, d, e, a, f4, KK0,  7,  6, &x);
    (a, c) = r(a, b, c, d, e, f4, KK0,  8, 15, &x);
    (e, b) = r(e, a, b, c, d, f4, KK0, 11,  8, &x);
    (d, a) = r(d, e, a, b, c, f4, KK0, 14,  1, &x);
    (c, e) = r(c, d, e, a, b, f4, KK0, 14, 10, &x);
    (b, d) = r(b, c, d, e, a, f4, KK0, 12,  3, &x);
    (a, c) = r(a, b, c, d, e, f4, KK0,  6, 12, &x); /* #15 */
    /* Parallel round 2 */
    (e, b) = r(e, a, b, c, d, f3, KK1,  9,  6, &x);
    (d, a) = r(d, e, a, b, c, f3, KK1, 13, 11, &x);
    (c, e) = r(c, d, e, a, b, f3, KK1, 15,  3, &x);
    (b, d) = r(b, c, d, e, a, f3, KK1,  7,  7, &x);
    (a, c) = r(a, b, c, d, e, f3, KK1, 12,  0, &x);
    (e, b) = r(e, a, b, c, d, f3, KK1,  8, 13, &x);
    (d, a) = r(d, e, a, b, c, f3, KK1,  9,  5, &x);
    (c, e) = r(c, d, e, a, b, f3, KK1, 11, 10, &x);
    (b, d) = r(b, c, d, e, a, f3, KK1,  7, 14, &x);
    (a, c) = r(a, b, c, d, e, f3, KK1,  7, 15, &x);
    (e, b) = r(e, a, b, c, d, f3, KK1, 12,  8, &x);
    (d, a) = r(d, e, a, b, c, f3, KK1,  7, 12, &x);
    (c, e) = r(c, d, e, a, b, f3, KK1,  6,  4, &x);
    (b, d) = r(b, c, d, e, a, f3, KK1, 15,  9, &x);
    (a, c) = r(a, b, c, d, e, f3, KK1, 13,  1, &x);
    (e, b) = r(e, a, b, c, d, f3, KK1, 11,  2, &x); /* #31 */
    /* Parallel round 3 */
    (d, a) = r(d, e, a, b, c, f2, KK2,  9, 15, &x);
    (c, e) = r(c, d, e, a, b, f2, KK2,  7,  5, &x);
    (b, d) = r(b, c, d, e, a, f2, KK2, 15,  1, &x);
    (a, c) = r(a, b, c, d, e, f2, KK2, 11,  3, &x);
    (e, b) = r(e, a, b, c, d, f2, KK2,  8,  7, &x);
    (d, a) = r(d, e, a, b, c, f2, KK2,  6, 14, &x);
    (c, e) = r(c, d, e, a, b, f2, KK2,  6,  6, &x);
    (b, d) = r(b, c, d, e, a, f2, KK2, 14,  9, &x);
    (a, c) = r(a, b, c, d, e, f2, KK2, 12, 11, &x);
    (e, b) = r(e, a, b, c, d, f2, KK2, 13,  8, &x);
    (d, a) = r(d, e, a, b, c, f2, KK2,  5, 12, &x);
    (c, e) = r(c, d, e, a, b, f2, KK2, 14,  2, &x);
    (b, d) = r(b, c, d, e, a, f2, KK2, 13, 10, &x);
    (a, c) = r(a, b, c, d, e, f2, KK2, 13,  0, &x);
    (e, b) = r(e, a, b, c, d, f2, KK2,  7,  4, &x);
    (d, a) = r(d, e, a, b, c, f2, KK2,  5, 13, &x); /* #47 */
    /* Parallel round 4 */
    (c, e) = r(c, d, e, a, b, f1, KK3, 15,  8, &x);
    (b, d) = r(b, c, d, e, a, f1, KK3,  5,  6, &x);
    (a, c) = r(a, b, c, d, e, f1, KK3,  8,  4, &x);
    (e, b) = r(e, a, b, c, d, f1, KK3, 11,  1, &x);
    (d, a) = r(d, e, a, b, c, f1, KK3, 14,  3, &x);
    (c, e) = r(c, d, e, a, b, f1, KK3, 14, 11, &x);
    (b, d) = r(b, c, d, e, a, f1, KK3,  6, 15, &x);
    (a, c) = r(a, b, c, d, e, f1, KK3, 14,  0, &x);
    (e, b) = r(e, a, b, c, d, f1, KK3,  6,  5, &x);
    (d, a) = r(d, e, a, b, c, f1, KK3,  9, 12, &x);
    (c, e) = r(c, d, e, a, b, f1, KK3, 12,  2, &x);
    (b, d) = r(b, c, d, e, a, f1, KK3,  9, 13, &x);
    (a, c) = r(a, b, c, d, e, f1, KK3, 12,  9, &x);
    (e, b) = r(e, a, b, c, d, f1, KK3,  5,  7, &x);
    (d, a) = r(d, e, a, b, c, f1, KK3, 15, 10, &x);
    (c, e) = r(c, d, e, a, b, f1, KK3,  8, 14, &x); /* #63 */
    /* Parallel round 5 */
    (b, d) = r(b, c, d, e, a, f0, KK4,  8, 12, &x);
    (a, c) = r(a, b, c, d, e, f0, KK4,  5, 15, &x);
    (e, b) = r(e, a, b, c, d, f0, KK4, 12, 10, &x);
    (d, a) = r(d, e, a, b, c, f0, KK4,  9,  4, &x);
    (c, e) = r(c, d, e, a, b, f0, KK4, 12,  1, &x);
    (b, d) = r(b, c, d, e, a, f0, KK4,  5,  5, &x);
    (a, c) = r(a, b, c, d, e, f0, KK4, 14,  8, &x);
    (e, b) = r(e, a, b, c, d, f0, KK4,  6,  7, &x);
    (d, a) = r(d, e, a, b, c, f0, KK4,  8,  6, &x);
    (c, e) = r(c, d, e, a, b, f0, KK4, 13,  2, &x);
    (b, d) = r(b, c, d, e, a, f0, KK4,  6, 13, &x);
    (a, c) = r(a, b, c, d, e, f0, KK4,  5, 14, &x);
    (e, b) = r(e, a, b, c, d, f0, KK4, 15,  0, &x);
    (d, a) = r(d, e, a, b, c, f0, KK4, 13,  3, &x);
    (c, e) = r(c, d, e, a, b, f0, KK4, 11,  9, &x);
    (b, d) = r(b, c, d, e, a, f0, KK4, 11, 11, &x); /* #79 */

    let t = state[1].wrapping_add(cc).wrapping_add(d);
    state[1] = state[2].wrapping_add(dd).wrapping_add(e);
    state[2] = state[3].wrapping_add(ee).wrapping_add(a);
    state[3] = state[4].wrapping_add(aa).wrapping_add(b);
    state[4] = state[0].wrapping_add(bb).wrapping_add(c);
    state[0] = t;
}

use crypto::digest::Digest;
use ripemd::Ripemd160;

fn main() {
    let input: &[u8] = b"Hello, world!";

    // Compute using our implementation
    let our_result = ripemd160(input);
    println!("Our result: {:?}", our_result);

    // Compute using `rust-crypto`
    let mut hasher = Ripemd160::new();
    hasher.update(input);
    let rust_crypto_result = hasher.finalize();
    println!("rust-crypto result: {:?}", rust_crypto_result);
}
