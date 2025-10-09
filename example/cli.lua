-- print(serde.json.encode({
--   "name"
-- }))
-- print(serde.yaml.encode({
--   item = {
--     name = "item\nn"
--   }
-- }))

iprint(re.replace("(^r)(.*)(r$)", "rASSSSr", function(_, g1, _, g2)
  return f"{g1}sss{g2}"
end))