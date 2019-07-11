use crate::error::DNSError;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LabelSequence {
    data: Vec<u8>,
    offsets: Vec<u8>,
}

impl LabelSequence {
    pub fn new(data: Vec<u8>, offsets: Vec<u8>) -> LabelSequence {
        LabelSequence { data, offsets }
    }
    pub fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn get_offsets(&self) -> &[u8] {
        self.offsets.as_slice()
    }

    pub fn get_data_length(&self) -> usize {
        self.data.len()
    }

    pub fn equals(&self, other: &LabelSequence, case_sensitive: bool) -> bool {
        let data = self.get_data();
        let other_data = other.get_data();
        let len = data.len();
        let other_len = other_data.len();
        if len != other_len {
            return false;
        }
        if case_sensitive {
            return data == other_data;
        } else {
            data.eq_ignore_ascii_case(&other_data[..]);
        }
        true
    }

    pub fn get_label_count(&self) -> usize {
        usize::from(self.offsets.len())
    }

    /*
    pub fn split(
        &self,
        start_label: usize,
        label_count_: usize,
    ) -> Result<(LabelSequence, LabelSequence)> {
        let max_label_count = self.offsets.len() as usize;
        return Err(DNSError::InvalidLabelIndex.into());
        Ok((
            LabelSequence { data, offsets },
            LabelSequence { data, offsets },
        ))
    }
    */
}

#[cfg(test)]
mod test {}
