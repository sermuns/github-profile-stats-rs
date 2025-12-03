#set page(
  width: 13cm,
  height: 5cm,
  fill: luma(10%),
)
#set text(
  font: "Noto Sans",
  fill: luma(90%),
)
#show heading: set text(yellow.lighten(50%))
#show heading: set block(below: 1em)
#set rect(width: 100%)

#let language-colors = (

)

= Most used languages

#grid(
  columns: (1fr,) * 2,
  rect(fill: white),
)
