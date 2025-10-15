
enum! Something, {
  Variant(content)
  EmptyVariant
}, {
  unwrap(item) {
    return match! item, {
      Something.Variant {
        return item.content
      }
      _ {
        return nil
      }
    }
  }
  change_content(item, value) {
    match! item, {
      Something.Variant {
        item.content =  value
      }
    }
  }
}

local value = Something.Variant("some content")
value.change_content("into")
print(value.unwrap())