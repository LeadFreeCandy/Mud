LEXEME_TYPE_INTEGER := 0;
LEXEME_TYPE_OPERATOR := 2;
LEXEME_TYPE_IDENTIFIER := 1;
#LEXEME_TYPE_STRING := 3;
#LEXEME_TYPE_KEYWORD := 4;

OPERATOR_MINUS := -1;
OPERATOR_PLUS := 0;
OPERATOR_ASTERISK := 1;


Lexer := struct {
    program: *u8;
    index: i32,

    data_num: i32,
    data_str: *u8,

    type: i32,
}

main := fn(argc: i32, argv: **u8) -> u8 {
    filename : *u8;
    filename = *(argv + 1);
    filename : *u8 = *(argv + 1);
    filename := *(argv + 1);

    filename = *(argv + 1);

    return 0
}
