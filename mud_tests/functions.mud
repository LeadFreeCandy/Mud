(func1 := fn(i: i32) -> i32 {
    i2: i32;
    i2 = i + 1;
    return i2 + i
});


(func2 := fn(ptr: *i32) -> *i32 {
    ptr2: *i32;
    ptr2 = ptr + 1;
    return ptr + 2
});

(main := fn() -> i32 {
    a: i32;
    b: *i32;
    c: i32;

    c = func1(34) + func1(3234);
    a = func1(0);
    b = malloc(23);
    b = func2(0)
})
