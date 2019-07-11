use crate::label_sequence::LabelSequence;
use crate::name::Name;
use crate::name::NameComparisonResult;
use crate::name::NameRelation;
use std::cmp;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LabelSlice<'a> {
    data: &'a [u8],
    offsets: &'a [u8],
    first_label: usize,
    last_label: usize,
}

impl<'a> LabelSlice<'a> {
    pub fn from_name(name: &'a Name) -> LabelSlice {
        LabelSlice {
            data: name.raw.as_slice(),
            offsets: name.offsets.as_slice(),
            first_label: 0,
            last_label: usize::from(name.label_count - 1),
        }
    }

    pub fn from_label_sequence(ls: &'a LabelSequence) -> LabelSlice {
        LabelSlice {
            data: ls.get_data(),
            offsets: ls.get_offsets(),
            first_label: 0,
            last_label: ls.get_label_count() - 1,
        }
    }

    pub fn into_label_sequence(self) -> LabelSequence {
        if self.first_label == 0 {
            LabelSequence::new(self.get_data().to_vec(), self.get_offsets().to_vec())
        } else {
            let curr_label_value = self.offsets[self.first_label];
            let mut offsets = self.get_offsets().to_vec();
            for v in &mut offsets {
                *v -= curr_label_value;
            }
            LabelSequence::new(self.get_data().to_vec(), offsets)
        }
    }

    pub fn get_offsets(&self) -> &'a [u8] {
        &self.offsets[self.first_label..self.last_label + 1]
    }

    pub fn get_data(&self) -> &'a [u8] {
        let first_label_index: usize = usize::from(self.offsets[self.first_label]);
        &self.data[first_label_index..first_label_index + self.get_data_length() + 1]
    }

    pub fn get_data_length(&self) -> usize {
        let last_label_len: u8 = self.data[usize::from(self.offsets[self.last_label])];
        usize::from(self.offsets[self.last_label] - self.offsets[self.first_label] + last_label_len)
    }

    pub fn equals(&self, other: &LabelSlice, case_sensitive: bool) -> bool {
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
            data.eq_ignore_ascii_case(other_data);
        }
        true
    }

    pub fn get_label_count(&self) -> usize {
        self.last_label - self.first_label + 1
    }

    pub fn compare(&self, other: &LabelSlice, case_sensitive: bool) -> NameComparisonResult {
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
            println!("pos 1 {}", pos1);
            let mut pos2: usize = usize::from(other.offsets[l2 + other.first_label]);
            println!("pos 2 {}", pos1);
            let count1: usize = usize::from(self.data[pos1]);
            let count2: usize = usize::from(other.data[pos2]);
            pos1 += 1;
            pos2 += 1;
            let cdiff: isize = (count1 as isize) - (count2 as isize);
            let mut count = cmp::min(count1, count2);

            while count > 0 {
                println!("data 11 {:?}", self.data);
                println!("data 22 {:?}", self.data);
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
mod test {
    use super::*;
    use crate::name::Name;
    #[test]
    fn test_label_sequence_new() {
        //0377777705626169647503636f6d00
        let n1 = Name::new("www.baidu.com.").unwrap();
        let ls1 = LabelSlice::from_name(&n1);
        assert_eq!(
            ls1.data,
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 109, 0]
        );
        assert_eq!(ls1.offsets, [0, 4, 10, 14]);
        assert_eq!(ls1.first_label, 0);
        assert_eq!(ls1.last_label, 3);

        let n2 = Name::new("www.baidu.coM.").unwrap();
        let ls2 = LabelSlice::from_name(&n2);
        assert_eq!(
            ls2.data,
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 77, 0]
        );
        assert_eq!(ls2.offsets, [0, 4, 10, 14]);
        assert_eq!(ls2.first_label, 0);
        assert_eq!(ls2.last_label, 3);

        assert_eq!(
            ls1.get_data(),
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 109, 0]
        );
        assert_eq!(
            ls2.get_data(),
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 77, 0]
        );
        assert_eq!(ls1.equals(&ls2, false), true);
        assert_eq!(ls1.equals(&ls2, true), false);
        let grand_parent = Name::new("com").unwrap();
        let ls_grand_parent = LabelSlice::from_name(&grand_parent);
        let parent = Name::new("BaIdU.CoM").unwrap();
        let ls_parent = LabelSlice::from_name(&parent);
        let child = Name::new("wWw.bAiDu.cOm").unwrap();
        let mut ls_child = LabelSlice::from_name(&child);
        let brother = Name::new("AaA.bAiDu.cOm").unwrap();
        let ls_brother = LabelSlice::from_name(&brother);
        let other = Name::new("aAa.BaIdu.cN").unwrap();
        let mut ls_other = LabelSlice::from_name(&other);
        assert_eq!(
            ls_grand_parent.compare(&ls_parent, false).relation,
            NameRelation::SuperDomain
        );
        assert_eq!(
            ls_parent.compare(&ls_child, false).relation,
            NameRelation::SuperDomain
        );
        assert_eq!(
            ls_child.compare(&ls_parent, false).relation,
            NameRelation::SubDomain
        );
        assert_eq!(
            ls_child.compare(&ls_grand_parent, false).relation,
            NameRelation::SubDomain
        );
        assert_eq!(
            ls_child.compare(&ls_brother, false).relation,
            NameRelation::CommonAncestor
        );
        assert_eq!(
            ls_child.compare(&ls_child, false).relation,
            NameRelation::Equal
        );
        ls_child.strip_left(1);
        ls_other.strip_left(1);
        assert_eq!(
            ls_child.compare(&ls_other, false).relation,
            NameRelation::CommonAncestor
        );
        ls_child.strip_right(1);
        ls_other.strip_right(1);
        assert_eq!(
            ls_child.compare(&ls_other, false).relation,
            NameRelation::None
        );
        let ls_name = Name::new("1.www.google.com").unwrap();
        let mut ls_slice = LabelSlice::from_name(&ls_name);

        ls_slice.strip_left(1);
        ls_slice.strip_right(1);
        let ls_sequence = ls_slice.into_label_sequence();
        let ls_slice2 = LabelSlice::from_label_sequence(&ls_sequence);

        let ls_name_2 = Name::new("www.google.com.").unwrap();
        let mut ls_slice3 = LabelSlice::from_name(&ls_name_2);
        ls_slice3.strip_right(1);
        assert_eq!(
            ls_slice2.compare(&ls_slice3, false).relation,
            NameRelation::Equal
        );
    }
}
