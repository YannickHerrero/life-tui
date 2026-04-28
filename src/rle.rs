use anyhow::{Context, Result, anyhow};

use crate::grid::Grid;

/// Encode the bounding box of living cells in `grid` as RLE.
/// Returns a complete RLE document including the header line.
pub fn encode_grid(grid: &Grid) -> String {
    let bounds = bounding_box(grid);
    let Some((min_x, min_y, max_x, max_y)) = bounds else {
        return "x = 0, y = 0, rule = B3/S23\n!\n".to_string();
    };
    let w = max_x - min_x + 1;
    let h = max_y - min_y + 1;

    let mut body = String::new();
    let mut row_runs: Vec<(usize, char)> = Vec::new();

    for y in min_y..=max_y {
        let mut runs: Vec<(usize, char)> = Vec::new();
        let mut current = ' ';
        let mut run = 0usize;
        for x in min_x..=max_x {
            let ch = if grid.get(x, y) { 'o' } else { 'b' };
            if run == 0 || ch != current {
                if run > 0 {
                    runs.push((run, current));
                }
                current = ch;
                run = 1;
            } else {
                run += 1;
            }
        }
        if run > 0 {
            runs.push((run, current));
        }
        // strip trailing dead-cell runs (RLE convention)
        while matches!(runs.last(), Some((_, 'b'))) {
            runs.pop();
        }
        // emit row, separated by $; empty rows still produce a $.
        for (n, c) in &runs {
            row_runs.push((*n, *c));
        }
        row_runs.push((1, '$'));
    }
    // final $ collapses into !; trim trailing $ runs.
    while matches!(row_runs.last(), Some((_, '$'))) {
        row_runs.pop();
    }
    row_runs.push((1, '!'));

    // build body, soft-wrapping at ~70 chars.
    let mut line_len = 0usize;
    for (n, c) in row_runs {
        let token = if n == 1 {
            c.to_string()
        } else {
            format!("{n}{c}")
        };
        if line_len + token.len() > 70 {
            body.push('\n');
            line_len = 0;
        }
        line_len += token.len();
        body.push_str(&token);
    }

    format!("x = {w}, y = {h}, rule = B3/S23\n{body}\n")
}

fn bounding_box(grid: &Grid) -> Option<(usize, usize, usize, usize)> {
    let mut found = false;
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if grid.get(x, y) {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if found {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Decode an RLE document into a list of (x, y) live-cell offsets relative to (0, 0).
pub fn decode(rle: &str) -> Result<Vec<(i32, i32)>> {
    let body: String = rle
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip_while(|l| l.trim_start().starts_with('x'))
        .collect::<Vec<_>>()
        .join("");

    if body.is_empty() {
        return Err(anyhow!("RLE document has no body"));
    }

    let mut cells = Vec::new();
    let mut x = 0i32;
    let mut y = 0i32;
    let mut count_str = String::new();

    for ch in body.chars() {
        if ch.is_ascii_digit() {
            count_str.push(ch);
            continue;
        }
        let n: i32 = if count_str.is_empty() {
            1
        } else {
            count_str
                .parse()
                .with_context(|| format!("bad run-length: {count_str}"))?
        };
        count_str.clear();

        match ch {
            'b' => x += n,
            'o' => {
                for _ in 0..n {
                    cells.push((x, y));
                    x += 1;
                }
            }
            '$' => {
                y += n;
                x = 0;
            }
            '!' => break,
            c if c.is_whitespace() => {}
            other => return Err(anyhow!("unexpected char in RLE body: {other:?}")),
        }
    }

    Ok(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_glider() {
        let mut g = Grid::new(10, 10);
        for &(x, y) in &[(2, 1), (3, 2), (1, 3), (2, 3), (3, 3)] {
            g.set(x, y, true);
        }
        let rle = encode_grid(&g);
        let cells = decode(&rle).expect("decode");
        // The cells are relative to the bounding-box origin (1, 1).
        let normalized: std::collections::BTreeSet<_> = cells.into_iter().collect();
        let expected: std::collections::BTreeSet<_> =
            [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)].into_iter().collect();
        assert_eq!(normalized, expected);
    }

    #[test]
    fn decode_blinker() {
        let cells = decode("x = 3, y = 1, rule = B3/S23\n3o!\n").unwrap();
        assert_eq!(cells, vec![(0, 0), (1, 0), (2, 0)]);
    }

    #[test]
    fn decode_skips_comments() {
        let rle = "#N glider\n#C generated\nx = 3, y = 3\nbo$2bo$3o!\n";
        let cells = decode(rle).unwrap();
        let set: std::collections::BTreeSet<_> = cells.into_iter().collect();
        let expected: std::collections::BTreeSet<_> =
            [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)].into_iter().collect();
        assert_eq!(set, expected);
    }
}
