Lexer := struct {
    program: *u8;
    index: i32,

    integer_literal: i32,
    string: *u8,
    ...,

    type: u8,
}

main := fn(argc: i32, argv: **u8) -> u8 {
    i : i32;
    i = argc;

    (while (i) {
        <*(argv + i);
        i = i - 1
    });

    return 0
}
