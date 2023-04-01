Stats := struct{
  health: u8,
  speed: i32
};

Cat := struct{
  name : *u8,
  age : i32
};


(main := fn() -> i32 {
  cat : Cat;
  cat.name = "tom";
  cat.age = 7;

  <(cat.name);
  <" the cat is ";
  <(cat.age);
  <" years old.\n"
})

