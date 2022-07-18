struct QuoteRplacePair {
    from: &'static str,
    to: &'static str,
}

pub trait DoQuote {
    fn wrap_single_quote(&self, line: String) -> String {
        line.replace('\'', "'\"'\"'")
    }

    fn replace(&self, line: String) -> String;

    fn quote(&self, line: String) -> String {
        format!("'{}'", self.replace(self.wrap_single_quote(line)))
    }
}

pub struct QuoteBasic {}

impl DoQuote for QuoteBasic {
    fn replace(&self, line: String) -> String {
        line
    }
}

pub struct QuotePrintable {}

impl DoQuote for QuotePrintable {
    fn replace(&self, line: String) -> String {
        // スタックに積まれないはず？
        const TBL: &[QuoteRplacePair] = &[
            QuoteRplacePair {
                from: "\u{8}",
                to: "'$'\\b''",
            },
            QuoteRplacePair {
                from: "\n",
                to: "'$'\\n''",
            },
        ];
        let mut ret: String = line;
        for pair in TBL {
            ret = ret.replace(pair.from, pair.to);
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::quote::{DoQuote, QuoteBasic, QuotePrintable};

    #[test]
    fn quote_line_by_basic() {
        let qb = QuoteBasic {};

        let quoted = qb.quote("test".to_string());
        assert_eq!(quoted, "'test'");

        let quoted = qb.quote("test test".to_string());
        assert_eq!(quoted, "'test test'");

        let quoted = qb.quote("test'test".to_string());
        assert_eq!(quoted, "'test'\"'\"'test'");

        let quoted = qb.quote("test\ntest".to_string());
        assert_eq!(quoted, "'test\ntest'");

        let quoted = qb.quote("test テスト".to_string());
        assert_eq!(quoted, "'test テスト'");

        let quoted = qb.quote("test'テスト".to_string());
        assert_eq!(quoted, "'test'\"'\"'テスト'");

        let quoted = qb.quote("test''テスト".to_string());
        assert_eq!(quoted, "'test'\"'\"''\"'\"'テスト'");

        let quoted = qb.quote("test'\nテスト".to_string());
        assert_eq!(quoted, "'test'\"'\"'\nテスト'");

        let quoted = qb.quote("test'🦀テスト".to_string());
        assert_eq!(quoted, "'test'\"'\"'🦀テスト'");
    }

    #[test]
    fn quote_line_by_printable() {
        let qb = QuotePrintable {};

        let quoted = qb.quote("test\u{8}test".to_string());
        assert_eq!(quoted, "'test'$'\\b''test'");

        let quoted = qb.quote("test test\u{8}".to_string());
        assert_eq!(quoted, "'test test'$'\\b'''");

        let quoted = qb.quote("test\ntest".to_string());
        assert_eq!(quoted, "'test'$'\\n''test'");

        let quoted = qb.quote("test test\n".to_string());
        assert_eq!(quoted, "'test test'$'\\n'''");
    }
}
