fn main() {
    let code = r#"local items = {0, 5, 10}

macro {
  for_each ($key, $iterator, $expr) {
    for $key in ipairs($iterator) do
      $expr
    end
  }
}

for_each! item, items {
  print(item)
}"#;

    println!("Original code:");
    println!("{}", code);
    println!();
    println!("{}", "=".repeat(50));
    println!();

    println!("Compiled code:");
    println!("{}", lulu::compiler::compile(code));
}
