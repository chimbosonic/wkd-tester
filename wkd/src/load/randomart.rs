//! Code adapted from https://github.com/RustCrypto/SSH/blob/master/ssh-key/src/fingerprint.rs (LICENSE: MIT OR Apache-2.0)
use super::Fingerprint;
use std::io::Write;

const WIDTH: usize = 17;
const HEIGHT: usize = 9;
const VALUES: &[u8; 17] = b" .o+=*BOX@%&#/^SE";
const NVALUES: u8 = VALUES.len() as u8 - 1;

type Field = [[u8; WIDTH]; HEIGHT];

#[cfg_attr(feature = "tracing", tracing::instrument)]
pub fn generate_randomart(
    fingerprint: &Fingerprint,
    key_algorithm: &str,
) -> Result<String, std::io::Error> {
    let mut field = Field::default();
    let mut x = WIDTH / 2;
    let mut y = HEIGHT / 2;

    for mut byte in fingerprint.as_bytes().iter().copied() {
        for _ in 0..4 {
            if byte & 0x1 == 0 {
                x = x.saturating_sub(1);
            } else {
                x = x.saturating_add(1);
            }

            if byte & 0x2 == 0 {
                y = y.saturating_sub(1);
            } else {
                y = y.saturating_add(1);
            }

            x = x.min(WIDTH.saturating_sub(1));
            y = y.min(HEIGHT.saturating_sub(1));

            if field[y][x] < NVALUES - 2 {
                field[y][x] = field[y][x].saturating_add(1);
            }

            byte >>= 2;
        }
    }

    field[HEIGHT / 2][WIDTH / 2] = NVALUES - 1;
    field[y][x] = NVALUES;

    let header = format!("[{}]", key_algorithm);
    let footer = format!("[{}]", fingerprint.algorithm);

    let mut result = Vec::new();
    writeln!(result, "+{:-^width$}+", header, width = WIDTH)?;

    for row in field {
        write!(result, "|")?;

        for c in row {
            write!(result, "{}", VALUES[c as usize] as char)?;
        }

        writeln!(result, "|")?;
    }

    write!(result, "+{:-^width$}+", footer, width = WIDTH)?;

    String::from_utf8(result).map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {

    const EXAMPLE_FINGERPRINT: &str =
        "5025222ebecf8ecf7014524c0c1c8b81cdcdaed754df8e0e814338e7064f7084";
    const EXAMPLE_RANDOMART: &str = "\
+--[ED25519 256]--+
|o+oO==+ o..      |
|.o++Eo+o..       |
|. +.oO.o . .     |
| . o..B.. . .    |
|  ...+ .S. o     |
|  .o. . . . .    |
|  o..    o       |
|   B      .      |
|  .o*            |
+----[SHA256]-----+";

    use super::*;
    use crate::load::Fingerprint;
    #[test]
    fn generation() {
        let key_algorithm = "ED25519 256".to_string();

        let fingerprint = Fingerprint {
            fingerprint: hex::decode(EXAMPLE_FINGERPRINT).unwrap(),

            algorithm: "SHA256".to_string(),
        };

        let randomart = generate_randomart(&fingerprint, &key_algorithm).unwrap();
        assert_eq!(EXAMPLE_RANDOMART, randomart);
    }
}
