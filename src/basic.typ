#set page(
  width: auto,
  height: auto,
  margin: 2em,
  fill: luma(10%),
)
#set text(
  font: "Noto Sans",
  fill: luma(90%),
)
#show heading: set text(yellow)
#show heading: set block(below: 1em)

= Samuel Åkesson's GitHub Stats

#grid(
  columns: 2,
  gutter: 1em,
  [Total Stars:], [],
  [Total Commits:], [],
  [Total PRs:], [],
  [Total Issues:], [],
  [Contributed to:], [],
)
