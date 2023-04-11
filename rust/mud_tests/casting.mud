(main := fn () -> i32{
  small_num : u8;
  small_num = 42;

  big_num : i32;
  big_num = 24;
  
  big_num = small_num;
  <big_num
})
