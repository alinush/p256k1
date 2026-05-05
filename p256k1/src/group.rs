use crate::_rename::{secp256k1_fe_inv, secp256k1_fe_mul, secp256k1_fe_set_int, secp256k1_fe_sqr};
use crate::bindings::{secp256k1_fe, secp256k1_ge, secp256k1_gej};

fn fe_default() -> secp256k1_fe {
    secp256k1_fe {
        n: Default::default(),
    }
}

fn ge_default() -> secp256k1_ge {
    secp256k1_ge {
        x: fe_default(),
        y: fe_default(),
        infinity: 0,
    }
}

#[inline]
pub fn secp256k1_ge_set_gej(r: &mut secp256k1_ge, a: &secp256k1_gej) {
    let mut z2 = fe_default();
    let mut z3 = fe_default();
    let mut az = fe_default();
    let mut ax = fe_default();
    let mut ay = fe_default();
    r.infinity = a.infinity;
    unsafe {
        secp256k1_fe_inv(&mut az, &a.z);
        secp256k1_fe_sqr(&mut z2, &az);
        secp256k1_fe_mul(&mut z3, &az, &z2);
        secp256k1_fe_mul(&mut ax, &a.x, &z2);
        secp256k1_fe_mul(&mut ay, &a.y, &z3);
        secp256k1_fe_set_int(&mut az, 1);
    }
    r.x = ax;
    r.y = ay;
}

/// Batch Jacobian → affine conversion using Montgomery's simultaneous-inversion
/// trick: 1 modular inversion + ~5n multiplications + n squarings, instead of
/// n modular inversions in `secp256k1_ge_set_gej`.
///
/// At-infinity inputs are passed through with `infinity = 1` set on the output.
pub fn secp256k1_ge_set_all_gej_var(affines: &mut [secp256k1_ge], jacs: &[secp256k1_gej]) {
    assert_eq!(affines.len(), jacs.len());
    let n = jacs.len();

    // Mark infinities first (and clear the rest to a canonical state).
    for i in 0..n {
        affines[i] = ge_default();
        if jacs[i].infinity != 0 {
            affines[i].infinity = 1;
        }
    }

    // Indices of non-infinity points.
    let live: Vec<usize> = (0..n).filter(|&i| jacs[i].infinity == 0).collect();
    let m = live.len();
    if m == 0 {
        return;
    }

    // Step 1: build the running-product chain over Z_i for live points.
    //   us[k] = Z_{live[0]} * Z_{live[1]} * ... * Z_{live[k]}.
    let mut us: Vec<secp256k1_fe> = vec![fe_default(); m];
    us[0] = jacs[live[0]].z;
    for k in 1..m {
        unsafe {
            secp256k1_fe_mul(&mut us[k], &us[k - 1], &jacs[live[k]].z);
        }
    }

    // Step 2: one inversion at the top.
    let mut v = fe_default();
    unsafe {
        secp256k1_fe_inv(&mut v, &us[m - 1]);
    }

    // Step 3: walk back, peeling off one Z⁻¹ at a time.
    for k in (0..m).rev() {
        let i = live[k];
        let z_inv = if k == 0 {
            v
        } else {
            let mut z_inv = fe_default();
            unsafe {
                secp256k1_fe_mul(&mut z_inv, &v, &us[k - 1]);
                let mut new_v = fe_default();
                secp256k1_fe_mul(&mut new_v, &v, &jacs[i].z);
                v = new_v;
            }
            z_inv
        };

        // (X, Y, Z) → (X / Z², Y / Z³).
        let mut z2 = fe_default();
        let mut z3 = fe_default();
        let mut x_aff = fe_default();
        let mut y_aff = fe_default();
        unsafe {
            secp256k1_fe_sqr(&mut z2, &z_inv);
            secp256k1_fe_mul(&mut z3, &z_inv, &z2);
            secp256k1_fe_mul(&mut x_aff, &jacs[i].x, &z2);
            secp256k1_fe_mul(&mut y_aff, &jacs[i].y, &z3);
        }
        affines[i].x = x_aff;
        affines[i].y = y_aff;
        affines[i].infinity = 0;
    }
}
