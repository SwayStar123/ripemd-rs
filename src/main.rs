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

    // The order in which the words should be selected from the block array X for the left hand side are (each sub array within the 2D array represents a round. The array at the top represents the round at the top and the array at the bottom represents the round at the bottom):
    let lhs = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], // R1
        [7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5, 2, 14, 11, 8], // R2
        [3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12], // R3
        [1, 9, 11, 10, 0, 8, 12, 4, 13, 3, 7, 15, 14, 5, 6, 2], // R4
        [4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13], // R5
    ];

    // The order in which the words should be selected from the array X for the right hand side are (following the same pattern as above):
    let rhs = [
        [5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12], // R1
        [6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12, 4, 9, 1, 2], // R2
        [15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13], // R3
        [8, 6, 4, 1, 3, 11, 15, 0, 5, 12, 2, 13, 9, 7, 10, 14], // R4
        [12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11], // R5
    ];

    // The order of the left rotates on the left hand side are:
    let lhs_rotates = [
        [11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8], // R1
        [7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15, 9, 11, 7, 13, 12], // R2
        [11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5], // R3
        [11, 12, 14, 15, 14, 15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12], // R4
        [9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6], // R5
    ];

    // The order of the left rotates on the right hand side are:
    let rhs_rotates = [
        [8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6], // R1
        [9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12, 7, 6, 15, 13, 11], // R2
        [9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5], // R3
        [15, 5, 8, 11, 14, 14, 6, 14, 6, 9, 12, 9, 12, 5, 15, 8], // R4
        [8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11], // R5
    ];

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];

    let aa = a;
    let bb = b;
    let cc = c;
    let dd = d;
    let ee = e;

    let functions = [f0, f1, f2, f3, f4];
    let k = [K0, K1, K2, K3, K4];
    let kk = [KK0, KK1, KK2, KK3, KK4];

    // Left hand side
    for i in 0..5 {
        for j in 0..16 {
            let (a1, c1) = r(
                a,
                b,
                c,
                d,
                e,
                functions[i],
                k[i],
                lhs_rotates[i][j],
                lhs[i][j],
                &x,
            );
            a = e;
            e = d;
            d = rol(10, c);
            c = b;
            b = a1;
            (a, c) = (a1, c1);
        }
    }

    // Right hand side
    for i in 0..5 {
        for j in 0..16 {
            let (a1, c1) = r(
                a,
                b,
                c,
                d,
                e,
                functions[i],
                kk[i],
                rhs_rotates[i][j],
                rhs[i][j],
                &x,
            );
            a = e;
            e = d;
            d = rol(10, c);
            c = b;
            b = a1;
            (a, c) = (a1, c1);
        }
    }

    let t = state[1].wrapping_add(c).wrapping_add(dd);
    state[1] = state[2].wrapping_add(d).wrapping_add(ee);
    state[2] = state[3].wrapping_add(e).wrapping_add(aa);
    state[3] = state[4].wrapping_add(a).wrapping_add(bb);
    state[4] = state[0].wrapping_add(b).wrapping_add(cc);
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
