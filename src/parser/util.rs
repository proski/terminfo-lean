#[inline]
pub fn number(i: &[u8]) -> i32 {
	let mut n: i32 = 0;

	for &ch in i {
		let d = (ch as i32).wrapping_sub(b'0' as i32);

		if d <= 9 {
			n = n.saturating_mul(10).saturating_add(d);
		}
	}

	n
}
