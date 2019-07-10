use crate::label_slice::LabelSlice;
use crate::name::Name;
use crate::name::NameComparisonResult;
use crate::name::NameRelation;
use std::cmp;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LabelSequence {
    data: Vec<u8>,
    offsets: Vec<u8>,
    first_label: usize,
    last_label: usize,
}

impl LabelSequence {
    pub fn new(ls: LabelSlice, name: Name) -> LabelSequence {
        LabelSequence {
            data: name.raw,
            offsets: name.offsets,
            first_label: 0,
            last_label: usize::from(name.label_count - 1),
        }
    }

    pub fn get_data(&self) -> &[u8] {
        let first_label_index: usize = usize::from(self.offsets[self.first_label]);
        &self.data[first_label_index..self.get_data_length()]
    }

    pub fn get_offset(&self) -> &[u8] {
        &self.offsets[0..]
    }

    pub fn get_data_length(&self) -> usize {
        let last_label_len: u8 = self.data[usize::from(self.offsets[self.last_label])] + 1;
        usize::from(self.offsets[self.last_label] - self.offsets[self.first_label] + last_label_len)
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
        self.last_label - self.first_label + 1
    }

    pub fn compare(&self, other: &LabelSequence, case_sensitive: bool) -> NameComparisonResult {
        let mut nlabels: usize = 0;
        let mut l1: usize = self.get_label_count();
        let mut l2: usize = other.get_label_count();
        let ldiff = (l1 as isize) - (l2 as isize);
        let mut l = cmp::min(l1, l2);

        while l > 0 {
            l -= 1;
            l1 -= 1;
            l2 -= 1;
            let mut pos1: usize = usize::from(self.offsets[l1 + self.first_label]);
            let mut pos2: usize = usize::from(other.offsets[l2 + other.first_label]);
            let count1: usize = usize::from(self.data[pos1]);
            let count2: usize = usize::from(other.data[pos2]);
            pos1 += 1;
            pos2 += 1;
            let cdiff: isize = (count1 as isize) - (count2 as isize);
            let mut count = cmp::min(count1, count2);

            while count > 0 {
                let label1: u8 = self.data[pos1];
                let label2: u8 = other.data[pos2];
                let mut chdiff: bool = true;
                if case_sensitive {
                    if (label1 - label2) != 0 {
                        chdiff = false;
                    }
                } else {
                    chdiff = label1.eq_ignore_ascii_case(&label2);
                }
                if !chdiff {
                    return NameComparisonResult {
                        order: (label1 as i8) - (label2 as i8),
                        common_label_count: nlabels as u8,
                        relation: if nlabels == 0 {
                            NameRelation::None
                        } else {
                            NameRelation::CommonAncestor
                        },
                    };
                }
                count -= 1;
                pos1 += 1;
                pos2 += 1;
            }
            if cdiff != 0 {
                return NameComparisonResult {
                    order: cdiff as i8,
                    common_label_count: nlabels as u8,
                    relation: if nlabels == 0 {
                        NameRelation::None
                    } else {
                        NameRelation::CommonAncestor
                    },
                };
            }
            nlabels += 1;
        }

        NameComparisonResult {
            order: ldiff as i8,
            common_label_count: nlabels as u8,
            relation: if ldiff < 0 {
                NameRelation::SuperDomain
            } else if ldiff > 0 {
                NameRelation::SubDomain
            } else {
                NameRelation::Equal
            },
        }
    }

    pub fn strip_left(&mut self, index: usize) {
        assert!(index < self.get_label_count());
        self.first_label += index;
    }

    pub fn strip_right(&mut self, index: usize) {
        assert!(index < self.get_label_count());
        self.last_label -= index;
    }
}

#[cfg(test)]
mod test {}
