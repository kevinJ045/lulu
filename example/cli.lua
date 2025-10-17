enum! Token, {
  @validate_type("string")
  String(name)
}

print(Token.String(11))