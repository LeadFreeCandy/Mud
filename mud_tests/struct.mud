Cat := struct{
  name : *u8,
  age : u8
};


(main := fn() -> i32 {
  cat : Cat
  #cat = {name : "tom"; age : 7}
  #cat.name = "tom" 
})

