use graphdb::query::parser::Lexer;

fn main() {
    let input = "CREATE MATCH RETURN";
    let mut lexer = Lexer::new(input);
    
    println!("Input: {}", input);
    println!("Tokens:");
    
    loop {
        let token = lexer.next_token();
        println!("  {:?} - '{}'", token.kind, token.lexeme);
        if token.kind == graphdb::query::parser::TokenKind::Eof {
            break;
        }
    }
}