#[cfg(test)]
mod test_enhanced_lexer {
    use graphdb::query::parser::core::token::{Token, TokenKind};
    use graphdb::query::parser::lexer::lexer::Lexer;

    fn assert_token(token: &Token, expected_kind: TokenKind, expected_lexeme: &str) {
        assert_eq!(
            token.kind, expected_kind,
            "Token kind mismatch. Expected: {:?}, Got: {:?}",
            expected_kind, token.kind
        );
        assert_eq!(
            token.lexeme, expected_lexeme,
            "Lexeme mismatch for token {:?}. Expected: '{}', Got: '{}'",
            expected_kind, expected_lexeme, token.lexeme
        );
    }

    #[test]
    fn test_special_properties() {
        let input = "_id _type _src _dst _rank";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::IdProp, "_id");
        assert_token(&lexer.next_token(), TokenKind::TypeProp, "_type");
        assert_token(&lexer.next_token(), TokenKind::SrcIdProp, "_src");
        assert_token(&lexer.next_token(), TokenKind::DstIdProp, "_dst");
        assert_token(&lexer.next_token(), TokenKind::RankProp, "_rank");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_graph_reference_identifiers() {
        let input = "$$ $^ $-";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::DstRef, "$$");
        assert_token(&lexer.next_token(), TokenKind::SrcRef, "$^");
        assert_token(&lexer.next_token(), TokenKind::InputRef, "$-");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_aggregation_functions() {
        let input = "COUNT SUM AVG MIN MAX";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::Count, "COUNT");
        assert_token(&lexer.next_token(), TokenKind::Sum, "SUM");
        assert_token(&lexer.next_token(), TokenKind::Avg, "AVG");
        assert_token(&lexer.next_token(), TokenKind::Min, "MIN");
        assert_token(&lexer.next_token(), TokenKind::Max, "MAX");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_new_keywords() {
        let input = "SOURCE DESTINATION RANK INPUT";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::Source, "SOURCE");
        assert_token(&lexer.next_token(), TokenKind::Destination, "DESTINATION");
        assert_token(&lexer.next_token(), TokenKind::Rank, "RANK");
        assert_token(&lexer.next_token(), TokenKind::Input, "INPUT");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_basic_functionality() {
        // Test that basic functionality still works after our enhancements
        let input = "CREATE (n:Person {name: 'John'}) RETURN n.name";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::Create, "CREATE");
        assert_token(&lexer.next_token(), TokenKind::LParen, "(");
        assert_token(
            &lexer.next_token(),
            TokenKind::Identifier("n".to_string()),
            "n",
        );
        assert_token(&lexer.next_token(), TokenKind::Colon, ":");
        assert_token(
            &lexer.next_token(),
            TokenKind::Identifier("Person".to_string()),
            "Person",
        );
    }
}
