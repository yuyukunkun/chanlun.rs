/*
 * MIT License
 *
 * Copyright (c) 2026 YuYuKunKun
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

/// 将 f64 格式化为与 Python `:g` (6 位有效数字) 兼容的字符串。
pub fn format_f64_g(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    if value == 0.0 {
        return "0".to_string();
    }

    let abs = value.abs();
    let exp = abs.log10().floor() as i32;

    // Python :g 科学计数法边界: exp < -4 或 exp >= p (=6)
    if !(-4..6).contains(&exp) {
        let significand = value / 10_f64.powi(exp);
        let s = format!("{:.5}", significand);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        return format!("{}e{:+03}", s, exp);
    }

    // 定点表示
    if abs >= 1.0 {
        let int_digits = exp as usize + 1;
        if int_digits >= 6 {
            return format!("{:.0}", value);
        }
        let s = format!("{:.prec$}", value, prec = 6 - int_digits);
        let s = s.trim_end_matches('0');
        s.trim_end_matches('.').to_string()
    } else {
        let leading_zeros = (-exp) as usize;
        let s = format!("{:.prec$}", value, prec = leading_zeros + 5);
        let s = s.trim_end_matches('0');
        s.trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_large() {
        assert_eq!(format_f64_g(82833.0), "82833");
        assert_eq!(format_f64_g(74192.2), "74192.2");
        assert_eq!(format_f64_g(100.0), "100");
        assert_eq!(format_f64_g(0.0), "0");
        assert_eq!(format_f64_g(12345.6789), "12345.7");
    }

    #[test]
    fn test_format_small() {
        assert_eq!(format_f64_g(0.001234), "0.001234");
        assert_eq!(format_f64_g(0.1), "0.1");
        assert_eq!(format_f64_g(0.001), "0.001");
    }

    #[test]
    fn test_format_extreme() {
        assert_eq!(format_f64_g(1e7), "1e+07");
        assert_eq!(format_f64_g(1e-5), "1e-05");
    }
}
