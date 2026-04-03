## SGF to Kifu
Command line tool to generate printable kifu from [SGF](https://en.wikipedia.org/wiki/Smart_Game_Format) files for go/baduk/weiqi games. 
A [kifu](https://senseis.xmp.net/?Kifu)(棋譜) is a record of the game where the moves are numbered in one diagram. 
They are used to replay the game on a physical board, which is the best way to study pro games.

<img src=./render/19570414-Go_Seigen-Kitani_Minoru.svg>

### Usage
```shell
sgf-to-kifu <sgf-file> [<more-files>...]
```
Outputs SVG and PNG kifu of each SGF file input.

Example SGFs and kifus are under `./sgfs` and `./renders`
