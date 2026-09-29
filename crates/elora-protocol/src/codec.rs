//! Byte-Kodierung: variable Ganzzahl-Länge (`LEB128`), `ZigZag` für Vorzeichen.

/// Fehler beim Dekodieren. Jedes fehlerhafte Paket wird abgelehnt, nie „geraten“.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    #[error("Daten zu kurz")]
    UnexpectedEnd,
    #[error("Zahl zu groß")]
    Overflow,
    #[error("ungültiger Wert: {0}")]
    Invalid(&'static str),
    #[error("ungültiges UTF-8")]
    Utf8,
    #[error("überzählige Bytes am Ende")]
    TrailingBytes,
}

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, Default, Clone)]
pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    /// Vorzeichenlose Zahl, 7 Bit pro Byte.
    pub fn uvar(&mut self, mut v: u64) {
        loop {
            let byte = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                self.buf.push(byte);
                return;
            }
            self.buf.push(byte | 0x80);
        }
    }

    /// Vorzeichenbehaftete Zahl (`ZigZag`: kleine Beträge = wenige Bytes).
    pub fn ivar(&mut self, v: i64) {
        #[allow(clippy::cast_sign_loss)]
        self.uvar(((v << 1) ^ (v >> 63)) as u64);
    }

    pub fn bytes(&mut self, v: &[u8]) {
        self.uvar(v.len() as u64);
        self.buf.extend_from_slice(v);
    }

    pub fn str(&mut self, v: &str) {
        self.bytes(v.as_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
    }
}

#[derive(Debug, Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Fehler, wenn noch Bytes übrig sind.
    ///
    /// # Errors
    /// [`DecodeError::TrailingBytes`]
    pub fn finish(&self) -> DecodeResult<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::TrailingBytes)
        }
    }

    /// # Errors
    /// Bei zu kurzen Daten.
    pub fn u8(&mut self) -> DecodeResult<u8> {
        let v = *self.data.get(self.pos).ok_or(DecodeError::UnexpectedEnd)?;
        self.pos += 1;
        Ok(v)
    }

    /// # Errors
    /// Bei zu kurzen Daten oder einem Wert außer 0/1.
    pub fn bool(&mut self) -> DecodeResult<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::Invalid("bool")),
        }
    }

    /// # Errors
    /// Bei zu kurzen Daten oder mehr als 64 Bit.
    pub fn uvar(&mut self) -> DecodeResult<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = self.u8()?;
            let part = u64::from(byte & 0x7f);
            if shift == 63 && part > 1 {
                return Err(DecodeError::Overflow);
            }
            v |= part << shift;
            if byte & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(DecodeError::Overflow)
    }

    /// # Errors
    /// Wie [`Self::uvar`].
    pub fn ivar(&mut self) -> DecodeResult<i64> {
        let u = self.uvar()?;
        #[allow(clippy::cast_possible_wrap)]
        Ok(((u >> 1) as i64) ^ -((u & 1) as i64))
    }

    /// Zahl mit Bereichsprüfung.
    ///
    /// # Errors
    /// Wenn der Wert nicht in den Zieltyp passt.
    pub fn int<T: TryFrom<i64>>(&mut self, what: &'static str) -> DecodeResult<T> {
        T::try_from(self.ivar()?).map_err(|_| DecodeError::Invalid(what))
    }

    /// # Errors
    /// Wenn der Wert nicht in den Zieltyp passt.
    pub fn uint<T: TryFrom<u64>>(&mut self, what: &'static str) -> DecodeResult<T> {
        T::try_from(self.uvar()?).map_err(|_| DecodeError::Invalid(what))
    }

    /// # Errors
    /// Bei zu kurzen Daten oder Länge über `max`.
    pub fn bytes(&mut self, max: usize) -> DecodeResult<&'a [u8]> {
        let len: usize = self.uint("Länge")?;
        if len > max {
            return Err(DecodeError::Invalid("Länge"));
        }
        let end = self.pos.checked_add(len).ok_or(DecodeError::Overflow)?;
        let v = self
            .data
            .get(self.pos..end)
            .ok_or(DecodeError::UnexpectedEnd)?;
        self.pos = end;
        Ok(v)
    }

    /// # Errors
    /// Bei zu kurzen Daten, Länge über `max` oder ungültigem UTF-8.
    pub fn str(&mut self, max: usize) -> DecodeResult<&'a str> {
        std::str::from_utf8(self.bytes(max)?).map_err(|_| DecodeError::Utf8)
    }

    /// # Errors
    /// Bei zu kurzen Daten.
    pub fn f32(&mut self) -> DecodeResult<f32> {
        let end = self.pos + 4;
        let b = self
            .data
            .get(self.pos..end)
            .ok_or(DecodeError::UnexpectedEnd)?;
        self.pos = end;
        Ok(f32::from_bits(u32::from_le_bytes([b[0], b[1], b[2], b[3]])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_roundtrip_and_are_compact() {
        let values = [
            0i64,
            1,
            -1,
            63,
            -64,
            64,
            1000,
            -1000,
            i64::from(i32::MAX),
            i64::MIN,
            i64::MAX,
        ];
        let mut w = Writer::new();
        for v in values {
            w.ivar(v);
        }
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        for v in values {
            assert_eq!(r.ivar().unwrap(), v);
        }
        r.finish().unwrap();

        let mut w = Writer::new();
        w.ivar(-3);
        assert_eq!(w.buf.len(), 1, "kleine Beträge brauchen 1 Byte");
    }

    #[test]
    fn rejects_truncated_and_oversized() {
        assert_eq!(Reader::new(&[0x80]).uvar(), Err(DecodeError::UnexpectedEnd));
        let too_long = [0xff; 11];
        assert!(Reader::new(&too_long).uvar().is_err());
        let mut w = Writer::new();
        w.str("hallo");
        let b = w.into_bytes();
        assert_eq!(Reader::new(&b).str(3), Err(DecodeError::Invalid("Länge")));
        assert_eq!(Reader::new(&b).str(10).unwrap(), "hallo");
        assert!(Reader::new(&[2]).bool().is_err());
    }
}
