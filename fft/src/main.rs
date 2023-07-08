const PI: f32 = std::f32::consts::PI;

fn butterfly_operation(xr: &mut [f32], xi: &mut [f32], n: usize, n_half: usize) {
    for k in 0..n_half {
        println!("change k={}, n/2+k={}", k, n_half + k);

        // 回転因子の計算
        let angle = 2.0 * PI * (k as f32) / (n as f32);
        let wr = angle.cos();
        let wi = (-angle).sin();

        // バタフライ演算
        // a - b
        let difference_r = xr[k] - xr[n_half + k];
        let difference_i = xi[k] - xi[n_half + k];

        // a + b
        xr[k] = xr[k] + xr[n_half + k];
        xi[k] = xi[k] + xi[n_half + k];

        // W(a - b)
        xr[n_half + k] = difference_r * wr + difference_i * wi * (-1.0/*= i^2 */);
        xi[n_half + k] = difference_i * wr + difference_r * wi;
    }
}

fn bit_reversal_permutation(data: &mut [f32]) {
    let mut i = 0;

    for j in 1..data.len() - 1 {
        // ビット反転
        let mut k = data.len() >> 1;
        i ^= k;
        while k > i {
            k >>= 1;
            i ^= k;
        }

        // 並べ替え
        if j < i {
            data.swap(i, j);
        }
    }
}

// 高速フーリエ変換 (FFT: Fast Fourier Transform) を行う関数
fn cq_fft_1(n: usize, xr: &mut [f32], xi: &mut [f32]) {
    // データ数が2以上の場合にFFTを適用
    println!("size n={}", n);
    println!("xr={:?}", xr);
    println!("xi={:?}", xi);
    if n > 1 {
        // データ数の半分
        let n_half = n / 2;
        // 入力データをバタフライ演算
        butterfly_operation(xr, xi, n, n_half);
        // 再帰的にFFTを適用
        cq_fft_1(n_half, &mut xr[0..n_half], &mut xi[0..n_half]);
        cq_fft_1(n_half, &mut xr[n_half..n], &mut xi[n_half..n]);
    }
}

fn main() {
    // データ数
    let n = 4;
    // 実数部のデータ
    let mut xr = vec![1.0, 1.0, -1.0, -1.0];
    // 虚数部のデータ（0で初期化）
    let mut xi = vec![0.0; n];
    // FFTを適用
    cq_fft_1(n, &mut xr, &mut xi);

    bit_reversal_permutation(&mut xr);
    bit_reversal_permutation(&mut xi);

    println!("FFT result(without normalization):");
    // 結果を出力
    for i in 0..n {
        println!("{} + {}i", xr[i], xi[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_cq_fft_1() {
        // データ数
        let n = 4;
        // 実数部のデータ
        let mut xr = vec![1.0, 1.0, -1.0, -1.0];
        // 虚数部のデータ（0で初期化）
        let mut xi = vec![0.0; n];

        // FFTを適用
        cq_fft_1(n, &mut xr, &mut xi);
        bit_reversal_permutation(&mut xr);
        bit_reversal_permutation(&mut xi);

        // 期待される結果（正規化していない）
        let expected_xr = vec![0.0, 2.0, 0.0, 2.0];
        let expected_xi = vec![0.0, -2.0, 0.0, 2.0];

        // 結果の検証
        for i in 0..n {
            assert_approx_eq!(xr[i], expected_xr[i], 1e-3f32);
            assert_approx_eq!(xi[i], expected_xi[i], 1e-3f32);
        }
    }

    #[test]
    fn test_bit_reversal_permutation() {
        let mut data1 = vec![1.0, 2.0, 3.0, 4.0];
        let expect = vec![1.0, 3.0, 2.0, 4.0];
        bit_reversal_permutation(&mut data1);
        assert_eq!(expect, data1);
    }
}
