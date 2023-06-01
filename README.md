# YSETL Language

A small, set-based programming language based off of ISETL.

## History

In the beginning, there was [SETL](https://en.wikipedia.org/wiki/SETL), showing up in 1969. It provided 2 composite data types: sets and tuples, and many built-in operations for working with sets. 2 decades later, Gary Levin, an associate professor of compsci at Clarkson University, developed ISETL (Interactive SETL) primary for use in 2 textbooks:
- **Learning Discrete Mathematics with ISETL** (1988, *ISBN 0-387-96898-9*)
- **Learning Abstract Algebra with ISETL** (1994, *ISBN 0-387-94152-5*)

3 decades after that, I went to Boston on my birthday and stopped at Brattle Book Shop, a very old used book store. My favorite section in used book stores is always the STEM section. Something about old math and programming books just hits different. On this particular day, I found the Abstract Algebra book listed above, in like-new condition with a floppy disk of the ISETL language still in an unopened envelope on the inside back cover. What a find. On my way home, I also found that someone actually put the ISETL source code on Github back in 2021, with a Make recipe to build on Ubuntu (and I would later realize that that _someone_ is Gary Levin himself). It definitely works, but I wanted something a little more modern: a smoother REPL experience, safer scoping rules for variables, better flow control, sleeker syntax, and some features that I would just like to have personally (like atom literals). But the OG language has some really neat features that you don't normally see in modern languages that might be fun to have around like dynamic variables, function overrides, and of course, native set operations. 

While my personal implementation isn't designed to be a hammer for every nail, I can absolutely see this being a useful tool for math-heavy software development. If this is in any usable state by the end of the year, I may try to tackle Advent of Code 2023 in YSETL.

## Name

There's nothing special about the name **YSETL**, and I'm not breaking any new ground here. I just wanted something with **-SETL** in the name, and "Y" is funny because truly, I have to ask myself: _"y r u doin this??"_. The answer, unsurprisingly, is `¯\_( ͡° ͜ʖ ͡°)_/¯`

---

## Features

### DataTypes:
- [x] Booleans
- [x] Integers
- [x] Floats
- [x] Strings
- [ ] Atoms
- [x] Tuples (Lists)
- [x] Sets
- [ ] Maps (specialized Sets)
- [ ] Functions

### Operations
- [x] Arithmetic
- [x] Control flow
- [x] Global variables
- [ ] Local variables
- [ ] Dynamic variables
- [ ] Boolean operations
- [ ] Tuple operations
- [ ] Set operations
- [ ] Map operations
- [ ] Iteration
- [ ] Function overrides

### Other
- [ ] REPL
- [ ] IO
- [ ] Separate Compilation and Execute steps (aka running prebuilt binaries)
