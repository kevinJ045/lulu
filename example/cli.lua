
spread! mytable, {
  first,
  _,
  ...thing,
  last_1,
  named: name,
  &named,
  last_2
}

-- type! {
--   mytype = string
--   Vec2 = { x: number, y: number }
-- }

-- class! MyClass, {}
-- class! MyClass2:MyClass, {}

-- let! a: mytype = "sjsj";
-- let! b: Vec2 = {
--   x = 10,
--   y = 10
-- };
-- let! c: MyClass = MyClass2();


-- class! Vec2Class(x, y), {}

-- let! d: Vec2 = Vec2Class();



-- let! e: string or number  = "something";

-- match! ~e, {
--   string {
--     -- if string
--   }
--   Vec2 {
--     -- if vec2
--   }
--   mytype {
--     -- if mytype
--   }
-- }
