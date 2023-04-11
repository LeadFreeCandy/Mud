LEXEME_TYPE_INTEGER := 0;
LEXEME_TYPE_OPERATOR := 2;
LEXEME_TYPE_IDENTIFIER := 1;
#LEXEME_TYPE_STRING := 3;
#LEXEME_TYPE_KEYWORD := 4;

OPERATOR_MINUS := 0;
OPERATOR_PLUS := 1;
OPERATOR_ASTERISK := 2;


(Lexer := struct {
    program: *u8,
    index: i32,

    data_num: i32,
    data_str: *u8,

    type: i32
});




main := fn(argc: i32, argv: **u8) -> i32 {
    filename : *u8;
    filename = *(argv + 1);

    file : *u8;
    file = read_file(filename);
    return 0
}
