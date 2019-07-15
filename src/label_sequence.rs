use crate::error::DNSError;
#[derive(Debug, Clone)]
pub struct LabelSequence {
    data: Vec<u8>,
    offsets: Vec<u8>,
}

impl LabelSequence {
    pub fn new(data: Vec<u8>, offsets: Vec<u8>) -> LabelSequence {
        LabelSequence { data, offsets }
    }
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn offsets(&self) -> &[u8] {
        self.offsets.as_slice()
    }

    pub fn data_length(&self) -> usize {
        self.data.len()
    }

    pub fn equals(&self, other: &LabelSequence, case_sensitive: bool) -> bool {
        let data = self.data();
        let other_data = other.data();
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

    pub fn label_count(&self) -> usize {
        self.offsets.len()
    }

    pub fn split(
        &mut self,
        start_label: usize,
        label_count_: usize,
    ) -> Result<LabelSequence, DNSError> {
        let max_label_count = self.label_count() as usize;
        if start_label >= max_label_count || label_count_ == 0 {
            return Err(DNSError::InvalidLabelIndex.into());
        }
        let mut label_count = label_count_;
        if start_label + label_count > max_label_count {
            label_count = max_label_count - start_label;
        }

        let last_label = start_label + label_count - 1;
        let last_label_len: u8 = self.data[usize::from(self.offsets[last_label])] + 1;
        let data_length: u8 = self.offsets[last_label] + last_label_len;
        let data_offset: u8 = data_length - self.offsets[start_label];
        let data: Vec<u8> = self
            .data
            .drain(self.offsets[start_label] as usize..data_length as usize)
            .collect();
        let mut offsets: Vec<u8> = self.offsets.drain(start_label..=last_label).collect();

        if start_label == 0 {
            for v in &mut self.offsets {
                *v -= data_offset;
            }
        } else {
            let mut index = 0;
            for v in &mut self.offsets {
                if index >= start_label {
                    *v -= data_offset;
                }
                index += 1;
            }
            let curr_label_value = offsets[0];
            for v in &mut offsets {
                *v -= curr_label_value;
            }
        }

        Ok(LabelSequence { data, offsets })
    }
}

#[cfg(test)]
mod test {
    use crate::name::Name;
    #[test]
    fn test_label_sequence_split() {
        let www_google_com_cn_ = Name::new("www.google.com.cn.").unwrap();
        let mut ls_www_google_com_cn_ = www_google_com_cn_.into_label_sequence(0, 4);
        let ls_www = ls_www_google_com_cn_.split(0, 1).unwrap();
        assert_eq!(ls_www.data(), [3, 119, 119, 119]);
        assert_eq!(ls_www.offsets(), [0]);
        assert_eq!(
            ls_www_google_com_cn_.data(),
            [6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 2, 99, 110, 0]
        );
        assert_eq!(ls_www_google_com_cn_.offsets(), [0, 7, 11, 14]);
    }
}
