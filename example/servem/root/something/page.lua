using {
  lulib.rayous
}

@AsWidget()
function MyWidget(@WidgetProps({ name = "" }) props)
  fprint(props)
end

return function()
  return Widget {
    Text {
      text = "Something"
    },
    MyWidget {}
  }
end
