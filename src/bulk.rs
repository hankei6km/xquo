use std::io::{BufRead, BufReader};

pub struct BulkReader<T> {
    reader: BufReader<T>,
    size: usize,
}

impl<T> BulkReader<T>
where
    T: std::io::Read,
{
    //fn new(reader: BufReader<Box<dyn std::io::Read>>) -> BlkRead {
    pub fn new(reader: T, size: usize) -> BulkReader<T> {
        let r = BufReader::<T>::new(reader);
        BulkReader { reader: r, size }
    }
    pub fn read(&mut self, byte: u8) -> (Vec<Vec<u8>>, usize) {
        let mut bulk = Vec::<Vec<u8>>::new();
        let mut line_cnt = 0usize;
        loop {
            let mut buf = Vec::<u8>::new();
            if self.reader.read_until(byte, &mut buf).unwrap() == 0 {
                break;
            }
            bulk.push(buf); // push はどれくらいコストがかかる?
            line_cnt += 1;
            if line_cnt >= self.size {
                break;
            }
        }
        (bulk, line_cnt)
    }
}

#[cfg(test)]
mod tests {
    use crate::bulk::BulkReader;

    fn lines_to_bulk(src: &[&str], trim: bool) -> Vec<Vec<u8>> {
        let mut ret = Vec::<Vec<u8>>::new();
        let len = src.len();
        if len == 0 {
            return ret;
        } else if len == 1 {
            ret.push(src[0].as_bytes().to_vec());
            return ret;
        } else if trim {
            src[0..len - 1].iter().for_each(|v| {
                ret.push(format!("{}\0", v).as_bytes().to_vec());
            });
            ret.push(src[len - 1].as_bytes().to_vec());
            return ret;
        }
        for v in src {
            ret.push(format!("{}\0", v).as_bytes().to_vec());
        }
        ret
    }

    #[test]
    fn single_line() {
        let bulk_size = 10;
        let s = ["aa"];
        let lines = s.join("\0");
        //let ex_lines = s.as_slice();

        let file = lines.as_bytes();
        let mut r = BulkReader::new(file, bulk_size);

        let (b, s) = r.read(0);
        assert_eq!(b, vec!["aa".as_bytes()]);
        assert_eq!(s, 1);
        let (b, s) = r.read(0);
        assert_eq!(b, Vec::<Vec<u8>>::new());
        assert_eq!(s, 0);
    }

    #[test]
    fn empty_line() {
        let bulk_size = 10usize;
        let s: [&str; 0] = [];
        let lines = s.join("\0");
        //let ex_lines = s.as_slice();

        let file = lines.as_bytes();
        let mut r = BulkReader::new(file, bulk_size);

        let (b, s) = r.read(0);
        assert_eq!(b, Vec::<Vec<u8>>::new());
        assert_eq!(s, 0);
    }

    #[test]
    fn multiple_lines_into_one_bulk() {
        let bulk_size = 10usize;
        let s = ["aa", "bb", "cc"];
        let lines = s.join("\0");
        let ex_lines = s.as_slice();

        let file = lines.as_bytes();
        let mut r = BulkReader::new(file, bulk_size);

        let (b, s) = r.read(0);
        assert_eq!(b, lines_to_bulk(ex_lines, true));
        assert_eq!(s, 3);
        let (b, s) = r.read(0);
        assert_eq!(b, Vec::<Vec<u8>>::new());
        assert_eq!(s, 0);
    }

    #[test]
    fn multiple_lines_spread_some_bulks() {
        let bulk_size = 10usize;
        let s = [
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
        ];
        let lines = s.join("\0");
        let ex_lines = s.as_slice();

        let file = lines.as_bytes();
        let mut r = BulkReader::new(file, bulk_size);

        let (b, s) = r.read(0);
        assert_eq!(b, lines_to_bulk(&ex_lines[0..10], false));
        assert_eq!(s, 10);
        let (b, s) = r.read(0);
        assert_eq!(b, lines_to_bulk(&ex_lines[10..12], true));
        assert_eq!(s, 2);
        let (b, s) = r.read(0);
        assert_eq!(b, Vec::<Vec<u8>>::new());
        assert_eq!(s, 0);
    }
}
