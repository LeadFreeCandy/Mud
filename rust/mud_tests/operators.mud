main := fn() -> i32 {
     lt : i32;
     gt : i32;
     eq : i32;
     ne : i32;

     and : i32;
     or  : i32;
     not : i32;

     a : i32;
     b : i32;
     c : i32;

     a = 1;
     b = 2;
     c = 3;

     lt = a < c;
     gt = c > b;
     eq = a == c - b;
     ne = a != b;

     and = a && b;
     or = a - 1 || b;

     not = !(a - 1);

     if lt && gt && eq && ne && and && or && not {
               <"Passed"
     }
}
