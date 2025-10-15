
  class!
  @States({
    name = "something",
    id = 1
  })
  @Component()
  MyWidget:Widget(props),
  {
    @async()
    @stated()
    render(@Props({
      name = self.name,
      id = self.id
    }) props, @List children) {
      return lml! {
        <box name={self.name} id={self.id}>
          {children}
        </box> 
      }
    }
  }
