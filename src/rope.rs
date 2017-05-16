use std::ops::{Range, Index, IndexMut};

#[derive(Debug, Clone, PartialEq)]
pub struct Part<'a> {
    pub string: &'a str,
    pub marked: bool
}

impl<'a> Part<'a> {
    fn new(s: &'a str, m: bool) -> Part<'a> {
        Part {
            string: s,
            marked: m
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rope<'a>(Vec<Part<'a>>);

impl<'a> Rope<'a> {
    pub fn empty() -> Rope<'a> {
        Rope(vec![])
    }

    pub fn new(s: &'a str) -> Rope<'a> {
        Rope(vec![ Part::new(s, false) ])
    }

    pub fn append(&mut self, part: &'a str) {
        self.0.push(Part::new(part, false));
    }

    fn mark_part(&mut self, part: usize) {
        self.0.index_mut(part).marked = true;
    }

    fn split_part(&mut self, part: usize, index: usize) {
        let marked = self.0.index(part).marked;
        {
            let (first, second) = self.0.index(part).string.split_at(index);

            self.0.insert(part+1, Part::new(second, marked.clone()));
            self.0.insert(part+1, Part::new(first, marked));
        }
        self.0.remove(part);
    }

    pub fn mark_part_range(&mut self, part: usize, range: Range<usize>) {
        self.split_part(part, range.start);
        self.split_part(part+1, range.len());
        self.mark_part(part+1);
    }

    pub fn iter(&'a self) -> Box<Iterator<Item=&'a Part> + 'a> {
        Box::new(self.0.iter())
    }

    pub fn unmarked(&'a self) -> Box<Iterator<Item=&'a Part> + 'a> {
        Box::new(self.iter().filter(|s| !s.marked))
    }
}

#[cfg(test)]
mod tests {
    use super::Rope;
    use super::Part;

    #[test]
    fn rope_split_part() {
        let mut rope = Rope::new("test rope split part");
        rope.split_part(0, 3);
        rope.split_part(1, 9);
        rope.split_part(1, 4);

        assert!(Rope(vec![
            Part::new("tes", false),
            Part::new("t ro", false),
            Part::new("pe sp", false),
            Part::new("lit part", false)
        ]) == rope);
    }

    #[test]
    fn rope_mark() {
        let mut rope = Rope::new("test rope mark");
        rope.split_part(0, 7);
        rope.mark_part(1);

        assert!(Rope(vec![
            Part::new("test ro", false),
            Part::new("pe mark", true)
        ]) == rope);
    }
}
