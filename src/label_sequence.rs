use crate::error::DNSError;
use crate::name::{self, Name};
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

    pub fn len(&self) -> usize {
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

    pub fn concat_all(&self, suffixes: &[&LabelSequence]) -> Result<Name, DNSError> {
        let mut final_length = self.len();
        let mut final_label_count = self.label_count();
        let suffix_count = suffixes.len();
        for suffix in suffixes {
            final_length += suffix.len();
            final_label_count += suffix.label_count();
        }

        if final_length > name::MAX_WIRE_LEN {
            return Err(DNSError::TooLongName.into());
        } else if final_label_count > name::MAX_LABEL_COUNT as usize {
            return Err(DNSError::TooLongLabel.into());
        }

        let mut data = Vec::with_capacity(final_length as usize);
        data.extend_from_slice(&self.data[..(self.len() as usize)]);
        for suffix in &suffixes[..(suffix_count as usize - 1)] {
            data.extend_from_slice(&suffix.data[..(suffix.len() as usize)])
        }
        data.extend_from_slice(&(suffixes[suffix_count - 1].data[..]));

        let mut offsets = Vec::with_capacity(final_label_count as usize);
        offsets.extend_from_slice(&self.offsets[..]);
        let mut next_label_index = self.label_count();
        for suffix in suffixes {
            offsets.extend_from_slice(&suffix.offsets[0..(suffix.label_count() as usize)]);
            for i in next_label_index..(next_label_index + suffix.label_count()) {
                let last_offset = offsets[next_label_index as usize - 1];
                offsets[i as usize] = last_offset + data[offsets[i - 1] as usize] + 1;
                next_label_index += 1;
            }
        }

        Ok(Name::from_raw(data, offsets))
    }
}

#[cfg(test)]
mod test {
    use crate::label_slice::LabelSlice;
    use crate::name::Name;
    use crate::name::NameRelation;
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

    #[test]
    fn test_label_sequence_concat_all() {
        let cn_ = Name::new("cn.").unwrap();
        let sli_cn_ = LabelSlice::from_name(&cn_);
        let cn_first_label = sli_cn_.first_label();
        let cn_last_label = sli_cn_.last_label();
        let seq_cn_ = cn_.into_label_sequence(cn_first_label, cn_last_label);
        assert_eq!(cn_first_label, 0);
        assert_eq!(cn_last_label, 1);

        let com_ = Name::new("com.").unwrap();
        let mut sli_com = LabelSlice::from_name(&com_);
        sli_com.strip_right(1);
        let com_first_label = sli_com.first_label();
        let com_last_label = sli_com.last_label();
        let seq_com = com_.into_label_sequence(com_first_label, com_last_label);
        assert_eq!(com_first_label, 0);
        assert_eq!(com_last_label, 0);

        let google_ = Name::new("google.").unwrap();
        let mut sli_google = LabelSlice::from_name(&google_);
        sli_google.strip_right(1);
        let google_first_label = sli_google.first_label();
        let google_last_label = sli_google.last_label();
        let seq_google = google_.into_label_sequence(google_first_label, google_last_label);
        assert_eq!(google_first_label, 0);
        assert_eq!(google_last_label, 0);

        let www_ = Name::new("www.").unwrap();
        let mut sli_www = LabelSlice::from_name(&www_);
        sli_www.strip_right(1);
        let www_first_label = sli_www.first_label();
        let www_last_label = sli_www.last_label();
        let seq_www = www_.into_label_sequence(www_first_label, www_last_label);
        assert_eq!(www_first_label, 0);
        assert_eq!(www_last_label, 0);

        let www_google_com_cn_ = Name::new("www.google.com.cn.").unwrap();
        let test_name = seq_www
            .concat_all(&[&seq_google, &seq_com, &seq_cn_])
            .unwrap();
        let relation = www_google_com_cn_.get_relation(&test_name);
        assert_eq!(relation.order, 0);
        assert_eq!(relation.common_label_count, 5);
        assert_eq!(relation.relation, NameRelation::Equal);

        let www_baidu_ = Name::new("www.baidu.").unwrap();
        let mut sli_www_baidu_ = LabelSlice::from_name(&www_baidu_);
        sli_www_baidu_.strip_right(1);
        let www_baidu_first_label = sli_www_baidu_.first_label();
        let www_baidu_last_label = sli_www_baidu_.last_label();
        let seq_www_baidu =
            www_baidu_.into_label_sequence(www_baidu_first_label, www_baidu_last_label);
        assert_eq!(www_baidu_first_label, 0);
        assert_eq!(www_baidu_last_label, 1);

        let cn_ = Name::new("cn.").unwrap();
        let sli_cn_ = LabelSlice::from_name(&cn_);
        let cn_first_label = sli_cn_.first_label();
        let cn_last_label = sli_cn_.last_label();
        let seq_cn_ = cn_.into_label_sequence(cn_first_label, cn_last_label);
        assert_eq!(cn_first_label, 0);
        assert_eq!(cn_last_label, 1);

        let www_baidu_cn_ = Name::new("www.baidu.cn.").unwrap();
        let test_baidu = seq_www_baidu.concat_all(&[&seq_cn_]).unwrap();
        let baidu_relation = www_baidu_cn_.get_relation(&test_baidu);
        assert_eq!(baidu_relation.order, 0);
        assert_eq!(baidu_relation.common_label_count, 4);
        assert_eq!(baidu_relation.relation, NameRelation::Equal);
    }
}
