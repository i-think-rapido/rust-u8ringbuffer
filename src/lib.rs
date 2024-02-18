
#![allow(dead_code)]

pub struct U8RingBuffer {
    ring: Vec<u8>,
    buffer: Vec<u8>,
    len: usize,
    pos: usize,
}

impl U8RingBuffer {

    pub fn new(capacity: usize) -> Self {
        let mut ring = Vec::with_capacity(capacity);
        unsafe {
            ring.set_len(capacity);
        }

        Self {
            ring: ring.clone(),
            buffer: ring,
            len: 0,
            pos: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.ring.capacity()
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn last(&self) -> usize {
        let capacity = self.capacity();
        if self.len == capacity { self.pos }
        else {
            let diff = self.pos as isize - self.len as isize;
            if diff >= 0 { diff as usize }
            else { (capacity as isize + diff) as usize }
        }
    }
    pub fn clear(&mut self) {
        self.len = 0;
        self.pos = 0;
    }

    pub fn push(&mut self, mut buffer: &[u8]) {
        let capacity = self.capacity();
        if buffer.len() > capacity {
            buffer = &buffer[buffer.len() - capacity..];
        }
        let pos = self.pos;
        if buffer.len() == capacity {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &buffer[0] as *const _ as *const u8,
                    &mut self.ring[pos] as *mut _ as *mut u8,
                    capacity - pos,
                );
                if pos != 0 {
                    std::ptr::copy_nonoverlapping(
                        &buffer[capacity - pos] as *const _ as *const u8,
                        &mut self.ring[0] as *mut _ as *mut u8,
                        pos,
                    );
                }
            }
            self.inc_pos_by(capacity);
        }
        else {
            if buffer.len() > capacity - pos {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        &buffer[0] as *const _ as *const u8,
                        &mut self.ring[pos] as *mut _ as *mut u8,
                        capacity - pos,
                    );
                    std::ptr::copy_nonoverlapping(
                        &buffer[buffer.len() - (capacity - pos) + 1] as *const _ as *const u8,
                        &mut self.ring[0] as *mut _ as *mut u8,
                        buffer.len() - (capacity - pos),
                    );
                }
            }
            else {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        &buffer[0] as *const _ as *const u8,
                        &mut self.ring[pos] as *mut _ as *mut u8,
                        buffer.len(),
                    )
                }
            }
            self.inc_pos_by(buffer.len());
        }
   }

    pub fn slice(&mut self) -> &[u8] {
        let capacity = self.capacity();
        let last = self.last();
        let len = self.len;
        let pos = self.pos;
        unsafe {
            if last < pos {
                std::ptr::copy_nonoverlapping(
                    &self.ring[last] as *const _ as *const u8,
                    &mut self.buffer[0] as *mut _ as *mut u8,
                    self.len,
                );
                &self.buffer[..len]
            }
            else if last > pos {
                std::ptr::copy_nonoverlapping(
                    &self.ring[pos] as *const _ as *const u8,
                    &mut self.buffer[0] as *mut _ as *mut u8,
                    capacity - pos,
                );
                if last != capacity {
                    std::ptr::copy_nonoverlapping(
                        &self.ring[0] as *const _ as *const u8,
                        &mut self.buffer[capacity - last] as *mut _ as *mut u8,
                        capacity - last,
                    );
                }
                &self.buffer[..len]
            }
            else {
                std::ptr::copy_nonoverlapping(
                    &self.ring[pos] as *const _ as *const u8,
                    &mut self.buffer[0] as *mut _ as *mut u8,
                    capacity - pos,
                );
                if pos != 0 {
                    std::ptr::copy_nonoverlapping(
                        &self.ring[0] as *const _ as *const u8,
                        &mut self.buffer[capacity - pos] as *mut _ as *mut u8,
                        pos,
                    );
                }
                &self.buffer[..]
            }
        }
    }

    fn inc_pos_by(&mut self, inc: usize) {
        let capacity = self.capacity();
        self.pos += inc;
        self.pos %= capacity;
        if self.len >= capacity { return; }
        self.len += inc;
        self.len = self.len.min(capacity);
    }

    fn occurence(&mut self, buffer: &[u8], offset: usize) -> Option<usize> {
        let slice = self.slice();
        let blen = buffer.len();
        let slen = slice.len();
        if offset + blen > slen { return None }
        for idx in offset..slen-blen {
            if buffer == &slice[idx..idx+blen] {
                return Some(idx)
            }
        }
        None
    }
    pub fn first_occurence(&mut self, buffer: &[u8]) -> Option<usize> {
        self.occurence(buffer, 0)
    }
    pub fn second_occurence(&mut self, buffer: &[u8]) -> Option<usize> {
        self.occurence(buffer, 0).map(|offset| self.occurence(buffer, offset + 1)).unwrap_or(None)
    }

    pub fn purge(&mut self, amount: usize) -> bool {
        if amount > self.len {
            return false;
        }
        if amount == self.len {
            self.len = 0;
            self.pos = 0;
        }
        else {
            self.len -= amount;
        }
        true
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        let buffer = U8RingBuffer::new(10);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 10);
    }

    #[test]
    fn test_push() {
        let mut buffer = U8RingBuffer::new(10);
        buffer.push(&[1,2,3]);
        assert_eq!(buffer.slice(), &[1,2,3]);
        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());

        buffer.push(&[4,5,6,7,8]);
        assert_eq!(buffer.slice(), &[1,2,3,4,5,6,7,8]);
        assert_eq!(buffer.len(), 8);
        assert!(!buffer.is_empty());

        buffer.push(&[9,10,11]);
        assert_eq!(buffer.slice(), &[2,3,4,5,6,7,8,9,10,11]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());

        buffer.push(&[12,13,14]);
        assert_eq!(buffer.slice(), &[5,6,7,8,9,10,11,12,13,14]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());

        buffer.push(&[15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30]);
        assert_eq!(buffer.slice(), &[21,22,23,24,25,26,27,28,29,30]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());

        buffer.push(&[15
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ]);
        assert_eq!(buffer.slice(), &[221,222,223,224,225,226,227,228,229,230]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_edge_cases() {
        let mut buffer = U8RingBuffer::new(10);
        buffer.push(&[2,3,4,5,6,7,8,9,10,11]);
        assert_eq!(buffer.slice(), &[2,3,4,5,6,7,8,9,10,11]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());

        let mut buffer = U8RingBuffer::new(10);
        buffer.push(&[1,2,3,4,5]);
        buffer.push(&[1,2,3,4,5]);
        buffer.push(&[1,2,3,4,5]);
        assert_eq!(buffer.slice(), &[1,2,3,4,5,1,2,3,4,5]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());

        let mut buffer = U8RingBuffer::new(10);
        buffer.push(&[15
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ]);
        assert_eq!(buffer.slice(), &[221,222,223,224,225,226,227,228,229,230]);
        assert_eq!(buffer.len(), 10);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_occurences() {
        let mut buffer = U8RingBuffer::new(100);
        buffer.push(&[15
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ]);

        assert_eq!(buffer.first_occurence(&[120, 122]), None);
        assert_eq!(buffer.second_occurence(&[120, 122]), None);
        assert_eq!(buffer.first_occurence(&[121, 122]), Some(6));
        assert_eq!(buffer.second_occurence(&[121, 122]), Some(36));
        assert_eq!(buffer.len(), 91);
        assert_eq!(buffer.purge(6), true);
        assert_eq!(buffer.len(), 85);
        assert_eq!(buffer.slice(), &[
                                    121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
            ,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130
            ,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230
        ]);
        assert_eq!(buffer.purge(100), false);
    }

    #[test]
    fn test_clean() {
        let mut buffer = U8RingBuffer::new(5);
        buffer.push(&[1,2,3]);
        buffer.clear();
        buffer.push(&[4,5,6]);
        assert_eq!(buffer.slice(), &[4, 5, 6]);
        buffer.push(&[7, 8, 9]);
        assert_eq!(buffer.slice(), &[5, 6, 7, 8, 9]);
        buffer.clear();
        buffer.push(&[4,5,6]);
        assert_eq!(buffer.slice(), &[4, 5, 6]);
    }
}
