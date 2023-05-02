Advent of Code involves quite a lot of string parsing, the kind of parsing `parsely` aims to excel at!

If you haven't heard of [Advent of Code](https://adventofcode.com/) before, here's a snippet from its about page:

> Advent of Code is an Advent calendar of small programming puzzles for a variety of skill sets and skill levels
> that can be solved in any programming language you like.
>
> People use them as interview prep, company training, university coursework, practice problems, a speed contest, or to challenge each other.

In this folder there a few solutions to some of the simpler advent of code puzzles that use parsely's parsers to transform the puzzle input into rust types.

## 2015

* Day 1: [Not quite lisp](not_quite_lisp.rs) uses [`char()`](../../src/lexer/char.rs) and [`.map()`](../../src/combinator/map.rs) to translate `(` and `)` into +/-1
* Day 2: [I was told there would be no math](i_was_told_there_would_be_no_math.rs) uses [`.then()`](../../src/combinator/then.rs) and [`.then_skip()`](../../src/combinator/skip.rs) to parse `lxwxh` into 3 separate dimension values
