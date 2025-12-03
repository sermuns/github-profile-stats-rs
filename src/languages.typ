// #import "@preview/cetz:0.4.2": canvas
// #import "@preview/cetz-plot:0.1.3": chart

#set page(
  width: 13cm,
  height: auto,
  margin: 1em,
  fill: luma(10%),
)
#set text(
  font: "Noto Sans",
  fill: luma(90%),
)
#show heading: set text(yellow.lighten(50%))
#show heading: set block(below: 1em)
#set rect(width: 100%)

// #let inputs = (
//   "C++": 35460,
//   Typst: 19300,
//   HTML: 13617,
//   VHDL: 8440,
//   Python: 7851,
// )
#sys.inputs

= Most used languages

// #grid(
//   columns: 2,
//   gutter: 1fr,
//   grid(
//     columns: 2,
//     gutter: 1em,
//     ..inputs.pairs().flatten().map(str),
//   ),
//
//   canvas({
//     let colors = gradient.linear(red, blue, green, yellow)
//
//     chart.piechart(
//       inputs.pairs(),
//       value-key: 1,
//       label-key: none,
//       radius: 3,
//       stroke: none,
//       slice-style: colors,
//       inner-radius: 1,
//       outer-label: (content: "%", radius: 100%),
//     )
//   }),
// )
